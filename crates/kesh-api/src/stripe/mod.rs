//! Client de l'API Stripe, **lecture seule** (Story 40-8a).
//!
//! Ce qui est propre à Stripe, le socle de la 38-12a faisant le reste
//! ([`crate::provider_env`], [`crate::http_client`],
//! [`crate::provider_download`]) :
//!
//! - **clé restreinte** (`rk_live_…`, `rk_test_…`) par compte Stripe, posée en
//!   variable d'environnement (`KESH_STRIPE_<n>_ACCOUNT`, `KESH_STRIPE_<n>_KEY`)
//!   et contrôlée au démarrage ([`check_field`]) : une clé secrète `sk_…` ou
//!   tout autre préfixe refuse le démarrage ; jamais en base, dans l'audit, une
//!   sauvegarde, un journal ni un message ;
//! - **requêtes `GET` seulement**, `Authorization: Bearer <clé>`, en-tête
//!   [`STRIPE_VERSION`] épinglé ;
//! - `GET /v1/payouts?arrival_date[gte]&arrival_date[lt]&status=paid&limit=100`,
//!   filtré par **`arrival_date`** (le jour que voit la banque, celui de
//!   `payout_date`) ; seuls les versements automatiques sont gardés, les
//!   manuels vont au compte rendu ([`StripeFetched::manual_payouts`]) ;
//! - pour chaque versement, `GET /v1/balance_transactions?payout=<po_…>&limit=100` ;
//! - pagination par curseur (`starting_after` = dernier identifiant, tant que
//!   `has_more`), sans fenêtre : Stripe ne borne pas la période d'une liste ;
//! - la traduction en lignes du rapport est celle de [`rows`].
//!
//! Erreurs (jamais le texte de Stripe, qui cite un fragment de la clé quand
//! elle est refusée, jamais l'URL ni la clé) : 401 `PROVIDER_AUTH_FAILED` ; 403
//! `PROVIDER_PERMISSION_DENIED` ; 429 et 5xx (délai et connexion coupée
//! compris) repris par [`with_retry`] puis `PROVIDER_RATE_LIMITED` ou
//! `PROVIDER_UNAVAILABLE` ; corps illisible ou trop grand
//! `PROVIDER_RESPONSE_INVALID` ; tout autre statut (400 compris)
//! `PROVIDER_API_ERROR`, `details: { provider, window, page }`. La ressource
//! (`payouts`, `balance_transactions`), `error.code` et `error.param` de Stripe
//! (bornés à [`ERROR_FIELD_MAX_CHARS`] caractères) et l'en-tête
//! `Stripe-Rate-Limited-Reason` vont au **journal du serveur**, jamais
//! `error.message` (amendement du 22.11, point 1).
//!
//! ⚠️ **Non confirmé** : le statut HTTP d'un droit manquant (400 ou 403, la
//! documentation ne le dit pas) ; le manuel d'administration (40-8b) dit de
//! vérifier les droits de la clé sur un `PROVIDER_API_ERROR`.

pub mod rows;

use std::sync::Arc;
use std::time::Duration;

use kesh_import::StripePayoutRow;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::errors::AppError;
use crate::http_client::{BodyError, build_client, read_body_bounded, retry_after};
use crate::provider_download::{
    Attempt, ProviderGate, RetryError, StatusClass, Transient, Wait, Window, classify_status,
    classify_transport, run_bounded, with_retry,
};
use crate::provider_env::{ProviderApiConfig, ProviderSeries, ProviderSpec};

use rows::{ApiBalanceTransaction, ApiPayout};

/// Le prestataire dans l'API et les journaux (`bank_accounts.provider`).
pub const PROVIDER: &str = "stripe";
/// Champ de la série : la clé restreinte (`KESH_STRIPE_<n>_KEY`).
pub const STRIPE_KEY: &str = "KEY";
/// Version de l'API épinglée, envoyée à chaque requête — relue le 2026-11-22
/// sur la page « Versioning » de la documentation de Stripe. Les champs lus
/// (`amount`, `fee`, `net`, `currency`, `created`, `available_on`,
/// `reporting_category`, `source`, `description`, `type`, `arrival_date`,
/// `automatic`, `status`) sont anciens et stables.
pub const STRIPE_VERSION: &str = "2026-11-18.endive";
/// Objets par page (maximum de l'API).
pub const PAGE_SIZE: u32 = 100;
/// Plafond de garde, en pages, d'une liste : 100 000 objets.
pub const MAX_PAGES: u32 = 1000;
/// Borne du corps d'une réponse (une page de 100 objets pèse ~150 Kio).
pub const MAX_BODY_BYTES: usize = 8 * 1024 * 1024;
/// Délai d'établissement de la connexion.
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Délai total d'une requête.
pub const TOTAL_TIMEOUT: Duration = Duration::from_secs(60);
/// Borne des champs d'erreur de Stripe écrits au journal.
pub const ERROR_FIELD_MAX_CHARS: usize = 64;
/// Ressource de la liste des versements (journal).
pub const RESOURCE_PAYOUTS: &str = "payouts";
/// Ressource de la liste des transactions de solde (journal).
pub const RESOURCE_BALANCE_TRANSACTIONS: &str = "balance_transactions";

/// Préfixe d'une clé restreinte de production.
pub const LIVE_PREFIX: &str = "rk_live_";
/// Préfixe d'une clé restreinte de test.
pub const TEST_PREFIX: &str = "rk_test_";

/// Raison du refus d'une clé secrète au démarrage.
pub const SECRET_KEY_REFUSED: &str =
    "clé secrète refusée : utilisez une clé restreinte en lecture (rk_live_… ou rk_test_…)";
/// Raison du refus de toute autre forme de clé.
pub const RESTRICTED_KEY_EXPECTED: &str =
    "clé restreinte attendue : préfixe rk_live_ ou rk_test_, suivi de lettres, chiffres ou _";

/// Stripe : une clé restreinte en lecture par compte Stripe. Inscrit à
/// [`crate::provider_env::ALL_PROVIDERS`].
pub const STRIPE: ProviderSpec = ProviderSpec {
    key: "STRIPE",
    provider: PROVIDER,
    fields: &[STRIPE_KEY],
    optional_fields: &[],
    default_base_url: "https://api.stripe.com",
    check: Some(check_field),
};

/// Le mode d'une clé, lu à son préfixe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyMode {
    /// `rk_live_…`.
    Live,
    /// `rk_test_…`.
    Test,
}

impl KeyMode {
    /// `live` ou `test` (montré à l'écran par 40-8b).
    pub fn as_str(self) -> &'static str {
        match self {
            KeyMode::Live => "live",
            KeyMode::Test => "test",
        }
    }
}

/// Le mode d'une clé **restreinte** : préfixe `rk_live_` ou `rk_test_`, suivi
/// d'au moins un caractère, lettres ASCII, chiffres ou `_` seulement (ce qui
/// exclut aussi tout caractère qui casserait un en-tête). `None` sinon.
pub fn key_mode(key: &str) -> Option<KeyMode> {
    let (mode, rest) = match key.strip_prefix(LIVE_PREFIX) {
        Some(r) => (KeyMode::Live, r),
        None => (KeyMode::Test, key.strip_prefix(TEST_PREFIX)?),
    };
    (!rest.is_empty() && rest.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_'))
        .then_some(mode)
}

/// Contrôle propre à Stripe, branché sur le lecteur du socle
/// ([`ProviderSpec::check`]) : la clé doit être restreinte ; une clé secrète
/// (`sk_…`) a son propre message. Les raisons ne citent jamais la valeur.
pub fn check_field(field: &str, value: &str) -> Result<(), &'static str> {
    if field != STRIPE_KEY || key_mode(value).is_some() {
        return Ok(());
    }
    Err(if value.starts_with("sk_") {
        SECRET_KEY_REFUSED
    } else {
        RESTRICTED_KEY_EXPECTED
    })
}

/// Une page d'une liste de Stripe.
#[derive(Debug, Deserialize)]
struct ListPage<T> {
    data: Vec<T>,
    has_more: bool,
}

/// Un objet d'une liste, repéré par son identifiant (curseur).
trait HasId {
    fn id(&self) -> &str;
}

impl HasId for ApiPayout {
    fn id(&self) -> &str {
        &self.id
    }
}

impl HasId for ApiBalanceTransaction {
    fn id(&self) -> &str {
        &self.id
    }
}

/// Le corps d'erreur de Stripe : seuls `code` et `param` sont lus (jamais
/// `message`).
#[derive(Deserialize)]
struct ErrorBody {
    error: ErrorFields,
}

#[derive(Deserialize)]
struct ErrorFields {
    code: Option<String>,
    param: Option<String>,
}

/// Un texte venu de la réponse, borné à [`ERROR_FIELD_MAX_CHARS`] caractères.
fn bounded(s: &str) -> String {
    s.chars().take(ERROR_FIELD_MAX_CHARS).collect()
}

/// `error.code` et `error.param` d'un corps d'erreur, bornés.
fn error_fields(body: &[u8]) -> (Option<String>, Option<String>) {
    match serde_json::from_slice::<ErrorBody>(body) {
        Ok(b) => (
            b.error.code.as_deref().map(bounded),
            b.error.param.as_deref().map(bounded),
        ),
        Err(_) => (None, None),
    }
}

/// Le client d'un compte Stripe.
pub struct StripeClient {
    http: reqwest::Client,
    base_url: String,
    key: String,
    gate: Arc<ProviderGate>,
    wait: Arc<dyn Wait>,
}

/// Ce qu'un téléchargement a lu.
#[derive(Debug, Default, PartialEq)]
pub struct StripeFetched {
    /// Les lignes du rapport, rangées par versement puis par transaction.
    pub rows: Vec<StripePayoutRow>,
    /// Versements **manuels** payés de la période, non lus (« versement
    /// manuel à examiner »), par identifiant.
    pub manual_payouts: Vec<String>,
}

impl StripeClient {
    /// Le client de la série `series`. `timeouts` remplace les délais (tests).
    pub fn new(
        config: &ProviderApiConfig,
        series: &ProviderSeries,
        gate: Arc<ProviderGate>,
        wait: Arc<dyn Wait>,
        timeouts: Option<(Duration, Duration)>,
    ) -> Result<Self, AppError> {
        let (connect, total) = timeouts.unwrap_or((CONNECT_TIMEOUT, TOTAL_TIMEOUT));
        let https_only = config.base_url.starts_with("https://");
        let http = build_client(https_only, connect, total).map_err(|_| {
            AppError::Internal("client HTTP de l'API du prestataire impossible à construire".into())
        })?;
        Ok(Self {
            http,
            base_url: config.base_url.clone(),
            key: config.spec.require_field(series, STRIPE_KEY)?,
            gate,
            wait,
        })
    }

    /// Le mode de la clé (`live`/`test`), `None` si elle n'est pas restreinte
    /// (impossible après la validation du démarrage).
    pub fn mode(&self) -> Option<KeyMode> {
        key_mode(&self.key)
    }

    /// Classe une erreur de transport (socle) : passagère ou définitive. Le
    /// texte de l'erreur (qui porte l'URL) n'est jamais journalisé.
    fn transport<T>(e: reqwest::Error) -> Attempt<T, AppError> {
        match classify_transport(&e) {
            Some(kind) => Attempt::Transient {
                kind,
                retry_after: None,
            },
            None => {
                tracing::warn!(
                    provider = PROVIDER,
                    connexion = e.is_connect(),
                    requete = e.is_request(),
                    "API : erreur de transport définitive (certificat, hôte ou schéma)"
                );
                Attempt::Fatal(AppError::ProviderApiError {
                    provider: PROVIDER,
                    window: None,
                    page: None,
                })
            }
        }
    }

    /// Classe un statut non 2xx par le socle ; ressource, `error.code` et
    /// `error.param` au journal.
    fn status<T>(
        status: u16,
        retry_after: Option<String>,
        rate_reason: Option<String>,
        body: &[u8],
        resource: &'static str,
    ) -> Attempt<T, AppError> {
        let (code, param) = error_fields(body);
        match classify_status(status) {
            StatusClass::Unauthorized => {
                tracing::warn!(provider = PROVIDER, ressource = resource, erreur = ?code, "API : clé refusée");
                Attempt::Fatal(AppError::ProviderAuthFailed { provider: PROVIDER })
            }
            StatusClass::Forbidden => {
                tracing::warn!(provider = PROVIDER, ressource = resource, erreur = ?code, "API : droits insuffisants");
                Attempt::Fatal(AppError::ProviderPermissionDenied { provider: PROVIDER })
            }
            StatusClass::Transient(kind) => {
                if rate_reason.is_some() {
                    tracing::info!(provider = PROVIDER, ressource = resource, motif = ?rate_reason, "API : débit limité");
                }
                Attempt::Transient { kind, retry_after }
            }
            StatusClass::Other => {
                tracing::warn!(
                    provider = PROVIDER,
                    ressource = resource,
                    statut = status,
                    erreur = ?code,
                    parametre = ?param,
                    "API : refus"
                );
                Attempt::Fatal(AppError::ProviderApiError {
                    provider: PROVIDER,
                    window: None,
                    page: None,
                })
            }
        }
    }

    /// Un essai d'une page de liste.
    async fn page_once<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&'static str, String)],
        resource: &'static str,
    ) -> Attempt<ListPage<T>, AppError> {
        let sent = self
            .http
            .get(format!("{}{path}", self.base_url))
            .query(query)
            .bearer_auth(&self.key)
            .header("Stripe-Version", STRIPE_VERSION)
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await;
        let response = match sent {
            Ok(r) => r,
            Err(e) => return Self::transport(e),
        };
        let status = response.status();
        let retry_after = retry_after(&response);
        let rate_reason = response
            .headers()
            .get("Stripe-Rate-Limited-Reason")
            .and_then(|v| v.to_str().ok())
            .map(bounded);
        let body = match read_body_bounded(response, MAX_BODY_BYTES).await {
            Ok(b) => b,
            Err(BodyError::TooLarge) => {
                return Attempt::Fatal(AppError::ProviderResponseInvalid {
                    provider: PROVIDER,
                    details: serde_json::json!({ "reason": "too_large", "resource": resource }),
                });
            }
            Err(BodyError::Transport(e)) => return Self::transport(e),
        };
        if !status.is_success() {
            return Self::status(status.as_u16(), retry_after, rate_reason, &body, resource);
        }
        match serde_json::from_slice::<ListPage<T>>(&body) {
            Ok(p) => Attempt::Done(p),
            Err(_) => Attempt::Fatal(AppError::ProviderResponseInvalid {
                provider: PROVIDER,
                details: serde_json::json!({ "reason": "page", "resource": resource }),
            }),
        }
    }

    /// Toutes les pages d'une liste, reprise comprise, curseur
    /// `starting_after` tant que `has_more`. `window` nomme la période dans les
    /// erreurs ; une page `has_more` vide ou un curseur qui ne bouge pas rend
    /// la réponse invalide (aucune boucle sans fin), comme plus de
    /// [`MAX_PAGES`] pages.
    async fn list<T: DeserializeOwned + HasId>(
        &self,
        path: &str,
        base: Vec<(&'static str, String)>,
        resource: &'static str,
        window: &str,
    ) -> Result<Vec<T>, AppError> {
        let mut out: Vec<T> = Vec::new();
        let mut cursor: Option<String> = None;
        for page in 1..=MAX_PAGES {
            let mut query = base.clone();
            query.push(("limit", PAGE_SIZE.to_string()));
            if let Some(c) = &cursor {
                query.push(("starting_after", c.clone()));
            }
            let outcome = with_retry(&self.gate, self.wait.as_ref(), || {
                self.page_once::<T>(path, &query, resource)
            })
            .await;
            let p = match outcome {
                Ok(p) => p,
                Err(RetryError::Fatal(e)) => return Err(locate(e, window, page)),
                Err(RetryError::Exhausted(kind)) => return Err(exhausted(kind, window, page)),
            };
            let next = p.data.last().map(|t| t.id().to_string());
            out.extend(p.data);
            if !p.has_more {
                return Ok(out);
            }
            match next {
                Some(n) if cursor.as_deref() != Some(n.as_str()) => cursor = Some(n),
                _ => {
                    return Err(AppError::ProviderResponseInvalid {
                        provider: PROVIDER,
                        details: serde_json::json!({ "reason": "cursor", "resource": resource }),
                    });
                }
            }
        }
        Err(AppError::ProviderResponseInvalid {
            provider: PROVIDER,
            details: serde_json::json!({ "reason": "too_many_pages", "resource": resource }),
        })
    }

    /// Les versements **payés** dont l'arrivée tombe dans la fenêtre
    /// (`arrival_date` dans `[start, end[`, en secondes d'époque ;
    /// [`Window::new`] borne aux jours de Zurich).
    pub async fn payouts(&self, w: &Window) -> Result<Vec<ApiPayout>, AppError> {
        let query = vec![
            ("arrival_date[gte]", w.start.timestamp().to_string()),
            ("arrival_date[lt]", w.end.timestamp().to_string()),
            ("status", "paid".to_string()),
        ];
        self.list("/v1/payouts", query, RESOURCE_PAYOUTS, &w.label())
            .await
    }

    /// Les transactions de solde d'un versement automatique.
    pub async fn balance_transactions(
        &self,
        payout_id: &str,
        w: &Window,
    ) -> Result<Vec<ApiBalanceTransaction>, AppError> {
        let query = vec![("payout", payout_id.to_string())];
        self.list(
            "/v1/balance_transactions",
            query,
            RESOURCE_BALANCE_TRANSACTIONS,
            &w.label(),
        )
        .await
    }
}

/// L'échec après six essais passagers, nommant la période et la page.
fn exhausted(kind: Transient, window: &str, page: u32) -> AppError {
    match kind {
        Transient::RateLimited => AppError::ProviderRateLimited {
            provider: PROVIDER,
            window: Some(window.to_string()),
            page: Some(page),
        },
        Transient::Unavailable => AppError::ProviderUnavailable {
            provider: PROVIDER,
            window: Some(window.to_string()),
            page: Some(page),
        },
    }
}

/// Ajoute la période et la page à une erreur d'API qui ne les porte pas.
fn locate(e: AppError, window: &str, page: u32) -> AppError {
    match e {
        AppError::ProviderApiError { provider, .. } => AppError::ProviderApiError {
            provider,
            window: Some(window.to_string()),
            page: Some(page),
        },
        other => other,
    }
}

/// Lit une période : la liste des versements payés, puis les transactions de
/// chaque versement **automatique**, au plus `parallel` versements à la fois
/// ([`run_bounded`] ; le sémaphore du prestataire borne en plus les requêtes en
/// vol), et rend les lignes du rapport ([`rows::rows_of`]). Un versement
/// manuel n'est pas lu : il va à [`StripeFetched::manual_payouts`]. Le premier
/// échec arrête tout ; rien n'est rendu.
pub async fn fetch_rows(
    client: Arc<StripeClient>,
    w: Window,
    parallel: usize,
) -> Result<StripeFetched, AppError> {
    let payouts = client.payouts(&w).await?;
    let mut manual_payouts = Vec::new();
    let mut automatic = Vec::new();
    for p in payouts {
        // `status=paid` est demandé ; un autre statut ne se lit pas (défense).
        if p.status != "paid" {
            continue;
        }
        if p.automatic {
            automatic.push(p);
        } else {
            manual_payouts.push(p.id);
        }
    }
    manual_payouts.sort();
    let works = automatic.into_iter().map(move |p| {
        let client = client.clone();
        async move {
            let txs = client.balance_transactions(&p.id, &w).await?;
            Ok((p, txs))
        }
    });
    let parts = run_bounded(works, parallel).await?;
    Ok(StripeFetched {
        rows: rows::rows_of(&parts)?,
        manual_payouts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_d_une_cle() {
        assert_eq!(key_mode("rk_live_Ab12_x"), Some(KeyMode::Live));
        assert_eq!(key_mode("rk_test_Ab12"), Some(KeyMode::Test));
        assert_eq!(KeyMode::Live.as_str(), "live");
        assert_eq!(KeyMode::Test.as_str(), "test");
        for bad in [
            "rk_live_",
            "rk_test_",
            "sk_live_abc",
            "pk_test_abc",
            "rk_prod_abc",
            "rk_test_a b",
            "rk_test_a\nb",
            "rk_test_é",
            "RK_TEST_abc",
        ] {
            assert_eq!(key_mode(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn controle_du_champ() {
        assert_eq!(check_field(STRIPE_KEY, "rk_live_x1"), Ok(()));
        assert_eq!(
            check_field(STRIPE_KEY, "sk_live_x1"),
            Err(SECRET_KEY_REFUSED)
        );
        assert_eq!(
            check_field(STRIPE_KEY, "sk_test_x1"),
            Err(SECRET_KEY_REFUSED)
        );
        assert_eq!(
            check_field(STRIPE_KEY, "pk_live_x1"),
            Err(RESTRICTED_KEY_EXPECTED)
        );
        // Le libellé n'est pas contrôlé ici.
        assert_eq!(check_field("ACCOUNT", "sk_live_x1"), Ok(()));
    }

    #[test]
    fn champs_d_erreur_bornes_sans_message() {
        let (code, param) = error_fields(
            br#"{"error":{"code":"parameter_unknown","param":"x","message":"rk_test_abc"}}"#,
        );
        assert_eq!(code.as_deref(), Some("parameter_unknown"));
        assert_eq!(param.as_deref(), Some("x"));
        let long = format!(r#"{{"error":{{"code":"{}"}}}}"#, "é".repeat(500));
        assert_eq!(
            error_fields(long.as_bytes()).0.unwrap().chars().count(),
            ERROR_FIELD_MAX_CHARS
        );
        assert_eq!(error_fields(b"<html>"), (None, None));
    }
}
