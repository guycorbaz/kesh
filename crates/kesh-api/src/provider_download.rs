//! Téléchargement des mouvements d'un prestataire — socle S2 et S3 (Story
//! 38-12a, AC-4, AC-6, AC-8), commun à PayPal, Wise et Stripe.
//!
//! - [`windows`] découpe une période en **fenêtres** de jours de Zurich, jamais
//!   au-delà de l'instant ; [`period_windows`] y ajoute la fin ramenée au jour du
//!   jour et le plancher de l'historique du prestataire (revue P1, F1, F11) ;
//! - [`check_period`] refuse une période impossible (`PROVIDER_PERIOD_INVALID`) ;
//! - [`classify_status`] et [`classify_transport`] classent l'échec d'une
//!   requête (passager, refus d'authentification, de droits, autre), pour tous
//!   les clients de prestataire (revue P1, F2, F5) ;
//! - [`run_bounded`] lance des travaux en parallèle, **jamais plus** que le
//!   parallélisme réglé (revue P1, F1) ;
//! - [`with_retry`] applique la **politique unique de reprise** : sur 429 et
//!   5xx (délai compris), pause **commune** à tous les fils du prestataire,
//!   `Retry-After` borné à 120 s sinon 5, 10, 20, 40 puis 60 secondes, la même
//!   requête reprise, six essais au plus ; aucune pause fixe entre requêtes ;
//! - [`Downloads`] tient les **tâches** en mémoire du processus (une par
//!   prestataire, société et libellé tant qu'elle tourne), leur progression et
//!   leur issue, purgées une heure après leur fin et perdues au redémarrage
//!   (relancer est sans risque : les doublons se reconnaissent à l'identifiant) ;
//!   un **sémaphore** par prestataire borne les requêtes en vol.
//!
//! L'attente est injectable ([`Wait`]) : les tests ne dorment pas.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chrono::{DateTime, Days, Months, NaiveDate, SubsecRound, Utc};
use kesh_core::horloge::{debut_du_jour_suisse, fin_du_jour_suisse, jour_suisse};
use serde::Serialize;
use tokio::sync::Semaphore;

use crate::errors::AppError;

// ---------------------------------------------------------------------------
// Fenêtres
// ---------------------------------------------------------------------------

/// Une fenêtre : des jours de Zurich `from..=to`, et ses bornes envoyées au
/// prestataire, en UTC, tronquées à la seconde.
///
/// **Recouvrement d'une seconde** (revue P1, F3) : la fin d'une fenêtre est le
/// premier instant du jour qui suit `to` (`00:00:00` de Zurich), c'est-à-dire
/// le début de la fenêtre suivante. Que le prestataire tienne sa borne de fin
/// pour incluse ou exclue, aucune seconde n'échappe ; une transaction de cette
/// seconde-là, lue deux fois, est une répétition identique, que les appelants
/// écartent déjà (`rows_of` pour PayPal).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    /// Premier jour.
    pub from: NaiveDate,
    /// Dernier jour.
    pub to: NaiveDate,
    /// Début : le premier instant de `from` à Zurich.
    pub start: DateTime<Utc>,
    /// Fin : le premier instant du jour qui suit `to` à Zurich (recouvrement
    /// d'une seconde avec la fenêtre suivante), ramenée à `now` si elle est
    /// plus tard.
    pub end: DateTime<Utc>,
}

impl Window {
    /// La fenêtre `from..=to`, fin ramenée à `now`.
    pub fn new(from: NaiveDate, to: NaiveDate, now: DateTime<Utc>) -> Self {
        let last_second = fin_du_jour_suisse(to).trunc_subsecs(0);
        let end = last_second
            .checked_add_signed(chrono::TimeDelta::seconds(1))
            .unwrap_or(last_second)
            .min(now)
            .trunc_subsecs(0);
        Self {
            from,
            to,
            start: debut_du_jour_suisse(from).trunc_subsecs(0),
            end,
        }
    }

    /// Les deux moitiés de la fenêtre (en jours), `None` pour une fenêtre d'un
    /// jour (le plancher). La première moitié garde le début de la fenêtre,
    /// relevé au plancher de l'historique s'il l'a été ([`period_windows`]).
    pub fn split(&self, now: DateTime<Utc>) -> Option<(Window, Window)> {
        let days = (self.to - self.from).num_days();
        if days < 1 {
            return None;
        }
        let mid = self.from.checked_add_days(Days::new((days / 2) as u64))?;
        let next = mid.succ_opt()?;
        let mut first = Window::new(self.from, mid, now);
        first.start = self.start;
        Some((first, Window::new(next, self.to, now)))
    }

    /// La fenêtre est-elle de durée nulle (`start >= end`) ? Possible pour la
    /// seconde moitié d'une coupe dont le jour de début est celui du jour, à
    /// la première seconde de ce jour (revue P3, P3-1) : on ne l'envoie pas.
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Libellé de la fenêtre pour un message (`2024-01-01…2024-01-30`).
    pub fn label(&self) -> String {
        format!("{}…{}", self.from, self.to)
    }
}

/// Découpe `from..=to` en fenêtres contiguës de `max_days` jours de Zurich au
/// plus — Story 38-12a (AC-4). Bornes par `debut_du_jour_suisse` et
/// `fin_du_jour_suisse` (aucun décalage recalculé) ; la fin d'une fenêtre est
/// le début de la suivante (recouvrement d'une seconde, [`Window`]) ; la fin de
/// la dernière est ramenée à `now` si elle est plus tard.
///
/// **Aucune fenêtre ne commence après `now`** (revue P1, F1) : une `to` future
/// ne produit ni fenêtre inversée (début après la fin ramenée à `now`) ni
/// fenêtres sans nombre — le découpage s'arrête à la fenêtre qui contient
/// `now`. Défense en profondeur : les routes ramènent déjà `to` au jour du jour
/// ([`period_windows`]).
///
/// Règle de `max_days` du socle : la limite du prestataire, moins un jour quand
/// elle ne se dit pas en jours entiers de Zurich (PayPal : 31 jours, unité non
/// dite → 30 ; une fenêtre de 31 jours qui contient le passage à l'heure
/// d'hiver dure 31 × 24 h + 1 h).
pub fn windows(from: NaiveDate, to: NaiveDate, max_days: u32, now: DateTime<Utc>) -> Vec<Window> {
    let mut out = Vec::new();
    // `to` est ramenée au jour du jour : aucune fenêtre (ni moitié d'une
    // coupe, [`Window::split`]) ne commence après `now` (revue P2, G3).
    let to = effective_to(to, now);
    if max_days == 0 || from > to {
        return out;
    }
    let mut start = from;
    loop {
        let end = start
            .checked_add_days(Days::new(u64::from(max_days) - 1))
            .map_or(to, |e| e.min(to));
        let window = Window::new(start, end, now);
        // `>=` : une fenêtre qui commence à `now` (la première seconde du
        // jour) serait de durée nulle, `start == end` (revue P2, G2).
        if window.start >= now {
            break;
        }
        out.push(window);
        match end.succ_opt() {
            Some(next) if next <= to => start = next,
            _ => break,
        }
    }
    out
}

/// Marge du plancher de l'historique : la première fenêtre commence une heure
/// après « maintenant moins `years` ans », pour que la requête, envoyée plus
/// tard que l'instant de la demande (file, pauses), reste dans l'historique du
/// prestataire (revue P1, F11).
pub const HISTORY_MARGIN: Duration = Duration::from_secs(3600);

/// Le plancher de l'historique d'un prestataire : `now` moins `years` ans (même
/// jour et même heure ; un 29 février retombe sur le 28), plus
/// [`HISTORY_MARGIN`]. Aucune requête ne part avant lui.
pub fn history_floor(now: DateTime<Utc>, years: u32) -> DateTime<Utc> {
    now.checked_sub_months(Months::new(12 * years))
        .and_then(|t| {
            t.checked_add_signed(chrono::TimeDelta::seconds(HISTORY_MARGIN.as_secs() as i64))
        })
        .unwrap_or(DateTime::<Utc>::MIN_UTC)
}

/// Motif d'un refus `PROVIDER_PERIOD_INVALID` (`details.reason`).
pub type PeriodRefusal = &'static str;

/// Contrôle une période demandée (AC-4), au jour de Zurich de `now` :
/// `from_after_to`, `future` (`from` après le jour du jour — une `to` future
/// n'est pas refusée, elle est ramenée au jour du jour par
/// [`period_windows`]), `too_old` (`from` avant le jour du plancher de
/// l'historique, [`history_floor`]). Le jour du plancher est **admis** : sa
/// fenêtre commence au plancher lui-même (une heure après « maintenant moins
/// `years` ans », [`HISTORY_MARGIN`]), si bien qu'aucune requête ne remonte
/// au-delà de l'historique (revue P1, F11). **Marge assumée** (revue P2, G4) :
/// pour une demande qui commence au jour exact du plancher, la première heure
/// de ce jour n'est pas lue — le nom de l'import et la trace disent le jour
/// entier demandé, non l'heure réelle de début.
pub fn check_period(
    from: NaiveDate,
    to: NaiveDate,
    now: DateTime<Utc>,
    years: u32,
) -> Result<(), PeriodRefusal> {
    if from > to {
        return Err("from_after_to");
    }
    if from > jour_suisse(now) {
        return Err("future");
    }
    if from < jour_suisse(history_floor(now, years)) {
        return Err("too_old");
    }
    Ok(())
}

/// Le dernier jour lu d'une période : `to`, ramenée au jour de Zurich de `now`
/// si elle est plus tard (une `to` future n'est pas refusée, AC-4).
pub fn effective_to(to: NaiveDate, now: DateTime<Utc>) -> NaiveDate {
    to.min(jour_suisse(now))
}

/// Les fenêtres d'une période contrôlée par [`check_period`] — ce que les routes
/// appellent (revue P1, F1, F11) : `to` ramenée au jour du jour
/// ([`effective_to`]), découpage par [`windows`], puis le début de la première
/// fenêtre relevé au plancher de l'historique ([`history_floor`]) ; une
/// fenêtre qui finirait avant de commencer, ou en même temps, est écartée.
pub fn period_windows(
    from: NaiveDate,
    to: NaiveDate,
    max_days: u32,
    now: DateTime<Utc>,
    years: u32,
) -> Vec<Window> {
    let floor = history_floor(now, years).trunc_subsecs(0);
    let mut out = windows(from, effective_to(to, now), max_days, now);
    for w in &mut out {
        if w.start < floor {
            w.start = floor;
        }
    }
    out.retain(|w| w.start < w.end);
    out
}

// ---------------------------------------------------------------------------
// Reprise
// ---------------------------------------------------------------------------

/// Essais au plus par requête (le premier compris).
pub const MAX_ATTEMPTS: u32 = 6;
/// Borne d'un `Retry-After`.
pub const RETRY_AFTER_CAP: Duration = Duration::from_secs(120);
/// Plancher d'un `Retry-After` : `Retry-After: 0` ne fait pas reprendre sans
/// pause (revue P1, F9).
pub const RETRY_AFTER_FLOOR: Duration = Duration::from_secs(1);
/// Pauses successives sans `Retry-After`, en secondes.
pub const BACKOFF_SECS: [u64; 5] = [5, 10, 20, 40, 60];

/// La pause après l'échec passager numéro `failed` (1 pour le premier) :
/// `Retry-After` en secondes s'il est présent et entier, entre 1 et 120 s,
/// sinon la suite 5, 10, 20, 40, 60.
pub fn pause_for(failed: u32, retry_after: Option<&str>) -> Duration {
    if let Some(secs) = retry_after.and_then(|v| v.trim().parse::<u64>().ok()) {
        return Duration::from_secs(secs).clamp(RETRY_AFTER_FLOOR, RETRY_AFTER_CAP);
    }
    let i = (failed.max(1) as usize - 1).min(BACKOFF_SECS.len() - 1);
    Duration::from_secs(BACKOFF_SECS[i])
}

/// Genre d'un échec passager.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transient {
    /// 429.
    RateLimited,
    /// 5xx ou délai dépassé.
    Unavailable,
}

/// Classe d'un statut HTTP non 2xx, commune aux clients des prestataires
/// (revue P1, F5) ; chaque client traduit ensuite les classes en ses erreurs
/// (renouvellement du jeton sur 401, cas propres comme `RESULTSET_TOO_LARGE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusClass {
    /// 401 : identifiants ou jeton refusés.
    Unauthorized,
    /// 403 : droits insuffisants (`PROVIDER_PERMISSION_DENIED`).
    Forbidden,
    /// 429 et 5xx : à reprendre.
    Transient(Transient),
    /// Tout autre statut (`PROVIDER_API_ERROR`, sauf cas propre au client).
    Other,
}

/// Classe un statut HTTP non 2xx (AC-6) : 401, 403, 429, 5xx, le reste.
pub fn classify_status(status: u16) -> StatusClass {
    match status {
        401 => StatusClass::Unauthorized,
        403 => StatusClass::Forbidden,
        429 => StatusClass::Transient(Transient::RateLimited),
        // 408 : délai dépassé côté serveur, passager comme celui du client.
        408 | 500..=599 => StatusClass::Transient(Transient::Unavailable),
        _ => StatusClass::Other,
    }
}

/// Classe une erreur de transport : `Some(Unavailable)` pour un délai dépassé
/// ou une connexion refusée, coupée ou injoignable, à reprendre ; `None` pour
/// une erreur **définitive** — certificat refusé, hôte inconnu, schéma refusé
/// — que la reprise ne changerait pas (revue P1, F2 :
/// [`crate::http_client::is_transient_transport`]).
pub fn classify_transport(e: &reqwest::Error) -> Option<Transient> {
    crate::http_client::is_transient_transport(e).then_some(Transient::Unavailable)
}

/// L'issue d'un essai.
#[derive(Debug)]
pub enum Attempt<T, E> {
    /// Réussi.
    Done(T),
    /// Passager : à reprendre après une pause.
    Transient {
        /// Genre.
        kind: Transient,
        /// En-tête `Retry-After`, tel que reçu.
        retry_after: Option<String>,
    },
    /// Définitif.
    Fatal(E),
}

/// L'échec d'une requête après reprise.
#[derive(Debug)]
pub enum RetryError<E> {
    /// Six essais passagers.
    Exhausted(Transient),
    /// Un essai définitif.
    Fatal(E),
}

/// Attente injectable.
pub trait Wait: Send + Sync {
    /// Attend `d`.
    fn wait(&self, d: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}

/// L'attente réelle (`tokio::time::sleep`).
pub struct TokioWait;

impl Wait for TokioWait {
    fn wait(&self, d: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(tokio::time::sleep(d))
    }
}

/// L'état partagé d'un prestataire : sémaphore des requêtes en vol et instant
/// de reprise commun.
pub struct ProviderGate {
    semaphore: Semaphore,
    paused_until: Mutex<Option<Instant>>,
}

impl ProviderGate {
    /// `permits` requêtes en vol au plus.
    pub fn new(permits: usize) -> Self {
        Self {
            semaphore: Semaphore::new(permits.max(1)),
            paused_until: Mutex::new(None),
        }
    }

    /// Secondes de pause restantes, `None` hors pause.
    pub fn paused_seconds(&self) -> Option<u64> {
        let until = (*self.paused_until.lock().expect("pause"))?;
        let left = until.checked_duration_since(Instant::now())?;
        Some(left.as_secs_f64().ceil() as u64)
    }

    /// Repousse l'instant de reprise commun à `now + d` au moins.
    fn pause(&self, d: Duration) {
        let until = Instant::now() + d;
        let mut slot = self.paused_until.lock().expect("pause");
        if slot.is_none_or(|t| t < until) {
            *slot = Some(until);
        }
    }

    /// Le reste de la pause commune, s'il y en a une.
    fn remaining(&self) -> Option<Duration> {
        let until = (*self.paused_until.lock().expect("pause"))?;
        until.checked_duration_since(Instant::now())
    }
}

/// Exécute une requête sous la politique de reprise (AC-6) : attend la pause
/// commune, prend une place du sémaphore, essaie ; sur un échec passager,
/// repousse la pause commune et reprend la **même** requête, six essais au
/// plus.
pub async fn with_retry<T, E, F, Fut>(
    gate: &ProviderGate,
    wait: &dyn Wait,
    mut attempt: F,
) -> Result<T, RetryError<E>>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Attempt<T, E>>,
{
    let mut failed = 0u32;
    loop {
        if let Some(d) = gate.remaining() {
            wait.wait(d).await;
        }
        let outcome = {
            let _permit = gate
                .semaphore
                .acquire()
                .await
                .expect("sémaphore jamais fermé");
            attempt().await
        };
        match outcome {
            Attempt::Done(v) => return Ok(v),
            Attempt::Fatal(e) => return Err(RetryError::Fatal(e)),
            Attempt::Transient { kind, retry_after } => {
                failed += 1;
                if failed >= MAX_ATTEMPTS {
                    return Err(RetryError::Exhausted(kind));
                }
                let d = pause_for(failed, retry_after.as_deref());
                tracing::warn!(
                    ?kind,
                    essai = failed,
                    pause_s = d.as_secs(),
                    "prestataire : échec passager, reprise après pause"
                );
                gate.pause(d);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Parallélisme borné
// ---------------------------------------------------------------------------

/// Exécute les travaux `works`, **au plus `parallel` à la fois** (1 au moins),
/// et rend leurs résultats **dans l'ordre des travaux** — revue P1, F1 : le
/// nombre de tâches lancées est borné par le parallélisme réglé, jamais par le
/// nombre de fenêtres. Les travaux sont tirés de l'itérateur au fur et à mesure
/// (aucun n'est créé d'avance). Le premier échec arrête tout (les autres sont
/// annulés) ; une panique se propage (la tâche de fond la conclut en
/// `INTERNAL_ERROR`, [`spawn_job`]).
pub async fn run_bounded<T, F, I>(works: I, parallel: usize) -> Result<Vec<T>, AppError>
where
    I: IntoIterator<Item = F>,
    F: Future<Output = Result<T, AppError>> + Send + 'static,
    T: Send + 'static,
{
    let parallel = parallel.max(1);
    let mut pending = works.into_iter().enumerate();
    let mut set = tokio::task::JoinSet::new();
    let mut parts: Vec<(usize, T)> = Vec::new();
    loop {
        while set.len() < parallel {
            let Some((i, work)) = pending.next() else {
                break;
            };
            set.spawn(async move { (i, work.await) });
        }
        let Some(joined) = set.join_next().await else {
            break;
        };
        match joined {
            Ok((i, Ok(v))) => parts.push((i, v)),
            Ok((_, Err(e))) => {
                set.abort_all();
                return Err(e);
            }
            Err(join) => {
                set.abort_all();
                if join.is_panic() {
                    std::panic::resume_unwind(join.into_panic());
                }
                return Err(AppError::Internal("travail interrompu".into()));
            }
        }
    }
    parts.sort_by_key(|(i, _)| *i);
    Ok(parts.into_iter().map(|(_, v)| v).collect())
}

// ---------------------------------------------------------------------------
// Tâches
// ---------------------------------------------------------------------------

/// Durée de vie d'une tâche finie.
pub const JOB_RETENTION: Duration = Duration::from_secs(3600);

/// État d'une tâche (forme commune S3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum JobState {
    /// En cours.
    Running,
    /// Réussie.
    Done,
    /// En échec.
    Failed,
}

#[derive(Debug, Clone)]
struct Job {
    provider: &'static str,
    company_id: i64,
    account_key: String,
    /// La période lue (début, fin ramenée au jour du jour) : le refus d'un
    /// second lancement la rend, pour que l'écran dise ce qu'il suit.
    period: (NaiveDate, NaiveDate),
    state: JobState,
    steps_done: u32,
    steps_total: Option<u32>,
    result: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
    finished_at: Option<Instant>,
}

/// La tâche qui tourne déjà pour un compte, rendue par le refus de
/// [`Downloads::start`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunningJob {
    /// Son identifiant.
    pub id: u64,
    /// Le début de sa période.
    pub from: NaiveDate,
    /// La fin lue de sa période.
    pub to: NaiveDate,
}

/// Progression d'une tâche.
#[derive(Debug, Clone, Serialize)]
pub struct Steps {
    /// Étapes faites.
    pub done: u32,
    /// Étapes en tout, `null` si inconnu d'avance.
    pub total: Option<u32>,
}

/// La réponse de `GET /api/v1/<prestataire>-downloads/{jobId}` (forme commune
/// S3).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobView {
    /// État.
    pub state: JobState,
    /// Progression.
    pub steps: Steps,
    /// Secondes de pause restantes (limitation de débit), `null` hors pause.
    pub paused_until_seconds: Option<u64>,
    /// Le compte rendu d'une tâche réussie.
    pub result: Option<serde_json::Value>,
    /// L'erreur d'une tâche en échec : `{ code, message, details? }`.
    pub error: Option<serde_json::Value>,
}

/// Les tâches et l'état partagé des prestataires, dans l'état d'application.
pub struct Downloads {
    wait: Arc<dyn Wait>,
    timeouts: Option<(Duration, Duration)>,
    gates: Mutex<HashMap<&'static str, Arc<ProviderGate>>>,
    jobs: Mutex<(u64, HashMap<u64, Job>)>,
}

impl Default for Downloads {
    fn default() -> Self {
        Self::new(Arc::new(TokioWait), None)
    }
}

impl Downloads {
    /// Tâches vides ; `timeouts` remplace les délais (connexion, total) des
    /// clients — tests seulement.
    pub fn new(wait: Arc<dyn Wait>, timeouts: Option<(Duration, Duration)>) -> Self {
        Self {
            wait,
            timeouts,
            gates: Mutex::new(HashMap::new()),
            jobs: Mutex::new((0, HashMap::new())),
        }
    }

    /// L'attente.
    pub fn wait(&self) -> Arc<dyn Wait> {
        self.wait.clone()
    }

    /// Les délais imposés par les tests, s'il y en a.
    pub fn timeouts(&self) -> Option<(Duration, Duration)> {
        self.timeouts
    }

    /// L'état partagé d'un prestataire, créé au premier appel avec `permits`
    /// places (le parallélisme lu au démarrage).
    pub fn gate(&self, provider: &'static str, permits: usize) -> Arc<ProviderGate> {
        self.gates
            .lock()
            .expect("gates")
            .entry(provider)
            .or_insert_with(|| Arc::new(ProviderGate::new(permits)))
            .clone()
    }

    fn purge(jobs: &mut HashMap<u64, Job>) {
        jobs.retain(|_, j| j.finished_at.is_none_or(|t| t.elapsed() < JOB_RETENTION));
    }

    /// Ouvre une tâche sur la période `period` (début, fin lue), ou rend
    /// celle qui tourne déjà pour ce prestataire, cette société et ce libellé
    /// (sans casse), **quelle que soit sa période** : son identifiant et sa
    /// période, que le refus `409` transmet (revue P1 de la 38-12b, M1).
    pub fn start(
        self: &Arc<Self>,
        provider: &'static str,
        company_id: i64,
        account: &str,
        period: (NaiveDate, NaiveDate),
        steps_total: Option<u32>,
    ) -> Result<JobHandle, RunningJob> {
        let account_key = account.trim().to_lowercase();
        let mut guard = self.jobs.lock().expect("jobs");
        let (next, jobs) = &mut *guard;
        Self::purge(jobs);
        if let Some((id, j)) = jobs.iter().find(|(_, j)| {
            j.state == JobState::Running
                && j.provider == provider
                && j.company_id == company_id
                && j.account_key == account_key
        }) {
            return Err(RunningJob {
                id: *id,
                from: j.period.0,
                to: j.period.1,
            });
        }
        *next += 1;
        let id = *next;
        jobs.insert(
            id,
            Job {
                provider,
                company_id,
                account_key,
                period,
                state: JobState::Running,
                steps_done: 0,
                steps_total,
                result: None,
                error: None,
                finished_at: None,
            },
        );
        Ok(JobHandle {
            id,
            downloads: Arc::clone(self),
        })
    }

    /// La vue d'une tâche de la société `company_id` et du prestataire ;
    /// `None` si elle n'existe pas, est purgée, ou appartient à une autre.
    pub fn view(&self, provider: &'static str, id: u64, company_id: i64) -> Option<JobView> {
        let mut guard = self.jobs.lock().expect("jobs");
        Self::purge(&mut guard.1);
        let job = guard.1.get(&id)?;
        if job.company_id != company_id || job.provider != provider {
            return None;
        }
        let paused = if job.state == JobState::Running {
            self.gates
                .lock()
                .expect("gates")
                .get(provider)
                .and_then(|g| g.paused_seconds())
        } else {
            None
        };
        Some(JobView {
            state: job.state,
            steps: Steps {
                done: job.steps_done,
                total: job.steps_total,
            },
            paused_until_seconds: paused,
            result: job.result.clone(),
            error: job.error.clone(),
        })
    }

    fn update(&self, id: u64, f: impl FnOnce(&mut Job)) {
        if let Some(job) = self.jobs.lock().expect("jobs").1.get_mut(&id) {
            f(job);
        }
    }
}

/// Poignée d'une tâche ouverte : progression et issue.
#[derive(Clone)]
pub struct JobHandle {
    /// Identifiant (entier croissant du processus).
    pub id: u64,
    downloads: Arc<Downloads>,
}

impl JobHandle {
    /// Une étape faite.
    pub fn step_done(&self) {
        self.downloads.update(self.id, |j| j.steps_done += 1);
    }

    /// Pose le nombre d'étapes en tout, inconnu jusque-là (Stripe, Story
    /// 40-8b : le nombre de versements n'est connu qu'une fois leur liste lue).
    pub fn set_total(&self, total: u32) {
        self.downloads
            .update(self.id, |j| j.steps_total = Some(total));
    }

    /// `n` étapes de plus (une fenêtre coupée en deux en ajoute une).
    pub fn add_steps(&self, n: u32) {
        self.downloads
            .update(self.id, |j| j.steps_total = j.steps_total.map(|t| t + n));
    }

    fn finish(
        &self,
        state: JobState,
        result: Option<serde_json::Value>,
        error: Option<serde_json::Value>,
    ) {
        self.downloads.update(self.id, |j| {
            if j.state == JobState::Running {
                j.state = state;
                j.result = result;
                j.error = error;
                j.finished_at = Some(Instant::now());
            }
        });
    }
}

/// Le corps `{ code, message, details }` d'une erreur, `details.provider`
/// ajouté (le rendu de l'erreur fait foi : même code, même message traduit
/// qu'une réponse HTTP).
pub async fn error_body(err: AppError, provider: &'static str) -> serde_json::Value {
    let response = axum::response::IntoResponse::into_response(err);
    let bytes = axum::body::to_bytes(response.into_body(), 1 << 20)
        .await
        .unwrap_or_default();
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();
    let mut error = value
        .get_mut("error")
        .map(serde_json::Value::take)
        .unwrap_or_else(|| serde_json::json!({ "code": "INTERNAL_ERROR" }));
    if let Some(map) = error.as_object_mut() {
        let details = map
            .entry("details")
            .or_insert_with(|| serde_json::json!({}));
        if !details.is_object() {
            *details = serde_json::json!({});
        }
        if let Some(d) = details.as_object_mut() {
            d.insert("provider".into(), serde_json::json!(provider));
        }
    }
    error
}

/// Lance la tâche `work` en tâche de fond. Son issue la conclut ; **une
/// panique ne la laisse jamais « en cours »** : le `JoinHandle` est surveillé
/// et la tâche passe en `failed` `INTERNAL_ERROR` (faute de Kesh, non du
/// prestataire).
pub fn spawn_job<F>(handle: JobHandle, provider: &'static str, work: F)
where
    F: Future<Output = Result<serde_json::Value, AppError>> + Send + 'static,
{
    tokio::spawn(async move {
        let inner = tokio::spawn(work);
        match inner.await {
            Ok(Ok(result)) => handle.finish(JobState::Done, Some(result), None),
            Ok(Err(err)) => {
                let body = error_body(err, provider).await;
                handle.finish(JobState::Failed, None, Some(body));
            }
            Err(join) => {
                tracing::error!(
                    provider,
                    job = handle.id,
                    panique = join.is_panic(),
                    "téléchargement : tâche interrompue"
                );
                let body = error_body(
                    AppError::Internal("tâche de téléchargement interrompue".into()),
                    provider,
                )
                .await;
                handle.finish(JobState::Failed, None, Some(body));
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn far() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap()
    }

    #[test]
    fn quatre_mois_en_fenetres_de_trente_jours() {
        let w = windows(d(2024, 1, 1), d(2024, 4, 30), 30, far());
        let days: Vec<(NaiveDate, NaiveDate)> = w.iter().map(|w| (w.from, w.to)).collect();
        assert_eq!(
            days,
            [
                (d(2024, 1, 1), d(2024, 1, 30)),
                (d(2024, 1, 31), d(2024, 2, 29)),
                (d(2024, 3, 1), d(2024, 3, 30)),
                (d(2024, 3, 31), d(2024, 4, 29)),
                (d(2024, 4, 30), d(2024, 4, 30)),
            ]
        );
        // Recouvrement d'une seconde : la fin d'une fenêtre est le début de la
        // suivante (revue P1, F3).
        for pair in w.windows(2) {
            assert_eq!(pair[0].end, pair[1].start);
        }
        // Bornes de Zurich, à la seconde : hiver (+01:00).
        assert_eq!(
            w[0].start,
            Utc.with_ymd_and_hms(2023, 12, 31, 23, 0, 0).unwrap()
        );
        assert_eq!(
            w[0].end,
            Utc.with_ymd_and_hms(2024, 1, 30, 23, 0, 0).unwrap()
        );
        // Été (+02:00).
        assert_eq!(
            w[3].end,
            Utc.with_ymd_and_hms(2024, 4, 29, 22, 0, 0).unwrap()
        );
    }

    #[test]
    fn trente_jours_ne_depassent_jamais_trente_et_un_fois_vingt_quatre_heures() {
        // Fenêtres glissées sur deux années : passages à l'heure d'été et
        // d'hiver compris.
        let limite = chrono::Duration::hours(31 * 24);
        for start in 0..730u64 {
            let from = d(2023, 1, 1).checked_add_days(Days::new(start)).unwrap();
            for w in windows(
                from,
                from.checked_add_days(Days::new(29)).unwrap(),
                30,
                far(),
            ) {
                assert!(w.end - w.start < limite, "{w:?}");
            }
        }
        // À 31 jours, la fenêtre qui contient le passage à l'heure d'hiver
        // dépasse 31 × 24 h (le constat qui fonde les 30 jours).
        let w = windows(d(2024, 10, 1), d(2024, 10, 31), 31, far());
        assert_eq!(w.len(), 1);
        assert!(w[0].end - w[0].start >= limite);
        assert_eq!(w[0].end - w[0].start, chrono::Duration::hours(31 * 24 + 1));
    }

    #[test]
    fn fin_d_annee_et_fin_ramenee_a_l_instant() {
        let w = windows(d(2024, 12, 15), d(2025, 1, 20), 30, far());
        assert_eq!(
            w.iter().map(|w| (w.from, w.to)).collect::<Vec<_>>(),
            [
                (d(2024, 12, 15), d(2025, 1, 13)),
                (d(2025, 1, 14), d(2025, 1, 20))
            ]
        );
        let now = Utc.with_ymd_and_hms(2025, 1, 16, 10, 30, 15).unwrap()
            + chrono::Duration::milliseconds(700);
        let w = windows(d(2025, 1, 1), d(2025, 1, 31), 30, now);
        assert_eq!(
            w.last().unwrap().end,
            Utc.with_ymd_and_hms(2025, 1, 16, 10, 30, 15).unwrap()
        );
        assert!(windows(d(2025, 1, 2), d(2025, 1, 1), 30, now).is_empty());
    }

    #[test]
    fn une_fenetre_se_coupe_en_deux_jusqu_au_jour() {
        let w = Window::new(d(2024, 1, 1), d(2024, 1, 30), far());
        let (a, b) = w.split(far()).unwrap();
        assert_eq!(
            (a.from, a.to, b.from, b.to),
            (
                d(2024, 1, 1),
                d(2024, 1, 15),
                d(2024, 1, 16),
                d(2024, 1, 30)
            )
        );
        let one = Window::new(d(2024, 1, 1), d(2024, 1, 1), far());
        assert!(one.split(far()).is_none());
        let two = Window::new(d(2024, 1, 1), d(2024, 1, 2), far());
        let (a, b) = two.split(far()).unwrap();
        assert_eq!((a.to, b.from), (d(2024, 1, 1), d(2024, 1, 2)));
    }

    #[test]
    fn periodes_refusees() {
        let now = Utc.with_ymd_and_hms(2026, 11, 20, 10, 0, 0).unwrap();
        let today = d(2026, 11, 20);
        assert_eq!(
            check_period(d(2026, 2, 1), d(2026, 1, 1), now, 3),
            Err("from_after_to")
        );
        assert_eq!(
            check_period(d(2026, 11, 21), d(2026, 11, 30), now, 3),
            Err("future")
        );
        assert_eq!(
            check_period(d(2023, 11, 19), d(2024, 1, 1), now, 3),
            Err("too_old")
        );
        assert_eq!(check_period(d(2023, 11, 20), d(2027, 1, 1), now, 3), Ok(()));
        assert_eq!(check_period(today, today, now, 3), Ok(()));
        // Le jour du jour à Zurich, non en UTC : à 23:30 UTC le 20, c'est le 21.
        let tard = Utc.with_ymd_and_hms(2026, 11, 20, 23, 30, 0).unwrap();
        assert_eq!(
            check_period(d(2026, 11, 21), d(2026, 11, 21), tard, 3),
            Ok(())
        );
    }

    /// Revue P1, F1 — une `to` lointaine ne produit ni fenêtre inversée ni
    /// fenêtres sans nombre : le découpage s'arrête à la fenêtre qui contient
    /// `now`.
    #[test]
    fn une_fin_future_s_arrete_a_l_instant() {
        let now = Utc.with_ymd_and_hms(2026, 11, 20, 10, 0, 0).unwrap();
        let w = windows(d(2026, 11, 1), d(9999, 12, 31), 30, now);
        assert_eq!(w.len(), 1, "{w:?}");
        assert_eq!((w[0].from, w[0].to), (d(2026, 11, 1), d(2026, 11, 20)));
        assert_eq!(w[0].end, now);
        // Sur trois ans : autant de fenêtres que de tranches de 30 jours
        // jusqu'au jour du jour, aucune inversée, aucune après `now`.
        let from = d(2023, 11, 21);
        let w = windows(from, d(9999, 12, 31), 30, now);
        let jours = (d(2026, 11, 20) - from).num_days() + 1;
        assert_eq!(w.len() as i64, (jours + 29) / 30);
        assert!(
            w.iter().all(|w| w.start <= w.end && w.start <= now),
            "{w:?}"
        );
        // Revue P2, G3 — `to` ramenée au jour du jour : même `windows` seul ne
        // rend ni fenêtre inversée ni moitié inversée d'une coupe.
        let now5 = Utc.with_ymd_and_hms(2026, 11, 5, 10, 0, 0).unwrap();
        let w = windows(d(2026, 11, 1), d(2026, 11, 30), 30, now5);
        assert_eq!(w[0].to, d(2026, 11, 5), "{w:?}");
        let (a, b) = w[0].split(now5).unwrap();
        assert!(a.start <= a.end && b.start <= b.end, "{a:?} {b:?}");
        // Revue P2, G2 — `now` à minuit pile de Zurich : aucune fenêtre de
        // durée nulle.
        let minuit = debut_du_jour_suisse(d(2026, 11, 20));
        let w = windows(d(2026, 11, 1), d(2026, 11, 20), 30, minuit);
        assert!(w.iter().all(|w| w.start < w.end), "{w:?}");
        let w = period_windows(d(2026, 11, 1), d(2026, 11, 20), 30, minuit, 3);
        assert!(w.iter().all(|w| w.start < w.end), "{w:?}");
        // Une dernière fenêtre qui commence à `now` (le 20, fenêtres de 19
        // jours) : la garde `start >= now` de `windows` l'écarte.
        let w = windows(d(2026, 11, 1), d(2026, 11, 20), 19, minuit);
        assert_eq!(w.len(), 1, "{w:?}");
        assert!(w.iter().all(|w| w.start < w.end), "{w:?}");
        // Revue P3, P3-1 — la seconde moitié d'une coupe, à la première
        // seconde du jour du jour, est de durée nulle : `is_empty`.
        let a_minuit = minuit + chrono::TimeDelta::milliseconds(500);
        let veille = Window::new(d(2026, 11, 19), d(2026, 11, 20), a_minuit);
        let (a, b) = veille.split(a_minuit).unwrap();
        assert!(!a.is_empty(), "{a:?}");
        assert!(b.is_empty(), "{b:?}");
        // Une période entièrement future : aucune fenêtre.
        assert!(windows(d(2027, 1, 1), d(2027, 3, 1), 30, now).is_empty());
    }

    /// Revue P1, F1, F11 — `period_windows` ramène `to` au jour du jour et
    /// relève le début de la première fenêtre au plancher de l'historique ;
    /// la première moitié d'une fenêtre coupée garde ce début.
    #[test]
    fn periode_ramenee_au_jour_et_au_plancher() {
        let now = Utc.with_ymd_and_hms(2026, 11, 20, 10, 0, 0).unwrap();
        let w = period_windows(d(2026, 11, 1), d(9999, 12, 31), 30, now, 3);
        assert_eq!(w.len(), 1);
        assert_eq!(
            (w[0].from, w[0].to, w[0].end),
            (d(2026, 11, 1), d(2026, 11, 20), now)
        );
        assert_eq!(effective_to(d(9999, 12, 31), now), d(2026, 11, 20));
        assert_eq!(effective_to(d(2026, 1, 31), now), d(2026, 1, 31));

        let floor = history_floor(now, 3);
        assert_eq!(floor, Utc.with_ymd_and_hms(2023, 11, 20, 11, 0, 0).unwrap());
        let w = period_windows(d(2023, 11, 20), d(2023, 12, 31), 30, now, 3);
        assert_eq!(w[0].from, d(2023, 11, 20));
        assert_eq!(w[0].start, floor, "aucune requête avant le plancher");
        assert!(w.iter().all(|w| w.start <= w.end));
        let (a, b) = w[0].split(now).unwrap();
        assert_eq!(a.start, floor);
        assert_eq!(a.end, b.start);
        // Plancher d'un 29 février : le 28, même heure, plus la marge.
        let bissextile = Utc.with_ymd_and_hms(2028, 2, 29, 10, 0, 0).unwrap();
        assert_eq!(
            history_floor(bissextile, 3),
            Utc.with_ymd_and_hms(2025, 2, 28, 11, 0, 0).unwrap()
        );
    }

    #[test]
    fn statuts_classes() {
        assert_eq!(classify_status(401), StatusClass::Unauthorized);
        assert_eq!(classify_status(403), StatusClass::Forbidden);
        assert_eq!(
            classify_status(429),
            StatusClass::Transient(Transient::RateLimited)
        );
        for s in [408, 500, 502, 503, 599] {
            assert_eq!(
                classify_status(s),
                StatusClass::Transient(Transient::Unavailable)
            );
        }
        for s in [400, 404, 407, 409, 422, 600] {
            assert_eq!(classify_status(s), StatusClass::Other);
        }
    }

    /// Revue P1, F1 — jamais plus de `parallel` travaux en vol, résultats dans
    /// l'ordre, premier échec rendu.
    #[tokio::test]
    async fn parallelisme_borne_et_ordre_garde() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let en_vol = Arc::new(AtomicUsize::new(0));
        let maxi = Arc::new(AtomicUsize::new(0));
        let lances = Arc::new(AtomicUsize::new(0));
        let works = (0..40u32).map(|i| {
            let (en_vol, maxi, lances) = (en_vol.clone(), maxi.clone(), lances.clone());
            async move {
                lances.fetch_add(1, Ordering::SeqCst);
                let n = en_vol.fetch_add(1, Ordering::SeqCst) + 1;
                maxi.fetch_max(n, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(u64::from(40 - i) % 7)).await;
                en_vol.fetch_sub(1, Ordering::SeqCst);
                Ok::<u32, AppError>(i)
            }
        });
        let r = run_bounded(works, 3).await.unwrap();
        assert_eq!(r, (0..40).collect::<Vec<_>>());
        assert!(maxi.load(Ordering::SeqCst) <= 3, "{maxi:?}");
        assert_eq!(lances.load(Ordering::SeqCst), 40);

        let works = (0..10u32).map(|i| async move {
            if i == 4 {
                Err(AppError::Internal("x".into()))
            } else {
                Ok(i)
            }
        });
        assert!(run_bounded(works, 1).await.is_err());
    }

    #[test]
    fn suite_des_pauses_et_borne() {
        let seq: Vec<u64> = (1..=6).map(|n| pause_for(n, None).as_secs()).collect();
        assert_eq!(seq, [5, 10, 20, 40, 60, 60]);
        assert_eq!(pause_for(1, Some("7")), Duration::from_secs(7));
        assert_eq!(pause_for(1, Some("3600")), RETRY_AFTER_CAP);
        assert_eq!(
            pause_for(2, Some("Wed, 21 Oct 2015 07:28:00 GMT")),
            Duration::from_secs(10)
        );
        assert_eq!(pause_for(1, Some("-3")), Duration::from_secs(5));
        // Revue P1, F9 : `Retry-After: 0` → la pause minimale, non zéro.
        assert_eq!(pause_for(1, Some("0")), RETRY_AFTER_FLOOR);
    }

    /// Attente qui ne dort pas et note ce qu'on lui demande.
    #[derive(Default)]
    struct Notee(Mutex<Vec<Duration>>);

    impl Wait for Notee {
        fn wait(&self, d: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
            self.0.lock().unwrap().push(d);
            Box::pin(async {})
        }
    }

    #[tokio::test]
    async fn six_essais_puis_echec_et_pauses_notees() {
        let gate = ProviderGate::new(1);
        let wait = Notee::default();
        let mut calls = 0;
        let r: Result<(), RetryError<()>> = with_retry(&gate, &wait, || {
            calls += 1;
            async {
                Attempt::Transient {
                    kind: Transient::RateLimited,
                    retry_after: None,
                }
            }
        })
        .await;
        assert!(matches!(
            r,
            Err(RetryError::Exhausted(Transient::RateLimited))
        ));
        assert_eq!(calls, MAX_ATTEMPTS);
        let waits: Vec<u64> = wait
            .0
            .lock()
            .unwrap()
            .iter()
            .map(|d| d.as_secs_f64().round() as u64)
            .collect();
        assert_eq!(waits, [5, 10, 20, 40, 60]);
    }

    #[tokio::test]
    async fn la_pause_est_commune_aux_fils_du_prestataire() {
        let gate = ProviderGate::new(2);
        let wait = Notee::default();
        // Un fil subit un 429 avec Retry-After : 30 s, puis réussit.
        let mut first = true;
        let r: Result<u8, RetryError<()>> = with_retry(&gate, &wait, || {
            let t = std::mem::replace(&mut first, false);
            async move {
                if t {
                    Attempt::Transient {
                        kind: Transient::RateLimited,
                        retry_after: Some("30".into()),
                    }
                } else {
                    Attempt::Done(1)
                }
            }
        })
        .await;
        assert_eq!(r.unwrap(), 1);
        // Un autre fil du même prestataire attend la même reprise avant sa
        // requête, sans échec de son côté.
        let r: Result<u8, RetryError<()>> =
            with_retry(&gate, &wait, || async { Attempt::Done(2) }).await;
        assert_eq!(r.unwrap(), 2);
        let waits = wait.0.lock().unwrap().clone();
        assert_eq!(waits.len(), 2);
        assert!(
            waits.iter().all(|d| d.as_secs() >= 29 && d.as_secs() <= 30),
            "{waits:?}"
        );
        assert!(gate.paused_seconds().is_some());
    }

    #[tokio::test]
    async fn une_erreur_definitive_ne_se_reprend_pas() {
        let gate = ProviderGate::new(1);
        let wait = Notee::default();
        let mut calls = 0;
        let r: Result<(), RetryError<&str>> = with_retry(&gate, &wait, || {
            calls += 1;
            async { Attempt::Fatal("403") }
        })
        .await;
        assert!(matches!(r, Err(RetryError::Fatal("403"))));
        assert_eq!(calls, 1);
        assert!(wait.0.lock().unwrap().is_empty());
    }

    #[test]
    fn une_tache_par_compte_tant_qu_elle_tourne() {
        let downloads = Arc::new(Downloads::default());
        let d = |s: &str| NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap();
        let p = (d("2020-01-01"), d("2020-01-30"));
        let h = downloads
            .start("paypal", 1, "Boutique", p, Some(3))
            .unwrap();
        // Une autre période du même compte est refusée, et le refus dit la
        // période de la tâche en cours (revue P1 de la 38-12b, M1).
        assert_eq!(
            downloads
                .start(
                    "paypal",
                    1,
                    " boutique ",
                    (d("2020-02-01"), d("2020-02-02")),
                    None
                )
                .err(),
            Some(RunningJob {
                id: h.id,
                from: p.0,
                to: p.1
            })
        );
        // Autre société, autre libellé, autre prestataire : libres.
        assert!(downloads.start("paypal", 2, "Boutique", p, None).is_ok());
        assert!(downloads.start("paypal", 1, "Autre", p, None).is_ok());
        assert!(downloads.start("wise", 1, "Boutique", p, None).is_ok());
        h.step_done();
        h.add_steps(1);
        let v = downloads.view("paypal", h.id, 1).unwrap();
        assert_eq!(
            (v.state, v.steps.done, v.steps.total),
            (JobState::Running, 1, Some(4))
        );
        assert!(downloads.view("paypal", h.id, 2).is_none(), "autre société");
        assert!(
            downloads.view("wise", h.id, 1).is_none(),
            "autre prestataire"
        );
        h.finish(
            JobState::Done,
            Some(serde_json::json!({ "ok": true })),
            None,
        );
        let v = downloads.view("paypal", h.id, 1).unwrap();
        assert_eq!(v.state, JobState::Done);
        // Finie, elle ne bloque plus la suivante ; une fin ne s'écrase pas.
        let h2 = downloads.start("paypal", 1, "Boutique", p, None).unwrap();
        assert!(h2.id > h.id);
        // Un total inconnu au départ se pose une fois connu (Stripe, 40-8b) ;
        // `add_steps` ne le crée pas.
        h2.add_steps(1);
        assert_eq!(
            downloads.view("paypal", h2.id, 1).unwrap().steps.total,
            None
        );
        h2.set_total(12);
        h2.step_done();
        let v = downloads.view("paypal", h2.id, 1).unwrap();
        assert_eq!((v.steps.done, v.steps.total), (1, Some(12)));
        h.finish(JobState::Failed, None, None);
        assert_eq!(
            downloads.view("paypal", h.id, 1).unwrap().state,
            JobState::Done
        );
    }

    #[tokio::test]
    async fn une_panique_conclut_la_tache_en_echec() {
        let downloads = Arc::new(Downloads::default());
        let jour = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let h = downloads
            .start("paypal", 1, "Boutique", (jour, jour), Some(1))
            .unwrap();
        let id = h.id;
        spawn_job(h, "paypal", async {
            panic!("panique de test");
        });
        for _ in 0..200 {
            let v = downloads.view("paypal", id, 1).unwrap();
            if v.state != JobState::Running {
                assert_eq!(v.state, JobState::Failed);
                let e = v.error.unwrap();
                assert_eq!(e["code"], "INTERNAL_ERROR");
                assert_eq!(e["details"]["provider"], "paypal");
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("tâche restée en cours");
    }
}
