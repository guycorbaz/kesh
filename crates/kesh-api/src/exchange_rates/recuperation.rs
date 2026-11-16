//! La récupération d'**une** unité de cours (genre et date) — le service
//! partagé par `POST /exchange-rates/fetch`, la tâche automatique et le
//! déclenchement à la connexion (Story 37-2, AC-3, AC-3 bis, AC-5).
//!
//! Sous le verrou `AppState::rate_fetch_lock`, pris **par unité** :
//! 1. raccourci d'un `POST /fetch` qui a **attendu** le verrou et trouve
//!    l'unité écrite par une récupération commencée après son arrivée ;
//! 2. mémoire des 15 minutes (`AppState::rate_fetch_memo`) : un
//!    `not_published` ou un échec ne se redemande pas avant ;
//! 3. pour la tâche, l'unité jugée manquante **sous le verrou** ;
//! 4. instant pris **avant** l'appel sortant, appel, analyse (`kesh-core`),
//!    contrôle du mois (`month_mismatch`) ou de la date (`date_mismatch`) ;
//! 5. une transaction sous `kesh_db::retry::retry_on_deadlock` : les valeurs
//!    publiées (règle (a)(b)(c)), puis la copie dans `exchange_rates` pour
//!    chaque société dont un exercice multi-devises ouvert a l'unité parmi ses
//!    unités utiles — et, pour un `POST /fetch`, pour la société de
//!    l'utilisateur.
//!
//! L'état (`_kesh_exchange_rate_fetch`) et l'audit sont à la charge de
//! l'appelant : un tour automatique écrit le pire statut de ses unités, un
//! `POST /fetch` le sien et sa ligne d'audit.

use std::collections::{BTreeSet, HashMap};
use std::time::{Duration, Instant};

use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike, Utc};
use kesh_core::exchange_rates::{
    PublishedRate, RateKind, RateUnit, RatesError, RejectedRate, check_daily_date,
    check_monthly_month, parse_daily_rates, parse_monthly_rates,
};
use kesh_db::entities::AUDIT_ENTITY_ID_NONE;
use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::errors::{DbError, map_db_error};
use kesh_db::repositories::audit_log;
use kesh_db::repositories::exchange_rates::{self as depot, FetchOutcomeStatus, SOURCE_AFC};

use crate::AppState;
use crate::audit::AuditActor;
use crate::exchange_rates::source::SourceError;
use crate::middleware::auth::CurrentUser;

/// Durée de la mémoire des `not_published` et des échecs (AC-5).
pub const MEMO_TTL: Duration = Duration::from_secs(15 * 60);

/// Pourquoi une unité n'a pas été récupérée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Echec {
    /// Pas encore publiée (racine vide, ou 404 de `xmldaily`).
    NotPublished,
    /// Source injoignable ou réponse inutilisable : le code classé
    /// (`timeout`, `connect`, `http_status:<n>`, `too_large`, `malformed`,
    /// `month_mismatch`, `date_mismatch`, `no_valid_rate`).
    Unavailable(String),
}

impl Echec {
    /// Statut de `_kesh_exchange_rate_fetch`.
    pub fn status(&self) -> FetchOutcomeStatus {
        match self {
            Self::NotPublished => FetchOutcomeStatus::NotPublished,
            Self::Unavailable(_) => FetchOutcomeStatus::Error,
        }
    }

    /// Code classé de `last_error` (`None` pour `not_published`).
    pub fn error_code(&self) -> Option<String> {
        match self {
            Self::NotPublished => None,
            Self::Unavailable(c) => Some(c.clone()),
        }
    }
}

impl From<SourceError> for Echec {
    fn from(e: SourceError) -> Self {
        match e {
            SourceError::NotPublished => Self::NotPublished,
            autre => Self::Unavailable(autre.code()),
        }
    }
}

impl From<RatesError> for Echec {
    fn from(e: RatesError) -> Self {
        match e {
            RatesError::NotPublished => Self::NotPublished,
            autre => Self::Unavailable(autre.code().to_string()),
        }
    }
}

/// Une entrée de la mémoire des 15 minutes.
#[derive(Debug, Clone)]
pub struct MemoEntry {
    /// Quand l'échec a été constaté.
    pub depuis: Instant,
    /// L'échec, rendu tel quel pendant [`MEMO_TTL`].
    pub echec: Echec,
}

/// La mémoire des 15 minutes, par `(genre, rate_date)` (`AppState::rate_fetch_memo`).
pub type RateFetchMemo = HashMap<(RateKind, NaiveDate), MemoEntry>;

/// Une unité récupérée (ou trouvée écrite, pour le raccourci).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recuperee {
    /// L'unité.
    pub unit: RateUnit,
    /// Date de publication lue (`<datum>`) d'un cours journalier.
    pub source_date: Option<NaiveDate>,
    /// Devises écrites (insérées, confirmées ou corrigées).
    pub count: usize,
    /// Devises écartées.
    pub rejected: usize,
    /// Devises corrigées (règle (c)).
    pub corrected: usize,
    /// Lignes copiées dans `exchange_rates`, toutes sociétés.
    pub copied: u64,
}

impl Recuperee {
    /// Code de `last_error` d'une réussite : `invalid_rates:<n>` si des
    /// devises ont été écartées.
    pub fn error_code(&self) -> Option<String> {
        (self.rejected > 0).then(|| format!("invalid_rates:{}", self.rejected))
    }
}

/// L'issue d'une demande.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Issue {
    /// Appel fait, unité écrite.
    Recuperee(Recuperee),
    /// `POST /fetch` qui a attendu : l'unité venait d'être écrite ; aucun appel.
    Raccourci(Recuperee),
    /// Appel fait, échec (mémorisé 15 minutes).
    Echouee(Echec),
    /// Échec mémorisé rendu sans appel.
    Memorisee(Echec),
    /// Tâche : l'unité n'était plus manquante sous le verrou ; aucun appel.
    DejaPresente,
}

/// Ce qu'un `POST /fetch` ajoute à une demande.
#[derive(Debug, Clone)]
pub struct Manuelle {
    /// L'utilisateur : sa société reçoit toujours la copie, et la ligne
    /// d'audit `exchange_rates.fetched` lui est attribuée.
    pub utilisateur: CurrentUser,
    /// L'instant d'arrivée de la requête (UTC, à la milliseconde) : le
    /// raccourci de l'AC-3.
    pub arrivee: NaiveDateTime,
}

/// Une demande de récupération d'une unité.
#[derive(Debug, Clone)]
pub struct Demande {
    /// L'unité.
    pub unit: RateUnit,
    /// Tâche : ne récupérer que si l'unité manque encore, jugé sous le verrou
    /// (faux pour la revérification du mois courant et pour un `POST /fetch`).
    pub seulement_si_manquante: bool,
    /// `POST /fetch` ; `None` pour un tour automatique (non audité).
    pub manuelle: Option<Manuelle>,
}

/// L'instant présent en UTC, à la milliseconde (précision de `DATETIME(3)`).
pub(crate) fn maintenant_ms() -> NaiveDateTime {
    let t = Utc::now().naive_utc();
    t.with_nanosecond(t.nanosecond() / 1_000_000 * 1_000_000)
        .unwrap_or(t)
}

/// Lit l'entrée de mémoire de l'unité, encore valable ; retire une entrée
/// expirée.
fn memo_lire(state: &AppState, unit: RateUnit) -> Option<Echec> {
    let mut memo = state
        .rate_fetch_memo
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let cle = (unit.kind, unit.date);
    match memo.get(&cle) {
        Some(e) if e.depuis.elapsed() < MEMO_TTL => Some(e.echec.clone()),
        Some(_) => {
            memo.remove(&cle);
            None
        }
        None => None,
    }
}

fn memo_ecrire(state: &AppState, unit: RateUnit, echec: Option<Echec>) {
    let mut memo = state
        .rate_fetch_memo
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let cle = (unit.kind, unit.date);
    match echec {
        Some(echec) => {
            memo.insert(
                cle,
                MemoEntry {
                    depuis: Instant::now(),
                    echec,
                },
            );
        }
        None => {
            memo.remove(&cle);
        }
    }
}

/// Ce que la réponse de la source a rendu, analysé et contrôlé.
struct Analyse {
    source_date: Option<NaiveDate>,
    rates: Vec<PublishedRate>,
    rejected: Vec<RejectedRate>,
}

/// Appelle la source, analyse et contrôle la réponse.
async fn appeler(state: &AppState, unit: RateUnit) -> Result<Analyse, Echec> {
    match unit.kind {
        RateKind::Month => {
            let corps = state
                .rate_source
                .fetch_month(unit.date.year(), unit.date.month())
                .await?;
            let m = parse_monthly_rates(corps.as_bytes())?;
            check_monthly_month(&m, unit.date)?;
            Ok(Analyse {
                source_date: None,
                rates: m.rates,
                rejected: m.rejected,
            })
        }
        RateKind::Day => {
            let corps = state.rate_source.fetch_day(unit.date).await?;
            let j = parse_daily_rates(corps.as_bytes())?;
            check_daily_date(&j, unit.date)?;
            Ok(Analyse {
                source_date: Some(j.date),
                rates: j.rates,
                rejected: j.rejected,
            })
        }
    }
}

/// Copie l'unité pour une société, dans sa propre transaction rejouée
/// (raccourci du `POST /fetch`).
async fn copier_pour(state: &AppState, unit: RateUnit, company_id: i64) -> Result<u64, DbError> {
    kesh_db::retry::retry_on_deadlock("exchange_rates::copy_fetched", || {
        let pool = state.pool.clone();
        async move {
            let mut tx = pool.begin().await.map_err(map_db_error)?;
            let n = depot::copy_fetched_in_tx(&mut tx, company_id, &[unit], None).await?;
            tx.commit().await.map_err(map_db_error)?;
            Ok(n)
        }
    })
    .await
}

/// Écrit les valeurs publiées, les copies et — pour un `POST /fetch` — la
/// ligne d'audit `exchange_rates.fetched`, en **une** transaction rejouée :
/// l'audit n'existe que si les cours sont écrits, et inversement.
async fn ecrire(
    state: &AppState,
    unit: RateUnit,
    analyse: &Analyse,
    fetched_at: NaiveDateTime,
    manuelle: Option<&Manuelle>,
) -> Result<(depot::FetchWriteOutcome, u64), DbError> {
    kesh_db::retry::retry_on_deadlock("exchange_rates::fetch", || {
        let pool = state.pool.clone();
        async move {
            let mut tx = pool.begin().await.map_err(map_db_error)?;
            let outcome = depot::write_fetched_unit_in_tx(
                &mut tx,
                SOURCE_AFC,
                unit,
                analyse.source_date,
                &analyse.rates,
                fetched_at,
            )
            .await?;
            let mut societes: BTreeSet<i64> = depot::companies_with_unit_in_tx(&mut tx, unit)
                .await?
                .into_iter()
                .collect();
            societes.extend(manuelle.map(|m| m.utilisateur.company_id));
            let mut copied = 0;
            for company_id in societes {
                copied += depot::copy_fetched_in_tx(&mut tx, company_id, &[unit], None).await?;
            }
            if let Some(m) = manuelle {
                audit_log::insert_in_tx(
                    &mut tx,
                    NewAuditLogEntry::from_current_user(
                        &m.utilisateur,
                        "exchange_rates.fetched",
                        "installation",
                        AUDIT_ENTITY_ID_NONE,
                        Some(serde_json::json!({
                            "kind": unit.kind.as_str(),
                            "rateDate": unit.date.format("%Y-%m-%d").to_string(),
                            "sourceDate": analyse
                                .source_date
                                .map(|d| d.format("%Y-%m-%d").to_string()),
                            "count": outcome.written(),
                            "rejected": analyse.rejected.len(),
                            "corrected": outcome.corrected.len(),
                            "copied": copied,
                        })),
                    ),
                )
                .await?;
            }
            tx.commit().await.map_err(map_db_error)?;
            Ok((outcome, copied))
        }
    })
    .await
}

/// Récupère **une** unité (cf. doc du module). Une erreur de base remonte ;
/// un échec de la source est une [`Issue`], jamais une erreur.
pub async fn recuperer_unite(state: &AppState, demande: Demande) -> Result<Issue, DbError> {
    let unit = demande.unit;
    let (_verrou, attendu) = match state.rate_fetch_lock.try_lock() {
        Ok(g) => (g, false),
        Err(_) => (state.rate_fetch_lock.lock().await, true),
    };

    // 1. Raccourci (AC-3) : un `POST /fetch` qui a attendu trouve l'unité
    //    écrite par une récupération commencée après son arrivée.
    if let Some(m) = &demande.manuelle
        && attendu
    {
        let mut conn = state.pool.acquire().await.map_err(map_db_error)?;
        let resume = depot::fetched_unit_summary(&mut conn, SOURCE_AFC, unit).await?;
        drop(conn);
        if let Some(r) = resume
            && r.last_fetched_at >= m.arrivee
        {
            let copied = copier_pour(state, unit, m.utilisateur.company_id).await?;
            return Ok(Issue::Raccourci(Recuperee {
                unit,
                source_date: r.source_date,
                count: usize::try_from(r.count).unwrap_or(0),
                rejected: 0,
                corrected: 0,
                copied,
            }));
        }
    }

    // 2. Mémoire des 15 minutes.
    if let Some(echec) = memo_lire(state, unit) {
        return Ok(Issue::Memorisee(echec));
    }

    // 3. La tâche juge « manque » sous le verrou.
    if demande.seulement_si_manquante {
        let mut conn = state.pool.acquire().await.map_err(map_db_error)?;
        if depot::missing_units(&mut conn, &[unit]).await?.is_empty() {
            return Ok(Issue::DejaPresente);
        }
    }

    // 4. Appel, analyse, contrôles.
    let fetched_at = maintenant_ms();
    let analyse = match appeler(state, unit).await {
        Ok(a) => a,
        Err(echec) => {
            memo_ecrire(state, unit, Some(echec.clone()));
            return Ok(Issue::Echouee(echec));
        }
    };

    // 5. Écriture.
    let (outcome, copied) =
        ecrire(state, unit, &analyse, fetched_at, demande.manuelle.as_ref()).await?;
    memo_ecrire(state, unit, None);

    for c in &outcome.corrected {
        tracing::warn!(
            currency = %c.currency,
            kind = %unit.kind,
            rate_date = %unit.date,
            ancienne_unite = c.old_unit,
            ancien_cours = %c.old_rate,
            nouvelle_unite = c.new_unit,
            nouveau_cours = %c.new_rate,
            "cours publié corrigé par la source"
        );
    }
    if !analyse.rejected.is_empty() {
        let ecartees: Vec<String> = analyse
            .rejected
            .iter()
            .map(|r| format!("{}:{}", r.code, r.reason.as_str()))
            .collect();
        tracing::warn!(
            kind = %unit.kind,
            rate_date = %unit.date,
            ecartees = ?ecartees,
            "devises écartées de la réponse de la source"
        );
    }
    Ok(Issue::Recuperee(Recuperee {
        unit,
        source_date: analyse.source_date,
        count: outcome.written(),
        rejected: analyse.rejected.len(),
        corrected: outcome.corrected.len(),
        copied,
    }))
}
