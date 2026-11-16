//! La source des cours (Story 37-2, AC-3) : le trait [`RateSource`], le client
//! HTTP de production [`HttpRateSource`] et la source factice
//! [`MockRateSource`] des tests et du mode test.
//!
//! La source est tenue pour **hostile** : URL de production constante (aucune
//! variable d'environnement ne la change), HTTPS seul, aucune redirection
//! suivie, aucun magasin de cookies, réponse bornée à 1 Mio — `Content-Length`
//! contrôlé quand il est présent, puis lecture par morceaux bornée (la source
//! réelle n'envoie pas de `Content-Length`, relevé du 2026-11-16). Les erreurs
//! se rendent en **codes classés** ([`SourceError::code`]), jamais par le texte
//! d'une erreur `reqwest`, qui porte l'URL.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{Datelike, NaiveDate};
use kesh_core::exchange_rates::{MAX_BODY_BYTES, RateKind, RateUnit};

/// Future renvoyée par les méthodes de [`RateSource`] (objet-safe sans
/// `async_trait`, comme `mail::MailFuture`) : le corps de la réponse.
pub type RateFuture<'a> = Pin<Box<dyn Future<Output = Result<String, SourceError>> + Send + 'a>>;

/// URL de base de la source de production (AFC / OFDF). Constante : aucune
/// variable d'environnement ne la change (AC-3).
pub const BASE_URL_PRODUCTION: &str = "https://www.backend-rates.bazg.admin.ch";

/// Délai d'établissement de la connexion.
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Délai total d'une requête (corps compris).
pub const TOTAL_TIMEOUT: Duration = Duration::from_secs(20);

/// Échec d'un appel à la source. Chaque variante a son code classé, écrit dans
/// `_kesh_exchange_rate_fetch.last_error` ; [`SourceError::NotPublished`] est
/// le seul qui ne soit pas un échec (404 de `xmldaily`, traité comme la racine
/// vide).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SourceError {
    /// Délai dépassé (connexion ou requête).
    #[error("délai dépassé")]
    Timeout,
    /// Connexion impossible ou interrompue.
    #[error("connexion impossible")]
    Connect,
    /// Statut HTTP non 2xx (une redirection comprise : elle n'est pas suivie).
    #[error("statut HTTP {0}")]
    HttpStatus(u16),
    /// Corps au-delà de 1 Mio.
    #[error("réponse trop volumineuse")]
    TooLarge,
    /// Corps qui n'est pas de l'UTF-8.
    #[error("réponse illisible")]
    Malformed,
    /// `xmldaily` a répondu 404 : pas de publication pour cette date.
    #[error("pas encore publié")]
    NotPublished,
}

impl SourceError {
    /// Code classé : `timeout`, `connect`, `http_status:<n>`, `too_large`,
    /// `malformed`, `not_published`.
    pub fn code(&self) -> String {
        match self {
            Self::Timeout => "timeout".into(),
            Self::Connect => "connect".into(),
            Self::HttpStatus(n) => format!("http_status:{n}"),
            Self::TooLarge => "too_large".into(),
            Self::Malformed => "malformed".into(),
            Self::NotPublished => "not_published".into(),
        }
    }

    /// L'inverse de [`SourceError::code`] (endpoint de test qui pilote la
    /// source factice). `None` pour un code inconnu.
    pub fn from_code(code: &str) -> Option<Self> {
        Some(match code {
            "timeout" => Self::Timeout,
            "connect" => Self::Connect,
            "too_large" => Self::TooLarge,
            "malformed" => Self::Malformed,
            "not_published" => Self::NotPublished,
            autre => {
                let n = autre.strip_prefix("http_status:")?.parse::<u16>().ok()?;
                if !(100..=599).contains(&n) {
                    return None;
                }
                Self::HttpStatus(n)
            }
        })
    }
}

/// La source des cours, portée par `AppState` (`Arc<dyn RateSource>`).
pub trait RateSource: Send + Sync {
    /// Cours mensuels du mois (`xmlavgmonth?j=<year>&m=<month>`).
    fn fetch_month(&self, year: i32, month: u32) -> RateFuture<'_>;
    /// Cours journaliers de la date (`xmldaily?d=AAAAMMJJ`) ; un 404 rend
    /// [`SourceError::NotPublished`].
    fn fetch_day(&self, date: NaiveDate) -> RateFuture<'_>;
}

// ---------------------------------------------------------------------------
// Client HTTP
// ---------------------------------------------------------------------------

/// Le client HTTP de production.
pub struct HttpRateSource {
    client: reqwest::Client,
    base_url: String,
    https_only: bool,
}

/// Construit le client : rustls, aucune redirection, délais, `User-Agent`
/// sans donnée personnelle, aucun magasin de cookies (aucun n'est activé).
fn client(
    https_only: bool,
    connect: Duration,
    total: Duration,
) -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .use_rustls_tls()
        .redirect(reqwest::redirect::Policy::none())
        .https_only(https_only)
        .connect_timeout(connect)
        .timeout(total)
        .user_agent(concat!("Kesh/", env!("CARGO_PKG_VERSION")))
        .build()
}

impl HttpRateSource {
    /// La source de production : [`BASE_URL_PRODUCTION`], HTTPS seul.
    pub fn new() -> Result<Self, reqwest::Error> {
        Ok(Self {
            client: client(true, CONNECT_TIMEOUT, TOTAL_TIMEOUT)?,
            base_url: BASE_URL_PRODUCTION.to_string(),
            https_only: true,
        })
    }

    /// Source sur une autre URL de base, HTTP admis — **tests seulement**
    /// (serveur axum local). Jamais appelée par le code de production.
    #[doc(hidden)]
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        Self {
            client: client(false, CONNECT_TIMEOUT, TOTAL_TIMEOUT).expect("client HTTP de test"),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            https_only: false,
        }
    }

    /// Mêmes réglages avec d'autres délais — **tests seulement** (un délai de
    /// 20 s ralentirait la suite).
    #[doc(hidden)]
    pub fn with_timeouts(mut self, connect: Duration, total: Duration) -> Self {
        self.client = client(self.https_only, connect, total).expect("client HTTP de test");
        self
    }

    /// Une requête `GET` : statut contrôlé, corps borné, UTF-8.
    async fn get(&self, url: String, kind: RateKind) -> Result<String, SourceError> {
        let mut reponse = self.client.get(&url).send().await.map_err(classer)?;
        let statut = reponse.status();
        if !statut.is_success() {
            // Seul `xmldaily` rend 404 pour « pas de publication » (relevé du
            // 2026-11-14) ; un 404 de `xmlavgmonth` reste une erreur.
            if kind == RateKind::Day && statut == reqwest::StatusCode::NOT_FOUND {
                return Err(SourceError::NotPublished);
            }
            return Err(SourceError::HttpStatus(statut.as_u16()));
        }
        if reponse
            .content_length()
            .is_some_and(|n| n > MAX_BODY_BYTES as u64)
        {
            return Err(SourceError::TooLarge);
        }
        let mut corps: Vec<u8> = Vec::new();
        while let Some(morceau) = reponse.chunk().await.map_err(classer)? {
            if corps.len() + morceau.len() > MAX_BODY_BYTES {
                return Err(SourceError::TooLarge);
            }
            corps.extend_from_slice(&morceau);
        }
        String::from_utf8(corps).map_err(|_| SourceError::Malformed)
    }
}

/// Classe une erreur `reqwest` : délai ou connexion — jamais son texte.
fn classer(e: reqwest::Error) -> SourceError {
    if e.is_timeout() {
        SourceError::Timeout
    } else {
        SourceError::Connect
    }
}

impl RateSource for HttpRateSource {
    fn fetch_month(&self, year: i32, month: u32) -> RateFuture<'_> {
        let url = format!("{}/api/xmlavgmonth?j={year}&m={month}", self.base_url);
        Box::pin(self.get(url, RateKind::Month))
    }

    fn fetch_day(&self, date: NaiveDate) -> RateFuture<'_> {
        let url = format!("{}/api/xmldaily?d={}", self.base_url, date.format("%Y%m%d"));
        Box::pin(self.get(url, RateKind::Day))
    }
}

// ---------------------------------------------------------------------------
// Source factice
// ---------------------------------------------------------------------------

/// Cours mensuels du XML synthétique **par défaut** de [`MockRateSource`] :
/// `(devise, unité publiée, cours publié)`. Valeurs inventées, fixées ici et
/// lues par les specs E2E (37-3b comprise) pour leurs assertions.
pub const MOCK_MONTHLY_RATES: &[(&str, i32, &str)] = &[
    ("USD", 1, "0.8800"),
    ("EUR", 1, "0.9300"),
    ("GBP", 1, "1.1200"),
    ("JPY", 100, "0.5800"),
    ("SEK", 100, "8.4000"),
];

/// Cours journaliers du XML synthétique par défaut — **distincts** des
/// mensuels (37-5 distingue ainsi le repli sur le cours mensuel).
pub const MOCK_DAILY_RATES: &[(&str, i32, &str)] = &[
    ("USD", 1, "0.89120"),
    ("EUR", 1, "0.94450"),
    ("GBP", 1, "1.12340"),
    ("JPY", 100, "0.58120"),
    ("SEK", 100, "8.41230"),
];

/// Les `<devise>` d'un XML synthétique.
fn devises_xml(rates: &[(&str, i32, &str)], journalier: bool) -> String {
    rates
        .iter()
        .map(|(code, unite, cours)| {
            let en = if journalier { "<land_en/>" } else { "" };
            format!(
                "<devise code=\"{}\"><land_de/><land_fr/><land_it/>{en}<waehrung>{unite} {code}</waehrung><kurs>{cours}</kurs></devise>",
                code.to_ascii_lowercase()
            )
        })
        .collect()
}

/// Un XML synthétique de `xmlavgmonth` pour le mois `(year, month)`.
pub fn monthly_xml(year: i32, month: u32, rates: &[(&str, i32, &str)]) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><monatsmittelkurs xmlns=\"https://www.backend-rates.bazg.admin.ch/avgratesmonth\"><monat>{year:04}-{month:02}</monat>{}</monatsmittelkurs>",
        devises_xml(rates, false)
    )
}

/// Un XML synthétique de `xmldaily` daté du `date` (`<datum>`).
pub fn daily_xml(date: NaiveDate, rates: &[(&str, i32, &str)]) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><wechselkurse xmlns=\"https://www.backend-rates.bazg.admin.ch/xmldaily\"><datum>{}</datum><zeit>04:15:03</zeit><gueltigkeit>{}</gueltigkeit>{}</wechselkurse>",
        date.format("%d.%m.%Y"),
        date.format("%d.%m.%Y"),
        devises_xml(rates, true)
    )
}

/// Ce que la source factice répond.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MockReply {
    /// XML synthétique **généré pour le mois ou le jour demandé**
    /// ([`MOCK_MONTHLY_RATES`], [`MOCK_DAILY_RATES`]).
    Default,
    /// Ce XML, servi tel quel (un `<datum>` antérieur pour un dimanche…).
    Xml(String),
    /// La racine sans enfant : « pas encore publié ».
    Empty,
    /// Cette erreur.
    Error(SourceError),
}

/// Une réponse pilotée : ce que répond la source, après un délai éventuel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockBehavior {
    /// La réponse.
    pub reply: MockReply,
    /// Attente avant de répondre.
    pub delay: Option<Duration>,
}

impl MockBehavior {
    /// Réponse immédiate.
    pub fn reply(reply: MockReply) -> Self {
        Self { reply, delay: None }
    }
}

#[derive(Debug, Default)]
struct MockState {
    par_genre: HashMap<RateKind, MockBehavior>,
    par_unite: HashMap<RateUnit, MockBehavior>,
    appels: Vec<RateUnit>,
}

/// La source **factice** : aucun appel sortant. XML synthétique par défaut,
/// réponses pilotées par genre (`month` | `day`) ou par unité, compteur
/// d'appels. Clonable : les clones partagent l'état (poignée
/// `AppState::test_rate_source` du mode test, tests Rust).
///
/// **Tests et mode test seulement** ; la production construit
/// [`HttpRateSource`].
#[doc(hidden)]
#[derive(Debug, Clone, Default)]
pub struct MockRateSource {
    state: Arc<Mutex<MockState>>,
}

impl MockRateSource {
    /// Source factice au comportement par défaut.
    pub fn new() -> Self {
        Self::default()
    }

    fn etat(&self) -> std::sync::MutexGuard<'_, MockState> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Pilote la réponse d'un genre (toutes ses unités sans réponse propre).
    pub fn set(&self, kind: RateKind, behavior: MockBehavior) {
        self.etat().par_genre.insert(kind, behavior);
    }

    /// Pilote la réponse d'une unité (prime sur celle du genre).
    pub fn set_for(&self, unit: RateUnit, behavior: MockBehavior) {
        self.etat().par_unite.insert(unit, behavior);
    }

    /// Revient au comportement par défaut et vide le compteur d'appels.
    pub fn reset(&self) {
        *self.etat() = MockState::default();
    }

    /// Les unités demandées, dans l'ordre des appels.
    pub fn calls(&self) -> Vec<RateUnit> {
        self.etat().appels.clone()
    }

    /// Nombre d'appels reçus.
    pub fn call_count(&self) -> usize {
        self.etat().appels.len()
    }

    /// Répond pour `unit` : enregistre l'appel, attend, rend la réponse pilotée.
    fn repondre(&self, unit: RateUnit) -> RateFuture<'_> {
        let behavior = {
            let mut etat = self.etat();
            etat.appels.push(unit);
            etat.par_unite
                .get(&unit)
                .or_else(|| etat.par_genre.get(&unit.kind))
                .cloned()
                .unwrap_or(MockBehavior::reply(MockReply::Default))
        };
        Box::pin(async move {
            if let Some(d) = behavior.delay {
                tokio::time::sleep(d).await;
            }
            match behavior.reply {
                MockReply::Default => Ok(match unit.kind {
                    RateKind::Month => {
                        monthly_xml(unit.date.year(), unit.date.month(), MOCK_MONTHLY_RATES)
                    }
                    RateKind::Day => daily_xml(unit.date, MOCK_DAILY_RATES),
                }),
                MockReply::Xml(x) => Ok(x),
                MockReply::Empty => Ok(match unit.kind {
                    RateKind::Month => "<monatsmittelkurs xmlns=\"https://www.backend-rates.bazg.admin.ch/avgratesmonth\"></monatsmittelkurs>".to_string(),
                    RateKind::Day => "<wechselkurse xmlns=\"https://www.backend-rates.bazg.admin.ch/xmldaily\"></wechselkurse>".to_string(),
                }),
                MockReply::Error(e) => Err(e),
            }
        })
    }
}

impl RateSource for MockRateSource {
    fn fetch_month(&self, year: i32, month: u32) -> RateFuture<'_> {
        match NaiveDate::from_ymd_opt(year, month, 1) {
            Some(d) => self.repondre(RateUnit::month_of(d)),
            None => Box::pin(async { Err(SourceError::HttpStatus(400)) }),
        }
    }

    fn fetch_day(&self, date: NaiveDate) -> RateFuture<'_> {
        self.repondre(RateUnit::day(date))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kesh_core::exchange_rates::{parse_daily_rates, parse_monthly_rates};

    #[test]
    fn codes_classes_aller_retour() {
        for e in [
            SourceError::Timeout,
            SourceError::Connect,
            SourceError::HttpStatus(503),
            SourceError::TooLarge,
            SourceError::Malformed,
            SourceError::NotPublished,
        ] {
            assert_eq!(SourceError::from_code(&e.code()), Some(e));
        }
        assert_eq!(SourceError::from_code("http_status:42"), None);
        assert_eq!(SourceError::from_code("inconnu"), None);
    }

    /// Le XML synthétique par défaut est lu par l'analyse de `kesh-core`, au
    /// mois ou au jour demandé, journaliers distincts des mensuels.
    #[tokio::test]
    async fn xml_par_defaut_genere_pour_l_unite() {
        let s = MockRateSource::new();
        let m = s.fetch_month(2026, 12).await.unwrap();
        let m = parse_monthly_rates(m.as_bytes()).unwrap();
        assert_eq!(m.month, NaiveDate::from_ymd_opt(2026, 12, 1).unwrap());
        assert_eq!(m.rates.len(), MOCK_MONTHLY_RATES.len());
        assert!(m.rejected.is_empty());
        let jour = NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();
        let j = s.fetch_day(jour).await.unwrap();
        let j = parse_daily_rates(j.as_bytes()).unwrap();
        assert_eq!(j.date, jour);
        assert_eq!(j.rates.len(), MOCK_DAILY_RATES.len());
        for (mensuel, journalier) in m.rates.iter().zip(&j.rates) {
            assert_ne!(mensuel.rate_per_unit, journalier.rate_per_unit);
        }
        assert_eq!(s.call_count(), 2);
    }

    /// Pilotage par genre puis par unité ; `reset` revient au défaut.
    #[tokio::test]
    async fn pilotage_par_genre_et_par_unite() {
        let s = MockRateSource::new();
        s.set(
            RateKind::Month,
            MockBehavior::reply(MockReply::Error(SourceError::Timeout)),
        );
        let dec = RateUnit::month_of(NaiveDate::from_ymd_opt(2026, 12, 1).unwrap());
        s.set_for(dec, MockBehavior::reply(MockReply::Empty));
        assert_eq!(s.fetch_month(2026, 11).await, Err(SourceError::Timeout));
        let vide = s.fetch_month(2026, 12).await.unwrap();
        assert!(parse_monthly_rates(vide.as_bytes()).is_err());
        assert_eq!(s.calls().len(), 2);
        s.reset();
        assert_eq!(s.call_count(), 0);
        assert!(s.fetch_month(2026, 11).await.is_ok());
    }
}
