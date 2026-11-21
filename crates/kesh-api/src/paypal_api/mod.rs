//! Client de l'API PayPal « Transaction Search » (Story 38-12a, AC-4 à AC-6).
//!
//! Ce qui est propre à PayPal, le socle faisant le reste
//! ([`crate::provider_download`], [`crate::http_client`]) :
//!
//! - **jeton** OAuth (`POST /v1/oauth2/token`, authentification Basic
//!   `client_id:secret`, `grant_type=client_credentials`), **un par tâche**,
//!   demandé au début, renouvelé à l'approche de `expires_in` et une fois sur
//!   401, puis `PROVIDER_AUTH_FAILED` ; aucun cache partagé ;
//! - **pages** de 500 (`GET /v1/reporting/transactions`, `fields=all`,
//!   `balance_affecting_records_only=Y`, `page` de 1 à `total_pages`), en-tête
//!   `PayPal-Enforce-ISO8601-Format: true` ;
//! - sur `400 RESULTSET_TOO_LARGE`, la fenêtre se coupe en deux et chaque
//!   moitié est reprise (plancher d'un jour, puis `PROVIDER_API_ERROR`) ;
//! - corps borné à 16 Mio, délai total de 60 s par requête.
//!
//! Statuts, classés par le socle ([`classify_status`], [`classify_transport`]) :
//! 401 (renouvelé une fois), 403 `PROVIDER_PERMISSION_DENIED`, 429 et 5xx
//! (délai et connexion refusée ou coupée compris) repris par [`with_retry`],
//! erreur de transport définitive (certificat, hôte inconnu)
//! `PROVIDER_API_ERROR` sans reprise, corps illisible ou trop grand
//! `PROVIDER_RESPONSE_INVALID`, le reste `PROVIDER_API_ERROR`. Ni le texte de
//! PayPal ni l'URL ne sortent : le **nom** d'erreur de PayPal
//! (`NOT_AUTHORIZED`…), borné à [`ERROR_NAME_MAX_CHARS`] caractères, va au
//! journal du serveur, sans secret.
//!
//! Relevés du 2026-11-20 sur un compte réel (lecture seule) : l'hôte
//! `api-m.paypal.com` répond pour un compte réel ; le jeton porte la portée
//! `…/services/reporting/search/read` et `expires_in` = 32 400 s ; une fenêtre
//! de 31 jours plus une heure est acceptée (la limite n'est pas mesurée en
//! jours de 24 h stricts : les fenêtres de 30 jours de Zurich gardent leur
//! marge) ; aucune limitation de débit n'a été rencontrée sur 33 requêtes.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use kesh_import::paypal_api::{PaypalApiPage, PaypalApiTx, parse_transactions_page};
use serde::Deserialize;

use crate::errors::AppError;
use crate::http_client::{BodyError, build_client, read_body_bounded, retry_after};
use crate::provider_download::{
    Attempt, JobHandle, ProviderGate, RetryError, StatusClass, Transient, Wait, Window,
    classify_status, classify_transport, run_bounded, with_retry,
};
use crate::provider_env::{PAYPAL_CLIENT_ID, PAYPAL_SECRET, ProviderApiConfig, ProviderSeries};

/// Le prestataire.
pub const PROVIDER: &str = "paypal";
/// Jours de Zurich par fenêtre (limite de PayPal : 31 jours, unité non dite).
pub const MAX_DAYS: u32 = 30;
/// Profondeur de l'historique de l'API, en années.
pub const HISTORY_YEARS: u32 = 3;
/// Transactions par page (maximum de l'API).
pub const PAGE_SIZE: u32 = 500;
/// Borne du corps d'une réponse (500 transactions en `fields=all`).
pub const MAX_BODY_BYTES: usize = 16 * 1024 * 1024;
/// Délai d'établissement de la connexion.
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Délai total d'une requête (une page de 500 transactions).
pub const TOTAL_TIMEOUT: Duration = Duration::from_secs(60);
/// Le jeton se renouvelle quand il lui reste moins que cette marge.
const TOKEN_MARGIN: Duration = Duration::from_secs(60);
/// Nom d'erreur de PayPal d'une fenêtre trop chargée.
const RESULTSET_TOO_LARGE: &str = "RESULTSET_TOO_LARGE";
/// Borne du nom d'erreur de PayPal écrit au journal : il vient de la réponse,
/// qui peut peser 16 Mio (revue P1, F10).
pub const ERROR_NAME_MAX_CHARS: usize = 64;

/// Échec définitif d'un essai.
#[derive(Debug)]
enum Fatal {
    /// 401 : le jeton est refusé.
    Unauthorized,
    /// `400 RESULTSET_TOO_LARGE`.
    TooLarge,
    /// Toute autre erreur, déjà traduite.
    App(AppError),
}

/// Un jeton et son échéance.
struct Token {
    value: String,
    expires_at: Instant,
}

/// Le client d'une tâche : un compte PayPal, un jeton.
pub struct PaypalClient {
    http: reqwest::Client,
    base_url: String,
    client_id: String,
    secret: String,
    gate: Arc<ProviderGate>,
    wait: Arc<dyn Wait>,
    token: tokio::sync::Mutex<Option<Token>>,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: Option<u64>,
}

#[derive(Deserialize)]
struct ErrorName {
    name: Option<String>,
}

/// Le nom d'erreur de PayPal d'un corps, s'il s'y lit, borné à
/// [`ERROR_NAME_MAX_CHARS`] caractères.
fn error_name(body: &[u8]) -> Option<String> {
    let name = serde_json::from_slice::<ErrorName>(body).ok()?.name?;
    Some(name.chars().take(ERROR_NAME_MAX_CHARS).collect())
}

/// Bornes d'une fenêtre au format de l'API (UTC, à la seconde).
fn api_instant(t: DateTime<Utc>) -> String {
    t.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

impl PaypalClient {
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
            client_id: config.spec.require_field(series, PAYPAL_CLIENT_ID)?,
            secret: config.spec.require_field(series, PAYPAL_SECRET)?,
            gate,
            wait,
            token: tokio::sync::Mutex::new(None),
        })
    }

    /// Classe une erreur de transport par le socle ([`classify_transport`]) :
    /// délai, connexion refusée ou coupée sont passagers ; le reste —
    /// certificat refusé, hôte inconnu — est définitif, `PROVIDER_API_ERROR`
    /// sans reprise. Le texte de l'erreur (qui porte l'URL) n'est jamais
    /// journalisé.
    fn transport<T>(e: reqwest::Error) -> Attempt<T, Fatal> {
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
                Attempt::Fatal(Fatal::App(AppError::ProviderApiError {
                    provider: PROVIDER,
                    window: None,
                    page: None,
                }))
            }
        }
    }

    /// Classe un statut non 2xx commun aux deux requêtes, par le socle
    /// ([`classify_status`]) ; `400 RESULTSET_TOO_LARGE` est propre à PayPal.
    fn status<T>(
        status: reqwest::StatusCode,
        retry_after: Option<String>,
        body: &[u8],
    ) -> Attempt<T, Fatal> {
        let name = error_name(body);
        match classify_status(status.as_u16()) {
            StatusClass::Unauthorized => Attempt::Fatal(Fatal::Unauthorized),
            StatusClass::Forbidden => {
                tracing::warn!(provider = PROVIDER, erreur = ?name, "API : droits insuffisants");
                Attempt::Fatal(Fatal::App(AppError::ProviderPermissionDenied {
                    provider: PROVIDER,
                }))
            }
            StatusClass::Transient(kind) => Attempt::Transient { kind, retry_after },
            StatusClass::Other
                if status.as_u16() == 400 && name.as_deref() == Some(RESULTSET_TOO_LARGE) =>
            {
                Attempt::Fatal(Fatal::TooLarge)
            }
            StatusClass::Other => {
                tracing::warn!(provider = PROVIDER, statut = status.as_u16(), erreur = ?name, "API : refus");
                Attempt::Fatal(Fatal::App(AppError::ProviderApiError {
                    provider: PROVIDER,
                    window: None,
                    page: None,
                }))
            }
        }
    }

    /// Lit le corps borné d'une réponse ; trop grand ou illisible → réponse
    /// invalide ; délai pendant la lecture → passager.
    async fn body<T>(response: reqwest::Response) -> Result<Vec<u8>, Attempt<T, Fatal>> {
        read_body_bounded(response, MAX_BODY_BYTES)
            .await
            .map_err(|e| match e {
                BodyError::TooLarge => {
                    Attempt::Fatal(Fatal::App(AppError::ProviderResponseInvalid {
                        provider: PROVIDER,
                        details: serde_json::json!({ "reason": "too_large" }),
                    }))
                }
                BodyError::Transport(e) => Self::transport(e),
            })
    }

    /// Un essai de la demande de jeton. Ce `POST` est **rejouable sans
    /// danger** : il ne crée aucune ressource durable (un jeton de plus), ce
    /// qui autorise la reprise commune des coupures après envoi
    /// ([`crate::http_client::is_transient_io_chain`]).
    async fn token_once(&self) -> Attempt<Token, Fatal> {
        let sent = self
            .http
            .post(format!("{}/v1/oauth2/token", self.base_url))
            .basic_auth(&self.client_id, Some(&self.secret))
            .header(reqwest::header::ACCEPT, "application/json")
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body("grant_type=client_credentials")
            .send()
            .await;
        let response = match sent {
            Ok(r) => r,
            Err(e) => return Self::transport(e),
        };
        let status = response.status();
        let retry_after = retry_after(&response);
        let body = match Self::body(response).await {
            Ok(b) => b,
            Err(a) => return a,
        };
        if !status.is_success() {
            return Self::status(status, retry_after, &body);
        }
        match serde_json::from_slice::<TokenResponse>(&body) {
            Ok(t) if !t.access_token.is_empty() => Attempt::Done(Token {
                value: t.access_token,
                expires_at: Instant::now() + Duration::from_secs(t.expires_in.unwrap_or(300)),
            }),
            _ => Attempt::Fatal(Fatal::App(AppError::ProviderResponseInvalid {
                provider: PROVIDER,
                details: serde_json::json!({ "reason": "token" }),
            })),
        }
    }

    /// Demande un jeton neuf (reprise comprise) et le garde.
    async fn renew(&self) -> Result<String, AppError> {
        let token = match with_retry(&self.gate, self.wait.as_ref(), || self.token_once()).await {
            Ok(t) => t,
            Err(RetryError::Fatal(Fatal::Unauthorized))
            | Err(RetryError::Fatal(Fatal::TooLarge)) => {
                return Err(AppError::ProviderAuthFailed { provider: PROVIDER });
            }
            Err(RetryError::Fatal(Fatal::App(e))) => return Err(e),
            Err(RetryError::Exhausted(kind)) => return Err(exhausted(kind, None, None)),
        };
        let value = token.value.clone();
        *self.token.lock().await = Some(token);
        Ok(value)
    }

    /// Le jeton courant, renouvelé à l'approche de son échéance.
    async fn current_token(&self) -> Result<String, AppError> {
        {
            let guard = self.token.lock().await;
            if let Some(t) = guard.as_ref()
                && t.expires_at.saturating_duration_since(Instant::now()) > TOKEN_MARGIN
            {
                return Ok(t.value.clone());
            }
        }
        self.renew().await
    }

    /// Un essai d'une page.
    async fn page_once(&self, token: &str, w: &Window, page: u32) -> Attempt<PaypalApiPage, Fatal> {
        let url = format!(
            "{}/v1/reporting/transactions?start_date={}&end_date={}&fields=all&page_size={PAGE_SIZE}&page={page}&balance_affecting_records_only=Y",
            self.base_url,
            api_instant(w.start).replace(':', "%3A"),
            api_instant(w.end).replace(':', "%3A"),
        );
        let sent = self
            .http
            .get(url)
            .bearer_auth(token)
            .header(reqwest::header::ACCEPT, "application/json")
            .header("PayPal-Enforce-ISO8601-Format", "true")
            .send()
            .await;
        let response = match sent {
            Ok(r) => r,
            Err(e) => return Self::transport(e),
        };
        let status = response.status();
        let retry_after = retry_after(&response);
        let body = match Self::body(response).await {
            Ok(b) => b,
            Err(a) => return a,
        };
        if !status.is_success() {
            return Self::status(status, retry_after, &body);
        }
        match parse_transactions_page(&body) {
            Ok(p) => Attempt::Done(p),
            Err(_) => Attempt::Fatal(Fatal::App(AppError::ProviderResponseInvalid {
                provider: PROVIDER,
                details: serde_json::json!({ "reason": "page" }),
            })),
        }
    }

    /// Une page, reprise comprise, le jeton renouvelé une fois sur 401. `None`
    /// pour `RESULTSET_TOO_LARGE`.
    async fn page(&self, w: &Window, page: u32) -> Result<Option<PaypalApiPage>, AppError> {
        let mut renewed = false;
        loop {
            let token = self.current_token().await?;
            let outcome = with_retry(&self.gate, self.wait.as_ref(), || {
                self.page_once(&token, w, page)
            })
            .await;
            return match outcome {
                Ok(p) => Ok(Some(p)),
                Err(RetryError::Fatal(Fatal::TooLarge)) => Ok(None),
                Err(RetryError::Fatal(Fatal::Unauthorized)) if !renewed => {
                    renewed = true;
                    self.renew().await?;
                    continue;
                }
                Err(RetryError::Fatal(Fatal::Unauthorized)) => {
                    Err(AppError::ProviderAuthFailed { provider: PROVIDER })
                }
                Err(RetryError::Fatal(Fatal::App(e))) => Err(locate(e, w, page)),
                Err(RetryError::Exhausted(kind)) => Err(exhausted(kind, Some(w), Some(page))),
            };
        }
    }

    /// Demande le jeton de la tâche (au début, avant toute fenêtre).
    pub async fn authenticate(&self) -> Result<(), AppError> {
        self.renew().await.map(|_| ())
    }
}

/// L'échec après six essais passagers, nommant la fenêtre et la page.
fn exhausted(kind: Transient, w: Option<&Window>, page: Option<u32>) -> AppError {
    let window = w.map(Window::label);
    match kind {
        Transient::RateLimited => AppError::ProviderRateLimited {
            provider: PROVIDER,
            window,
            page,
        },
        Transient::Unavailable => AppError::ProviderUnavailable {
            provider: PROVIDER,
            window,
            page,
        },
    }
}

/// Ajoute la fenêtre et la page à une erreur d'API qui ne les porte pas.
fn locate(e: AppError, w: &Window, page: u32) -> AppError {
    match e {
        AppError::ProviderApiError { provider, .. } => AppError::ProviderApiError {
            provider,
            window: Some(w.label()),
            page: Some(page),
        },
        other => other,
    }
}

/// Ce qu'un téléchargement a lu : les transactions, rangées par fenêtre puis
/// par page, et le plus ancien « données à jour jusqu'à » des pages.
#[derive(Debug, Default)]
pub struct Fetched {
    /// Transactions, dans l'ordre des fenêtres et des pages.
    pub transactions: Vec<PaypalApiTx>,
    /// Le plus ancien `last_refreshed_datetime` (RFC 3339).
    pub last_refreshed: Option<String>,
}

impl Fetched {
    fn absorb(&mut self, other: Fetched) {
        self.transactions.extend(other.transactions);
        self.last_refreshed = oldest(self.last_refreshed.take(), other.last_refreshed);
    }
}

/// Le plus ancien de deux instants RFC 3339 (un illisible est ignoré).
fn oldest(a: Option<String>, b: Option<String>) -> Option<String> {
    let parse = |s: &Option<String>| {
        s.as_deref()
            .and_then(|v| DateTime::parse_from_rfc3339(v).ok())
    };
    match (parse(&a), parse(&b)) {
        (Some(x), Some(y)) => {
            if y < x {
                b
            } else {
                a
            }
        }
        (Some(_), None) => a,
        (None, Some(_)) => b,
        (None, None) => a.or(b),
    }
}

type WindowFuture = Pin<Box<dyn Future<Output = Result<Fetched, AppError>> + Send>>;

/// Lit une fenêtre : ses pages une à une ; coupée en deux sur
/// `RESULTSET_TOO_LARGE` (une étape de plus), jusqu'au jour.
fn fetch_window(
    client: Arc<PaypalClient>,
    w: Window,
    now: DateTime<Utc>,
    job: JobHandle,
) -> WindowFuture {
    Box::pin(async move {
        let mut fetched = Fetched::default();
        let first = match client.page(&w, 1).await? {
            Some(p) => p,
            None => {
                let Some((a, b)) = w.split(now) else {
                    return Err(AppError::ProviderApiError {
                        provider: PROVIDER,
                        window: Some(w.label()),
                        page: Some(1),
                    });
                };
                job.add_steps(1);
                fetched.absorb(fetch_window(client.clone(), a, now, job.clone()).await?);
                // Une moitié de durée nulle (première seconde du jour) ne part
                // pas chez PayPal (revue P3, P3-1).
                if !b.is_empty() {
                    fetched.absorb(fetch_window(client, b, now, job).await?);
                }
                return Ok(fetched);
            }
        };
        let total_pages = first.total_pages;
        fetched.last_refreshed = first.last_refreshed_datetime;
        fetched.transactions = first.transactions;
        for page in 2..=total_pages {
            match client.page(&w, page).await? {
                Some(p) => fetched.absorb(Fetched {
                    transactions: p.transactions,
                    last_refreshed: p.last_refreshed_datetime,
                }),
                None => {
                    return Err(AppError::ProviderApiError {
                        provider: PROVIDER,
                        window: Some(w.label()),
                        page: Some(page),
                    });
                }
            }
        }
        job.step_done();
        Ok(fetched)
    })
}

/// Télécharge les fenêtres — **au plus `parallel` à la fois**
/// ([`run_bounded`] : un seul fil si `parallel` vaut 1, jamais une tâche par
/// fenêtre ; le sémaphore du prestataire borne en plus les requêtes en vol de
/// tous ses téléchargements) ; résultats rangés par fenêtre puis par page. Le
/// premier échec arrête tout.
pub async fn download(
    client: Arc<PaypalClient>,
    windows: Vec<Window>,
    parallel: usize,
    now: DateTime<Utc>,
    job: JobHandle,
) -> Result<Fetched, AppError> {
    client.authenticate().await?;
    let works = windows
        .into_iter()
        .map(|w| fetch_window(client.clone(), w, now, job.clone()));
    let mut all = Fetched::default();
    for part in run_bounded(works, parallel).await? {
        all.absorb(part);
    }
    Ok(all)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_plus_ancien_rafraichissement() {
        let a = Some("2024-02-01T08:00:00+00:00".to_string());
        let b = Some("2024-02-01T07:00:00+00:00".to_string());
        assert_eq!(oldest(a.clone(), b.clone()), b);
        assert_eq!(oldest(b.clone(), a.clone()), b);
        assert_eq!(oldest(None, a.clone()), a);
        assert_eq!(oldest(Some("x".into()), a.clone()), a);
    }

    #[test]
    fn bornes_au_format_de_l_api() {
        use chrono::TimeZone;
        let t = Utc.with_ymd_and_hms(2024, 1, 30, 22, 59, 59).unwrap();
        assert_eq!(api_instant(t), "2024-01-30T22:59:59Z");
    }

    #[test]
    fn nom_d_erreur_de_paypal() {
        assert_eq!(
            error_name(br#"{"name":"RESULTSET_TOO_LARGE","message":"x"}"#).as_deref(),
            Some(RESULTSET_TOO_LARGE)
        );
        assert_eq!(error_name(b"<html>"), None);
        // Revue P1, F10 : un nom démesuré est borné avant le journal.
        let long = format!(r#"{{"name":"{}"}}"#, "É".repeat(10_000));
        assert_eq!(
            error_name(long.as_bytes()).unwrap().chars().count(),
            ERROR_NAME_MAX_CHARS
        );
    }
}
