//! Montage et outils de test du serveur MCP — Story 32-6a (refs #723).
//!
//! - le montage : sociétés, administrateur, clés MCP, réglages, routeur par
//!   `build_router_avec_registre` sur un `static_dir` **réel** (un
//!   `index.html` de test), requêtes par `tower::ServiceExt::oneshot` avec
//!   `MockConnectInfo` ;
//! - les outils de test (AC-a21) : **compteur**, **piégé**, **refus de
//!   service**, **lent**.
//!
//! Aucune donnée réelle : fixtures synthétiques et valeurs piégées.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::extract::connect_info::MockConnectInfo;
use axum::http::{HeaderMap, Request, StatusCode};
use chrono::TimeDelta;
use http_body_util::BodyExt;
use kesh_api::config::Config;
use kesh_api::mcp::outil::FuturOutil;
use kesh_api::mcp::{ContexteMcp, Donnees, ErreurOutil, McpTool, Registre};
use kesh_api::services::acces::Portee;
use kesh_api::{AppState, build_router_avec_registre};
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{
    Language, McpScope, McpScopes, NewCompany, NewMcpKey, NewUser, OrgType, Role, SendValidation,
};
use kesh_db::repositories::{companies, mcp_keys, users};
use sqlx::MySqlPool;
use tower::ServiceExt;

/// Le marqueur que l'outil piégé lit dans les livres (AC-a22).
pub const PIEGE_E1: &str = "PIEGE_E1_326";

/// Le contenu de l'`index.html` de test.
pub const INDEX_HTML: &str = "<!doctype html><title>Kesh 326</title><p>index de test</p>";

/// Le secret JWT de la `Config` de test.
pub const TEST_JWT_SECRET: &str = "test-secret-32-bytes-minimum-test-secret-padding";

/// Un compte du plan de la société, nommé `nom` (fixture synthétique).
pub async fn compte(pool: &MySqlPool, company_id: i64, user_id: i64, numero: &str, nom: &str) {
    kesh_db::repositories::accounts::create(
        pool,
        user_id,
        kesh_db::entities::NewAccount::new(
            company_id,
            numero,
            nom,
            kesh_db::entities::AccountType::Asset,
            None,
        ),
    )
    .await
    .expect("compte");
}

/// Une société ; rend son id.
pub async fn societe(pool: &MySqlPool, nom: &str) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: nom.into(),
            first_name: None,
            last_name: None,
            address_structured: StructuredAddress {
                street: "Rue Test".into(),
                building: "1".into(),
                postal_code: "1000".into(),
                city: "Lausanne".into(),
                country: "CH".into(),
            },
            ide_number: None,
            org_type: OrgType::Independant,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .expect("société")
    .id
}

/// Un administrateur de la société ; rend son id.
pub async fn administrateur(pool: &MySqlPool, company_id: i64, nom: &str) -> i64 {
    users::create(
        pool,
        NewUser {
            username: nom.into(),
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$dGVzdA$dGVzdA".into(),
            role: Role::Admin,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .expect("utilisateur")
    .id
}

/// Une clé MCP ; rend `(id, jeton)`.
pub async fn cle(
    pool: &MySqlPool,
    company_id: i64,
    user_id: i64,
    portees: &[McpScope],
    validation: SendValidation,
) -> (i64, String) {
    let (jeton, key_hash) = kesh_api::auth::api_key::generate_mcp_key();
    let mut tx = pool.begin().await.unwrap();
    let id = mcp_keys::create_in_tx(
        &mut tx,
        NewMcpKey {
            company_id,
            created_by_user_id: user_id,
            name: "agent".into(),
            key_hash,
            scopes: McpScopes::new(portees.iter().copied()).unwrap(),
            send_validation: validation,
            expires_at: None,
        },
    )
    .await
    .expect("clé MCP")
    .id;
    tx.commit().await.unwrap();
    (id, jeton)
}

/// Écrit (ou remplace) la ligne de réglages d'une société, en SQL brut.
pub async fn reglages(
    pool: &MySqlPool,
    company_id: i64,
    enabled: bool,
    filtre: bool,
    reseaux: &str,
    mandataires: &str,
    origines: &str,
) {
    sqlx::query(
        "REPLACE INTO mcp_server_settings (company_id, enabled, address_filter_enabled, \
         allowed_networks, trusted_proxies, allowed_origins) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(enabled)
    .bind(filtre)
    .bind(reseaux)
    .bind(mandataires)
    .bind(origines)
    .execute(pool)
    .await
    .expect("réglages MCP");
}

/// Les sept plages par défaut, en JSON.
pub const RESEAUX_DEFAUT: &str = r#"["127.0.0.0/8","::1/128","10.0.0.0/8","172.16.0.0/12","192.168.0.0/16","fc00::/7","fe80::/10"]"#;

/// Réglages activés aux plages par défaut, sans mandataire ni origine.
pub async fn activer(pool: &MySqlPool, company_id: i64) {
    reglages(pool, company_id, true, true, RESEAUX_DEFAUT, "[]", "[]").await;
}

/// Une société et son administrateur, d'ids distincts (une société leurre
/// d'abord) ; rend `(société, administrateur)`.
pub async fn livres(pool: &MySqlPool) -> (i64, i64) {
    societe(pool, "Leurre").await;
    let c = societe(pool, "Livres MCP").await;
    let u = administrateur(pool, c, "admin-mcp").await;
    (c, u)
}

/// La `Config` de test : `port` 80, sans `public_base_url`.
pub fn config() -> Config {
    Config::from_fields_for_test(
        "mysql://test:test@localhost:3306/test".to_string(),
        "admin".to_string(),
        "e2e-test-admin-password".to_string(),
        TEST_JWT_SECRET.to_string(),
        TimeDelta::minutes(15),
        TimeDelta::days(30),
        TimeDelta::minutes(15),
        TimeDelta::minutes(15),
        100,
        TimeDelta::minutes(30),
        12,
    )
}

/// Un `static_dir` réel, avec son `index.html`.
pub fn static_dir() -> tempfile::TempDir {
    let d = tempfile::tempdir().expect("dossier statique");
    std::fs::write(d.path().join("index.html"), INDEX_HTML).unwrap();
    d
}

fn i18n() -> Arc<kesh_i18n::I18nBundle> {
    Arc::new(
        kesh_i18n::I18nBundle::load(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("i18n"),
    )
}

/// L'état d'application de test sur `pool`, avec `config`.
pub fn etat(pool: MySqlPool, config: Config) -> AppState {
    let rate_limiter = kesh_api::middleware::rate_limit::RateLimiter::new(&config);
    AppState::new_for_tests(pool, Arc::new(config), Arc::new(rate_limiter), i18n())
}

/// Le routeur, sans `ConnectInfo` (pour le test de l'extracteur absent).
pub fn routeur_nu(pool: MySqlPool, config: Config, statique: &Path, registre: Registre) -> Router {
    build_router_avec_registre(
        etat(pool, config),
        statique.to_string_lossy().into_owned(),
        registre,
    )
}

/// Le routeur, le pair simulé étant `pair`.
pub fn routeur(
    pool: MySqlPool,
    config: Config,
    statique: &Path,
    registre: Registre,
    pair: &str,
) -> Router {
    let pair: SocketAddr = format!("{}:50000", ip_crochets(pair)).parse().unwrap();
    routeur_nu(pool, config, statique, registre).layer(MockConnectInfo(pair))
}

fn ip_crochets(ip: &str) -> String {
    if ip.contains(':') {
        format!("[{ip}]")
    } else {
        ip.to_string()
    }
}

/// Une réponse, entière : statut, en-têtes **hors `date`**, octets du corps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rendu {
    pub statut: StatusCode,
    pub en_tetes: Vec<(String, Vec<u8>)>,
    pub corps: Vec<u8>,
}

impl Rendu {
    pub fn json(&self) -> serde_json::Value {
        serde_json::from_slice(&self.corps).expect("corps JSON")
    }

    pub fn en_tete(&self, nom: &str) -> Option<String> {
        self.en_tetes
            .iter()
            .find(|(n, _)| n == nom)
            .map(|(_, v)| String::from_utf8_lossy(v).into_owned())
    }
}

/// Envoie une requête par `oneshot` et rend la réponse entière.
pub async fn envoyer(app: &Router, requete: Request<Body>) -> Rendu {
    let r = app.clone().oneshot(requete).await.expect("réponse");
    let statut = r.status();
    let mut en_tetes: Vec<(String, Vec<u8>)> = r
        .headers()
        .iter()
        .filter(|(n, _)| n.as_str() != "date")
        .map(|(n, v)| (n.as_str().to_string(), v.as_bytes().to_vec()))
        .collect();
    en_tetes.sort();
    let corps = r.into_body().collect().await.unwrap().to_bytes().to_vec();
    Rendu {
        statut,
        en_tetes,
        corps,
    }
}

/// Une requête `POST /mcp` JSON-RPC, sous `jeton` s'il y en a un.
pub fn post_mcp(jeton: Option<&str>, corps: &str) -> Request<Body> {
    let mut b = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("content-type", "application/json")
        .header("mcp-protocol-version", "2025-11-25");
    if let Some(j) = jeton {
        b = b.header("authorization", format!("Bearer {j}"));
    }
    b.body(Body::from(corps.to_string())).unwrap()
}

/// Une requête vers `chemin`, méthode et en-têtes choisis.
pub fn requete(methode: &str, chemin: &str, en_tetes: &HeaderMap, corps: Body) -> Request<Body> {
    let mut b = Request::builder().method(methode).uri(chemin);
    for (n, v) in en_tetes {
        b = b.header(n, v);
    }
    b.body(corps).unwrap()
}

/// Un message `tools/call`.
pub fn appel(id: i64, nom: &str, arguments: serde_json::Value) -> String {
    serde_json::json!({
        "jsonrpc": "2.0", "id": id, "method": "tools/call",
        "params": { "name": nom, "arguments": arguments },
    })
    .to_string()
}

/// La réponse attendue d'un outil en erreur (`isError: true`, texte fixe).
pub fn attendu_outil_en_erreur(id: i64, texte: &str) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": { "content": [{ "type": "text", "text": texte }], "isError": true },
    })
}

/// La réponse attendue d'une erreur JSON-RPC à l'`id` de la requête.
pub fn attendu_erreur(id: i64, code: i64, message: &str) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message },
    })
}

// ---------------------------------------------------------------------------
// Les outils de test (AC-a21)
// ---------------------------------------------------------------------------

/// L'outil **compteur** : compte ses exécutions, rend `{"n": …}`.
pub struct Compteur {
    pub nom: &'static str,
    pub portee: Portee,
    pub n: Arc<AtomicUsize>,
}

impl Compteur {
    pub fn nouveau(nom: &'static str, portee: Portee) -> (Self, Arc<AtomicUsize>) {
        let n = Arc::new(AtomicUsize::new(0));
        (
            Self {
                nom,
                portee,
                n: n.clone(),
            },
            n,
        )
    }
}

const SCHEMA_VIDE: &str = r#"{"type":"object","properties":{},"additionalProperties":false}"#;

impl McpTool for Compteur {
    fn nom(&self) -> &'static str {
        self.nom
    }
    fn description(&self) -> &'static str {
        "outil de test : compteur"
    }
    fn schema_entree(&self) -> &'static str {
        SCHEMA_VIDE
    }
    fn portee(&self) -> Portee {
        self.portee
    }
    fn executer<'a>(&'a self, _ctx: &'a ContexteMcp<'a>, _a: serde_json::Value) -> FuturOutil<'a> {
        Box::pin(async move {
            let n = self.n.fetch_add(1, Ordering::SeqCst) + 1;
            let s = serde_json::json!({ "n": n });
            Ok(Donnees::nouvelles(s.to_string()))
        })
    }
}

/// L'outil **piégé** : lit le plan comptable par `plan_comptable::lister` et
/// rend les noms des comptes (dont celui qui porte [`PIEGE_E1`]).
pub struct Piege;

impl McpTool for Piege {
    fn nom(&self) -> &'static str {
        "piege"
    }
    fn description(&self) -> &'static str {
        "outil de test : lit le plan comptable"
    }
    fn schema_entree(&self) -> &'static str {
        SCHEMA_VIDE
    }
    fn portee(&self) -> Portee {
        Portee::BooksRead
    }
    fn executer<'a>(&'a self, ctx: &'a ContexteMcp<'a>, _a: serde_json::Value) -> FuturOutil<'a> {
        Box::pin(async move {
            let comptes =
                kesh_api::services::plan_comptable::lister(ctx.acces(), ctx.lecture(), true)
                    .await
                    .map_err(kesh_api::mcp::textes::depuis_app_error)?;
            let noms: Vec<String> = comptes.into_iter().map(|c| c.compte.name).collect();
            Ok(Donnees::nouvelles(noms.join(", ")))
        })
    }
}

/// L'outil de **refus de service** : portée `BooksRead`, mais appelle
/// `factures::lister` (portée `InvoicesRead`) — le service refuse.
pub struct RefusDeService {
    pub n: Arc<AtomicUsize>,
}

impl McpTool for RefusDeService {
    fn nom(&self) -> &'static str {
        "refus_de_service"
    }
    fn description(&self) -> &'static str {
        "outil de test : le service refuse"
    }
    fn schema_entree(&self) -> &'static str {
        SCHEMA_VIDE
    }
    fn portee(&self) -> Portee {
        Portee::BooksRead
    }
    fn executer<'a>(&'a self, ctx: &'a ContexteMcp<'a>, _a: serde_json::Value) -> FuturOutil<'a> {
        Box::pin(async move {
            self.n.fetch_add(1, Ordering::SeqCst);
            kesh_api::services::factures::lister(ctx.acces(), ctx.lecture(), Default::default())
                .await
                .map_err(kesh_api::mcp::textes::depuis_app_error)?;
            Err(ErreurOutil::Interne)
        })
    }
}

/// L'outil **lent** : dort plus que le délai réduit des tests.
pub struct Lent;

impl McpTool for Lent {
    fn nom(&self) -> &'static str {
        "lent"
    }
    fn description(&self) -> &'static str {
        "outil de test : lent"
    }
    fn schema_entree(&self) -> &'static str {
        SCHEMA_VIDE
    }
    fn portee(&self) -> Portee {
        Portee::BooksRead
    }
    fn executer<'a>(&'a self, _ctx: &'a ContexteMcp<'a>, _a: serde_json::Value) -> FuturOutil<'a> {
        Box::pin(async move {
            tokio::time::sleep(Duration::from_secs(5)).await;
            Ok(Donnees::nouvelles(String::new()))
        })
    }
}

/// L'outil **lecteur retardé** : attend `attente`, puis lit le plan comptable
/// par `plan_comptable::lister` — une instruction sur `accounts`, que le test
/// peut tenir en attente par un `LOCK TABLES accounts WRITE` (revue P2, H1).
/// L'attente fait partir l'instruction **après** le début du délai de Kesh :
/// c'est le cas où elle survit à l'abandon.
pub struct LecteurRetarde {
    pub attente: Duration,
}

impl McpTool for LecteurRetarde {
    fn nom(&self) -> &'static str {
        "lecteur_retarde"
    }
    fn description(&self) -> &'static str {
        "outil de test : lit le plan comptable après une attente"
    }
    fn schema_entree(&self) -> &'static str {
        SCHEMA_VIDE
    }
    fn portee(&self) -> Portee {
        Portee::BooksRead
    }
    fn executer<'a>(&'a self, ctx: &'a ContexteMcp<'a>, _a: serde_json::Value) -> FuturOutil<'a> {
        Box::pin(async move {
            tokio::time::sleep(self.attente).await;
            kesh_api::services::plan_comptable::lister(ctx.acces(), ctx.lecture(), true)
                .await
                .map_err(kesh_api::mcp::textes::depuis_app_error)?;
            Ok(Donnees::nouvelles(String::new()))
        })
    }
}

/// L'outil **en erreur** (Story 32-6b) : rend toujours `erreur`, sans rien
/// lire — pour les issues `introuvable` et `erreur` du journal.
pub struct EnErreur {
    pub nom: &'static str,
    pub erreur: ErreurOutil,
}

impl McpTool for EnErreur {
    fn nom(&self) -> &'static str {
        self.nom
    }
    fn description(&self) -> &'static str {
        "outil de test : rend une erreur"
    }
    fn schema_entree(&self) -> &'static str {
        SCHEMA_VIDE
    }
    fn portee(&self) -> Portee {
        Portee::BooksRead
    }
    fn executer<'a>(&'a self, _ctx: &'a ContexteMcp<'a>, _a: serde_json::Value) -> FuturOutil<'a> {
        let erreur = self.erreur;
        Box::pin(async move { Err(erreur) })
    }
}

/// L'outil **paniqué** (Story 32-6b) : panique à l'exécution — pour l'issue
/// `erreur` d'un panic (C-32-6-58).
pub struct Panique;

impl McpTool for Panique {
    fn nom(&self) -> &'static str {
        "panique"
    }
    fn description(&self) -> &'static str {
        "outil de test : panique"
    }
    fn schema_entree(&self) -> &'static str {
        SCHEMA_VIDE
    }
    fn portee(&self) -> Portee {
        Portee::BooksRead
    }
    fn executer<'a>(&'a self, _ctx: &'a ContexteMcp<'a>, _a: serde_json::Value) -> FuturOutil<'a> {
        Box::pin(async move { panic!("outil de test : panique voulue (326b)") })
    }
}

// ---------------------------------------------------------------------------
// Story 32-8b — les envois à valider : un outil à sortie fixée et les aides
// ---------------------------------------------------------------------------

/// Outil de test de la 32-8b (AC-B15, point 11) : rend un texte et un résumé
/// **fixés** — les pièges de l'AC-B2, les caractères cachés de l'AC-B3, un
/// texte de taille réglable pour l'AC-B4 — et compte ses exécutions. (Son champ
/// `structure` est parti avec celui de `Donnees` à la 0.16.1, v016-1,
/// C-v016-1-5.)
pub struct Fixe {
    pub nom: &'static str,
    pub texte: String,
    pub resume: Option<String>,
    pub n: Arc<AtomicUsize>,
}

impl Fixe {
    /// Un outil `nom` (portée `books:read`) qui rend `texte`, sans résumé ; rend
    /// aussi son compteur.
    pub fn texte(nom: &'static str, texte: impl Into<String>) -> (Self, Arc<AtomicUsize>) {
        let n = Arc::new(AtomicUsize::new(0));
        (
            Self {
                nom,
                texte: texte.into(),
                resume: None,
                n: n.clone(),
            },
            n,
        )
    }
}

impl McpTool for Fixe {
    fn nom(&self) -> &'static str {
        self.nom
    }
    fn description(&self) -> &'static str {
        "outil de test : sortie fixée (32-8b)"
    }
    fn schema_entree(&self) -> &'static str {
        SCHEMA_VIDE
    }
    fn portee(&self) -> Portee {
        Portee::BooksRead
    }
    fn executer<'a>(&'a self, _ctx: &'a ContexteMcp<'a>, _a: serde_json::Value) -> FuturOutil<'a> {
        Box::pin(async move {
            self.n.fetch_add(1, Ordering::SeqCst);
            let d = Donnees::nouvelles(self.texte.clone());
            Ok(match &self.resume {
                Some(r) => d.avec_resume(r.clone()),
                None => d,
            })
        })
    }
}

/// `tools/call` de `kesh_lire_envoi` pour le numéro `n`.
pub fn lire_envoi(id: i64, n: i64) -> String {
    appel(id, "kesh_lire_envoi", serde_json::json!({ "envoi": n }))
}

/// Le numéro que rend l'accusé d'un dépôt, lu au préfixe « Envoi n° » de son
/// texte (v016-1, C-v016-1-9 : l'accusé ne porte plus de `structuredContent`).
pub fn numero_depose(r: &Rendu) -> i64 {
    r.json()["result"]["content"][0]["text"]
        .as_str()
        .and_then(|t| t.strip_prefix("Envoi n° "))
        .and_then(|t| t.split(' ').next())
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| {
            panic!(
                "accusé de dépôt attendu : {}",
                String::from_utf8_lossy(&r.corps)
            )
        })
}

/// La réponse attendue d'un dépôt réussi (AC-B5).
pub fn attendu_depose(id: i64, n: i64) -> serde_json::Value {
    serde_json::json!({"jsonrpc":"2.0","id":id,"result":{
        "content":[{"type":"text","text":kesh_api::mcp::textes::texte_envoi_depose(n)}],
        "isError":false}})
}

/// La réponse attendue d'une relecture d'un envoi en attente (AC-B6).
pub fn attendu_en_attente(id: i64, n: i64) -> serde_json::Value {
    serde_json::json!({"jsonrpc":"2.0","id":id,"result":{
        "content":[{"type":"text","text":kesh_api::mcp::textes::texte_envoi_en_attente(n)}],
        "isError":false}})
}

/// La réponse attendue d'une relecture d'un envoi validé (AC-B6) : le second
/// bloc est `texte`, octet pour octet.
pub fn attendu_valide(id: i64, n: i64, modifie: bool, texte: &str) -> serde_json::Value {
    serde_json::json!({"jsonrpc":"2.0","id":id,"result":{
        "content":[
            {"type":"text","text":kesh_api::mcp::textes::texte_envoi_valide(n, modifie)},
            {"type":"text","text":texte},
        ],
        "isError":false}})
}

/// La réponse attendue d'une relecture refusée, échue ou inconnue (AC-B6).
pub fn attendu_refuse(id: i64, n: i64) -> serde_json::Value {
    attendu_outil_en_erreur(id, &kesh_api::mcp::textes::texte_envoi_refuse(n))
}

/// Idem, échu.
pub fn attendu_echu(id: i64, n: i64) -> serde_json::Value {
    attendu_outil_en_erreur(id, &kesh_api::mcp::textes::texte_envoi_echu(n))
}

/// Idem, inconnu (numéro inexistant, autre clé, autre société).
pub fn attendu_inconnu(id: i64, n: i64) -> serde_json::Value {
    attendu_outil_en_erreur(id, &kesh_api::mcp::textes::texte_envoi_inconnu(n))
}

/// Une session (JWT forgé) d'un utilisateur `user_id` du rôle `role`.
pub fn session(user_id: i64, role: Role, company_id: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let claims = kesh_api::auth::jwt::Claims {
        sub: user_id.to_string(),
        role: role.as_str().to_string(),
        company_id,
        iat: now,
        exp: now + 3600,
    };
    jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
    )
    .unwrap()
}

/// Une requête vers l'API d'administration, `Authorization: Bearer <jeton>`,
/// corps JSON facultatif.
pub fn requete_api(
    methode: &str,
    chemin: &str,
    jeton: Option<&str>,
    corps: Option<serde_json::Value>,
) -> Request<Body> {
    let mut b = Request::builder().method(methode).uri(chemin);
    if let Some(j) = jeton {
        b = b.header("authorization", format!("Bearer {j}"));
    }
    match corps {
        Some(c) => b
            .header("content-type", "application/json")
            .body(Body::from(c.to_string()))
            .unwrap(),
        None => b.body(Body::empty()).unwrap(),
    }
}

/// Le contenu d'un envoi en base, octet pour octet (`None` si nul).
pub async fn contenu_en_base(pool: &MySqlPool, id: i64) -> Option<String> {
    sqlx::query_scalar("SELECT content FROM mcp_sends WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Le nombre de lignes de `mcp_sends`.
pub async fn envois_en_base(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM mcp_sends")
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Pose l'échéance de l'envoi `id` dans le passé (AC-B7).
pub async fn echoir(pool: &MySqlPool, id: i64) {
    sqlx::query("UPDATE mcp_sends SET expires_at = NOW(3) - INTERVAL 1 SECOND WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
}
