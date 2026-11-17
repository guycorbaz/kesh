//! Routes des cours de change (Story 37-2, AC-5).
//!
//! - `POST /api/v1/exchange-rates/fetch` — récupérer les cours d'un mois
//!   (`{year, month}`) ou d'un jour (`{date}`), Administrateur ou Comptable ;
//! - `GET /api/v1/exchange-rates/status` — l'état de la dernière
//!   récupération, tout rôle authentifié ;
//! - `GET /api/v1/exchange-rates?month=…|date=…[&currency=]` — les cases d'un
//!   mois ou d'un jour, tout rôle ;
//! - `GET /api/v1/exchange-rates/lookup` — le **cours effectif** (cours retenu
//!   par la société), en lecture seule, tout rôle ;
//! - `PUT /api/v1/exchange-rates/rates` — saisir un cours (`manual`) ;
//! - `POST /api/v1/exchange-rates/rates/reset` — le ↑, reprendre la valeur
//!   publiée (ou retirer un cours saisi sans valeur publiée) ;
//! - `GET /api/v1/exchange-rates/notices` et
//!   `POST /api/v1/exchange-rates/notices/acknowledge` — corrections de l'AFC à
//!   annoncer à la société, et leur acquittement ;
//! - `GET /api/v1/fiscal-years/{id}/exchange-rates` — le tableau annuel d'un
//!   exercice, tout rôle.
//!
//! Les routes mutantes (Administrateur ou Comptable) écrivent en transaction
//! avec leur audit (`from_current_user`), rejouée sur interblocage
//! (`kesh_db::retry::retry_on_deadlock`).
//!
//! Toutes exigent une société **multi-devises** (`409
//! MULTI_CURRENCY_DISABLED`), dans l'ordre des refus de l'AC-5 :
//! authentification et rôle (401, 403 — couches du routeur) → société
//! mono-devise (409) → paramètres illisibles ou hors bornes (400, dont `CHF`)
//! → devise inconnue (`400 CURRENCY_UNKNOWN`, lue telle quelle) → cours hors
//! bornes (`400 FX_RATE_INVALID`) → absence (404) → version (409). Seul écart :
//! un corps JSON illisible est refusé par l'extracteur (400) avant le handler ;
//! les champs se lisent en chaînes et se jugent ensuite. Le tableau annuel
//! juge l'exercice (404 d'une autre société, puis 409 mono-devise avec
//! `fiscalYearId`).

use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use chrono::{DateTime, NaiveDate, Utc};
use kesh_core::exchange_rates::{
    MIN_RATE_DATE, RateKind, RateUnit, check_rate_bounds, exercise_months, first_of_month,
    next_month,
};
use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::errors::{DbError, map_db_error};
use kesh_db::repositories::audit_log;
use kesh_db::repositories::exchange_rates::{
    self as depot, CompanyRate, FetchOutcomeStatus, FetchStatus, FetchTrigger, FetchedRate,
    NoticeAck, RateCell, RateChange, SOURCE_AFC,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::AppState;
use crate::audit::AuditActor;
use crate::errors::{AppError, t};
use crate::exchange_rates::recuperation::{self, Demande, Echec, Issue, Manuelle};
use crate::extractors::impl_validation_extractor;
use crate::middleware::auth::CurrentUser;

/// Refuse une société mono-devise : `409 MULTI_CURRENCY_DISABLED` (forme de
/// 37-1, `details` vide). Partagé par les routes des cours.
pub(crate) async fn exiger_multi_devises(
    state: &AppState,
    company_id: i64,
) -> Result<(), AppError> {
    let societe = kesh_db::repositories::companies::find_by_id(&state.pool, company_id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;
    if !societe.multi_currency_enabled {
        return Err(AppError::Database(DbError::MultiCurrencyDisabled {
            fiscal_year_id: None,
        }));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// POST /exchange-rates/fetch
// ---------------------------------------------------------------------------

/// Corps de `POST /exchange-rates/fetch` : `{year, month}` **ou** `{date}`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchRatesBody {
    /// Année du mois demandé.
    pub year: Option<i32>,
    /// Mois demandé (1 à 12).
    pub month: Option<u32>,
    /// Jour demandé, `AAAA-MM-JJ`.
    pub date: Option<String>,
}

impl_validation_extractor!(
    FetchRatesBodyExtractor,
    FetchRatesBody,
    "`year` i32 et `month` 1..12, ou `date` \"AAAA-MM-JJ\""
);

/// Réponse 200 de `POST /exchange-rates/fetch`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchRatesResponse {
    /// `month` ou `day`.
    pub kind: RateKind,
    /// 1er du mois ou jour demandé, `AAAA-MM-JJ`.
    pub rate_date: NaiveDate,
    /// Date de publication lue (`<datum>`) d'un cours journalier.
    pub source_date: Option<NaiveDate>,
    /// Devises écrites.
    pub count: usize,
    /// Devises écartées (codes au journal du serveur).
    pub rejected: usize,
    /// Devises corrigées (valeur publiée différente de celle déjà connue).
    pub corrected: usize,
}

/// L'unité demandée, bornes de l'AC-5 contrôlées au jour de Zurich
/// `aujourdhui` : mois de 2000-01 au mois suivant le mois courant, à toute
/// date ; jour de 2000-01-01 à la veille (strictement antérieur à
/// aujourd'hui). `Err` : le message du 400.
pub fn unite_demandee(body: &FetchRatesBody, aujourdhui: NaiveDate) -> Result<RateUnit, String> {
    match (body.year, body.month, body.date.as_deref()) {
        (Some(y), Some(m), None) => {
            let premier = NaiveDate::from_ymd_opt(y, m, 1)
                .ok_or_else(|| "mois invalide : `month` de 1 à 12".to_string())?;
            if premier < MIN_RATE_DATE || premier > next_month(aujourdhui) {
                return Err(
                    "mois hors bornes : de 2000-01 au mois suivant le mois courant".to_string(),
                );
            }
            Ok(RateUnit::month_of(premier))
        }
        (None, None, Some(texte)) => {
            let jour = lire_date(texte).ok_or_else(|| "date illisible : AAAA-MM-JJ".to_string())?;
            if jour < MIN_RATE_DATE || jour >= aujourdhui {
                return Err(
                    "date hors bornes : de 2000-01-01 à hier (cours du jour pas encore publié)"
                        .to_string(),
                );
            }
            Ok(RateUnit::day(jour))
        }
        _ => Err("demander soit `year` et `month`, soit `date`".to_string()),
    }
}

/// Lit une date `AAAA-MM-JJ` stricte (dix caractères, chiffres et tirets).
fn lire_date(texte: &str) -> Option<NaiveDate> {
    let b = texte.as_bytes();
    let forme = b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit());
    if !forme {
        return None;
    }
    NaiveDate::parse_from_str(texte, "%Y-%m-%d").ok()
}

/// La traduction HTTP d'un échec de récupération : 404 ou 503, jamais 500.
fn refus(echec: &Echec) -> AppError {
    match echec {
        Echec::NotPublished => AppError::RateNotPublished,
        Echec::Unavailable(code) => AppError::RateSourceUnavailable(code.clone()),
    }
}

/// Le travail d'un `POST /fetch` — verrou, appel, transaction (cours, copies,
/// audit), état — exécuté dans une tâche que le handler attend.
async fn recuperer_et_noter(
    state: &AppState,
    utilisateur: CurrentUser,
    unit: RateUnit,
    arrivee: chrono::NaiveDateTime,
) -> Result<FetchRatesResponse, AppError> {
    let issue = recuperation::recuperer_unite(
        state,
        Demande {
            unit,
            seulement_si_manquante: false,
            manuelle: Some(Manuelle {
                utilisateur,
                arrivee,
            }),
        },
    )
    .await?;
    let (recuperee, ecrire_etat) = match issue {
        Issue::Recuperee(r) => (Ok(r), true),
        Issue::Raccourci(r) => (Ok(r), false),
        Issue::Echouee(e) => (Err(e), true),
        Issue::Memorisee(e) => (Err(e), false),
        Issue::DejaPresente => {
            return Err(AppError::Internal(
                "récupération manuelle jugée « déjà présente »".into(),
            ));
        }
    };
    if ecrire_etat {
        let (status, error) = match &recuperee {
            Ok(r) => (FetchOutcomeStatus::Ok, r.error_code()),
            Err(e) => (e.status(), e.error_code()),
        };
        let mut conn = state.pool.acquire().await.map_err(map_db_error)?;
        depot::record_fetch_status(
            &mut conn,
            &FetchStatus {
                trigger: FetchTrigger::Manual,
                last_run_at: recuperation::maintenant_ms(),
                status,
                error,
                kind: unit.kind,
                rate_date: unit.date,
            },
        )
        .await?;
    }
    let r = recuperee.map_err(|e| refus(&e))?;
    Ok(FetchRatesResponse {
        kind: unit.kind,
        rate_date: unit.date,
        source_date: r.source_date,
        count: r.count,
        rejected: r.rejected,
        corrected: r.corrected,
    })
}

/// `POST /api/v1/exchange-rates/fetch` — récupère les cours d'un mois ou d'un
/// jour (AC-5) : 200 `{kind, rateDate, sourceDate, count, rejected,
/// corrected}`, 404 `RATE_NOT_PUBLISHED`, 503 `RATE_SOURCE_UNAVAILABLE`.
///
/// Le travail tourne dans un `tokio::spawn` attendu (comme « Sauvegarder
/// maintenant ») : l'abandon du client n'interrompt pas une récupération
/// entre l'écriture des cours et celle de l'état. Les cours, les copies et la
/// ligne d'audit `exchange_rates.fetched` s'écrivent en une transaction
/// rejouée sur interblocage (`kesh_db::retry::retry_on_deadlock`, dans
/// `exchange_rates::recuperation`).
pub async fn fetch_rates(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    FetchRatesBodyExtractor(body): FetchRatesBodyExtractor,
) -> Result<Json<FetchRatesResponse>, AppError> {
    let arrivee = recuperation::maintenant_ms();
    exiger_multi_devises(&state, current_user.company_id).await?;
    let unit = unite_demandee(&body, state.horloge.aujourd_hui()).map_err(AppError::Validation)?;
    let etat = state.clone();
    let tache =
        tokio::spawn(async move { recuperer_et_noter(&etat, current_user, unit, arrivee).await });
    let reponse = tache
        .await
        .map_err(|e| AppError::Internal(format!("tâche de récupération des cours : {e}")))??;
    Ok(Json(reponse))
}

// ---------------------------------------------------------------------------
// GET /exchange-rates/status
// ---------------------------------------------------------------------------

/// Réponse de `GET /exchange-rates/status` : tous les champs à `null` tant
/// qu'aucune récupération n'a eu lieu.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchStatusResponse {
    /// `auto` ou `manual`.
    pub trigger: Option<&'static str>,
    /// Fin de la dernière récupération (UTC).
    pub last_run_at: Option<DateTime<Utc>>,
    /// `ok`, `not_published` ou `error`.
    pub status: Option<&'static str>,
    /// Code classé (`timeout`, `invalid_rates:<n>`…).
    pub error: Option<String>,
    /// Genre de l'unité en cause.
    pub kind: Option<RateKind>,
    /// Date de l'unité en cause.
    pub rate_date: Option<NaiveDate>,
}

/// `GET /api/v1/exchange-rates/status` — état de la dernière récupération.
pub async fn get_fetch_status(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<FetchStatusResponse>, AppError> {
    exiger_multi_devises(&state, current_user.company_id).await?;
    let mut conn = state.pool.acquire().await.map_err(map_db_error)?;
    let reponse = match depot::get_fetch_status(&mut conn).await? {
        None => FetchStatusResponse::default(),
        Some(s) => FetchStatusResponse {
            trigger: Some(s.trigger.as_str()),
            last_run_at: Some(s.last_run_at.and_utc()),
            status: Some(s.status.as_str()),
            error: s.error,
            kind: Some(s.kind),
            rate_date: Some(s.rate_date),
        },
    };
    Ok(Json(reponse))
}

// ---------------------------------------------------------------------------
// Lecture et jugement des paramètres (lus en chaînes, AC-5)
// ---------------------------------------------------------------------------

/// Plus grande date d'un cours (AC-5 : 2000-01-01 ≤ `rateDate` ≤ 2099-12-31).
const MAX_RATE_DATE: NaiveDate = match NaiveDate::from_ymd_opt(2099, 12, 31) {
    Some(d) => d,
    None => panic!("2099-12-31 est une date"),
};

/// Plus grand nombre d'entrées d'un acquittement.
pub const MAX_ACKNOWLEDGE_NOTICES: usize = 500;

/// Lit un genre `month` ou `day` (exact) ; sinon 400.
fn lire_genre(texte: &str) -> Result<RateKind, AppError> {
    RateKind::from_str(texte)
        .map_err(|_| AppError::Validation("`kind` : \"month\" ou \"day\"".to_string()))
}

/// Lit une date `AAAA-MM-JJ` dans les bornes d'un cours ; sinon 400.
fn lire_jour_borne(texte: &str, champ: &str) -> Result<NaiveDate, AppError> {
    let jour = lire_date(texte)
        .ok_or_else(|| AppError::Validation(format!("`{champ}` illisible : AAAA-MM-JJ")))?;
    if !(MIN_RATE_DATE..=MAX_RATE_DATE).contains(&jour) {
        return Err(AppError::Validation(format!(
            "`{champ}` hors bornes : de 2000-01-01 à 2099-12-31"
        )));
    }
    Ok(jour)
}

/// La date d'un cours retenu (`rateDate` d'un corps) : bornée, et le 1er du
/// mois pour un cours mensuel ; sinon 400.
fn lire_date_de_cours(texte: &str, kind: RateKind) -> Result<NaiveDate, AppError> {
    let jour = lire_jour_borne(texte, "rateDate")?;
    if kind == RateKind::Month && first_of_month(jour) != jour {
        return Err(AppError::Validation(
            "`rateDate` d'un cours mensuel : le 1er du mois".to_string(),
        ));
    }
    Ok(jour)
}

/// Lit un mois `AAAA-MM` borné ; rend son 1er. Sinon 400.
fn lire_mois_borne(texte: &str) -> Result<NaiveDate, AppError> {
    let b = texte.as_bytes();
    let forme = b.len() == 7
        && b[4] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 4 || c.is_ascii_digit());
    let premier = forme
        .then(|| NaiveDate::parse_from_str(&format!("{texte}-01"), "%Y-%m-%d").ok())
        .flatten()
        .ok_or_else(|| AppError::Validation("`month` illisible : AAAA-MM".to_string()))?;
    if !(MIN_RATE_DATE..=MAX_RATE_DATE).contains(&premier) {
        return Err(AppError::Validation(
            "`month` hors bornes : de 2000-01 à 2099-12".to_string(),
        ));
    }
    Ok(premier)
}

/// Premier temps du jugement d'une devise, avec les paramètres : `CHF`
/// (devise principale, sans cours) → 400 `Validation`, sans code dédié.
fn refuser_chf(devise: &str) -> Result<(), AppError> {
    if devise == kesh_core::currency::BASE_CURRENCY {
        return Err(AppError::Validation(
            "`currency` : CHF est la devise principale, elle n'a pas de cours".to_string(),
        ));
    }
    Ok(())
}

/// Second temps, après les autres paramètres : la devise, lue **telle
/// quelle** (sans normalisation : `usd` est refusée), doit être de la liste
/// ISO 4217 de 37-1 → sinon `400 CURRENCY_UNKNOWN`.
fn exiger_devise_connue(devise: &str) -> Result<(), AppError> {
    if kesh_core::currency::ISO_4217_CURRENCIES
        .binary_search(&devise)
        .is_err()
    {
        return Err(AppError::Database(DbError::CurrencyUnknown {
            value: devise.to_string(),
        }));
    }
    Ok(())
}

/// Lit un cours en chaîne décimale (`-`, chiffres, au plus un point) ;
/// illisible → 400 `Validation`. Les bornes (signe, décimales, 10⁶) se jugent
/// après la devise, par [`juger_cours`].
fn lire_cours(texte: &str, champ: &str) -> Result<Decimal, AppError> {
    let corps = texte.strip_prefix('-').unwrap_or(texte);
    let mut parties = corps.splitn(2, '.');
    let entier = parties.next().unwrap_or("");
    let fraction = parties.next();
    let forme = !entier.is_empty()
        && entier.bytes().all(|c| c.is_ascii_digit())
        && fraction.is_none_or(|f| !f.is_empty() && f.bytes().all(|c| c.is_ascii_digit()));
    if !forme {
        return Err(AppError::Validation(format!(
            "`{champ}` illisible : un nombre décimal en chaîne, p. ex. \"0.9123\""
        )));
    }
    Decimal::from_str(texte).map_err(|_| AppError::Validation(format!("`{champ}` illisible")))
}

/// Bornes d'un cours seul (AC-2) → `400 FX_RATE_INVALID`, `details {currency}`.
fn juger_cours(cours: &Decimal, devise: &str) -> Result<(), AppError> {
    check_rate_bounds(cours).map_err(|_| AppError::FxRateInvalid {
        currency: devise.to_string(),
    })
}

// ---------------------------------------------------------------------------
// La case (`Cell`, AC-5) — une seule forme pour le tableau annuel, la liste et
// les réponses du `PUT` et du ↑
// ---------------------------------------------------------------------------

/// La ligne `exchange_rates` d'une case.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CellRow {
    /// `exchange_rates.id`.
    pub id: i64,
    /// Verrou optimiste, à renvoyer au `PUT` et au ↑.
    pub version: i32,
    /// Dernière modification (UTC).
    pub updated_at: DateTime<Utc>,
    /// Vrai pour une ligne écrite par Kesh (copie, `updated_by IS NULL`).
    pub updated_by_kesh: bool,
}

/// La dernière valeur publiée d'une case, pour information.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CellPublished {
    /// Cours par unité.
    pub rate: Decimal,
    /// Unité publiée.
    pub published_unit: i32,
    /// Cours publié pour l'unité.
    pub published_rate: Decimal,
    /// Date de publication lue (`day`), `null` pour un mois.
    pub source_date: Option<NaiveDate>,
    /// Dernière récupération qui l'a confirmée (UTC).
    pub fetched_at: DateTime<Utc>,
    /// Récupération qui a apporté la valeur actuelle (UTC).
    pub value_changed_at: DateTime<Utc>,
}

/// Une case : le cours retenu par la société (une seule valeur, U1) et la
/// dernière valeur publiée.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cell {
    /// `month` ou `day`.
    pub kind: RateKind,
    /// 1er du mois ou jour.
    pub rate_date: NaiveDate,
    /// Le cours retenu par la société ; `null` sans ligne.
    pub effective_rate: Option<Decimal>,
    /// Sa source (`afc`, `manual`, ou la valeur lue telle quelle) ; `null`
    /// sans ligne.
    pub source: Option<String>,
    /// La ligne `exchange_rates`, `null` sans ligne.
    pub row: Option<CellRow>,
    /// La dernière valeur publiée, `null` sans valeur.
    pub published: Option<CellPublished>,
    /// Ligne `afc` dont la valeur publiée diffère (correction non reprise).
    pub corrected: bool,
    /// [`Cell::corrected`] et acquittée à cette valeur publiée.
    pub acknowledged: bool,
}

impl Cell {
    /// La case d'une ligne et d'une valeur publiée (l'une ou l'autre, ou
    /// aucune).
    fn de(
        kind: RateKind,
        rate_date: NaiveDate,
        company: Option<&CompanyRate>,
        published: Option<&FetchedRate>,
    ) -> Self {
        let rc = RateCell {
            currency: String::new(),
            kind,
            rate_date,
            company: company.cloned(),
            published: published.cloned(),
        };
        Self {
            kind,
            rate_date,
            effective_rate: company.map(|c| c.rate),
            source: company.map(|c| c.source.as_str().to_string()),
            row: company.map(|c| CellRow {
                id: c.id,
                version: c.version,
                updated_at: c.updated_at.and_utc(),
                updated_by_kesh: c.updated_by.is_none(),
            }),
            published: published.map(|p| CellPublished {
                rate: p.rate_per_unit,
                published_unit: p.published_unit,
                published_rate: p.published_rate,
                source_date: p.source_date,
                fetched_at: p.fetched_at.and_utc(),
                value_changed_at: p.value_changed_at.and_utc(),
            }),
            corrected: rc.corrected(),
            acknowledged: rc.acknowledged(),
        }
    }

    /// La case d'un [`RateCell`] de la lecture groupée, ou une case vide.
    fn depuis(kind: RateKind, rate_date: NaiveDate, cell: Option<&RateCell>) -> Self {
        Self::de(
            kind,
            rate_date,
            cell.and_then(|c| c.company.as_ref()),
            cell.and_then(|c| c.published.as_ref()),
        )
    }
}

/// Une case avec sa devise (`GET /exchange-rates`).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyCell {
    /// Code ISO 4217.
    pub currency: String,
    /// La case.
    #[serde(flatten)]
    pub cell: Cell,
}

// ---------------------------------------------------------------------------
// GET /exchange-rates
// ---------------------------------------------------------------------------

/// Paramètres de `GET /exchange-rates` (lus en chaînes).
#[derive(Debug, Deserialize)]
pub struct ListRatesQuery {
    /// Mois `AAAA-MM`.
    pub month: Option<String>,
    /// Jour `AAAA-MM-JJ`.
    pub date: Option<String>,
    /// Une seule devise.
    pub currency: Option<String>,
}

/// Réponse de `GET /exchange-rates`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRatesResponse {
    /// `month` ou `day`.
    pub kind: RateKind,
    /// 1er du mois ou jour.
    pub rate_date: NaiveDate,
    /// Une case par devise (ligne de la société ou valeur publiée), triées.
    pub rates: Vec<CurrencyCell>,
}

/// `GET /api/v1/exchange-rates?month=AAAA-MM|date=AAAA-MM-JJ[&currency=USD]`
/// — toutes les devises qui ont une ligne de la société **ou** une valeur
/// publiée pour ce mois ou ce jour (union), ou une seule ; lecture seule.
pub async fn list_rates(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(q): Query<ListRatesQuery>,
) -> Result<Json<ListRatesResponse>, AppError> {
    exiger_multi_devises(&state, current_user.company_id).await?;
    let (kind, rate_date) = match (q.month.as_deref(), q.date.as_deref()) {
        (Some(m), None) => (RateKind::Month, lire_mois_borne(m)?),
        (None, Some(d)) => (RateKind::Day, lire_jour_borne(d, "date")?),
        _ => {
            return Err(AppError::Validation(
                "demander soit `month` (AAAA-MM), soit `date` (AAAA-MM-JJ)".to_string(),
            ));
        }
    };
    if let Some(c) = q.currency.as_deref() {
        refuser_chf(c)?;
        exiger_devise_connue(c)?;
    }
    let mut conn = state.pool.acquire().await.map_err(map_db_error)?;
    let cells = depot::list_cells_in_tx(
        &mut conn,
        current_user.company_id,
        kind,
        rate_date,
        rate_date,
    )
    .await?;
    let rates = cells
        .iter()
        .filter(|c| q.currency.as_deref().is_none_or(|d| c.currency == d))
        .map(|c| CurrencyCell {
            currency: c.currency.clone(),
            cell: Cell::depuis(kind, rate_date, Some(c)),
        })
        .collect();
    Ok(Json(ListRatesResponse {
        kind,
        rate_date,
        rates,
    }))
}

// ---------------------------------------------------------------------------
// GET /exchange-rates/lookup
// ---------------------------------------------------------------------------

/// Paramètres de `GET /exchange-rates/lookup` (lus en chaînes).
#[derive(Debug, Deserialize)]
pub struct LookupQuery {
    /// Code ISO 4217, lu tel quel.
    pub currency: Option<String>,
    /// Date `AAAA-MM-JJ` (n'importe quel jour du mois pour `month`).
    pub date: Option<String>,
    /// `month` (défaut) ou `day`.
    pub kind: Option<String>,
}

/// Réponse 200 de `GET /exchange-rates/lookup` — forme lue par 37-3b et 37-5.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LookupResponse {
    /// Code ISO 4217.
    pub currency: String,
    /// Genre.
    pub kind: RateKind,
    /// 1er du mois (`month`) ou jour (`day`).
    pub rate_date: NaiveDate,
    /// Le cours retenu par la société (cours effectif).
    pub effective_rate: Decimal,
    /// Sa source.
    pub source: String,
    /// Dernière valeur publiée par unité, pour information ; `null` si aucune.
    pub published_rate: Option<Decimal>,
    /// Unité publiée ; `null` si aucune.
    pub published_unit: Option<i32>,
    /// Date de publication lue, `null` hors `day`.
    pub source_date: Option<NaiveDate>,
}

/// `GET /api/v1/exchange-rates/lookup?currency=USD&date=AAAA-MM-JJ[&kind=month|day]`
/// — le **cours effectif** (cours retenu par la société) du mois de la date
/// ou du jour, en lecture seule, sans appel sortant ni écriture ni audit.
/// Aucune ligne de la société → `404 RATE_UNAVAILABLE` (forme unique de
/// `DbError::RateUnavailable` pour un mois ; la même plus `date` pour un
/// jour), même si une valeur publiée existe.
pub async fn lookup_rate(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(q): Query<LookupQuery>,
) -> Result<Json<LookupResponse>, AppError> {
    exiger_multi_devises(&state, current_user.company_id).await?;
    let kind = match q.kind.as_deref() {
        None => RateKind::Month,
        Some(k) => lire_genre(k)?,
    };
    let jour = lire_jour_borne(
        q.date
            .as_deref()
            .ok_or_else(|| AppError::Validation("`date` absente : AAAA-MM-JJ".to_string()))?,
        "date",
    )?;
    let devise = q
        .currency
        .as_deref()
        .ok_or_else(|| AppError::Validation("`currency` absente".to_string()))?;
    refuser_chf(devise)?;
    exiger_devise_connue(devise)?;
    let rate_date = match kind {
        RateKind::Month => first_of_month(jour),
        RateKind::Day => jour,
    };

    let mut conn = state.pool.acquire().await.map_err(map_db_error)?;
    let Some(retenu) =
        depot::find_company_rate_in_tx(&mut conn, current_user.company_id, devise, kind, rate_date)
            .await?
    else {
        return Err(match kind {
            RateKind::Month => AppError::Database(DbError::RateUnavailable {
                currency: devise.to_string(),
                month: rate_date,
            }),
            RateKind::Day => AppError::RateUnavailableDay {
                currency: devise.to_string(),
                date: rate_date,
            },
        });
    };
    let publie =
        depot::find_fetched_rate_in_tx(&mut conn, SOURCE_AFC, devise, kind, rate_date).await?;
    Ok(Json(LookupResponse {
        currency: devise.to_string(),
        kind,
        rate_date,
        effective_rate: retenu.rate,
        source: retenu.source.as_str().to_string(),
        published_rate: publie.as_ref().map(|p| p.rate_per_unit),
        published_unit: publie.as_ref().map(|p| p.published_unit),
        source_date: if kind == RateKind::Day {
            publie.as_ref().and_then(|p| p.source_date)
        } else {
            None
        },
    }))
}

// ---------------------------------------------------------------------------
// PUT /exchange-rates/rates et POST /exchange-rates/rates/reset
// ---------------------------------------------------------------------------

/// Corps de `PUT /exchange-rates/rates` — champs lus en chaînes.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PutRateBody {
    /// Code ISO 4217, lu tel quel.
    pub currency: String,
    /// `month` ou `day`.
    pub kind: String,
    /// 1er du mois (`month`) ou jour, `AAAA-MM-JJ`.
    pub rate_date: String,
    /// Le cours, **par unité**, en chaîne décimale.
    pub rate: String,
    /// Absent : création ; présent : remplacement de cette version.
    pub version: Option<i32>,
}

impl_validation_extractor!(
    PutRateBodyExtractor,
    PutRateBody,
    "`currency`, `kind`, `rateDate`, `rate` en chaînes, `version` entier facultatif"
);

/// Corps de `POST /exchange-rates/rates/reset` — champs lus en chaînes.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetRateBody {
    /// Code ISO 4217, lu tel quel.
    pub currency: String,
    /// `month` ou `day`.
    pub kind: String,
    /// 1er du mois (`month`) ou jour, `AAAA-MM-JJ`.
    pub rate_date: String,
    /// Obligatoire si la ligne existe, interdit sinon.
    pub version: Option<i32>,
    /// La valeur publiée (par unité) que l'écran a montrée.
    pub published_rate: Option<String>,
}

impl_validation_extractor!(
    ResetRateBodyExtractor,
    ResetRateBody,
    "`currency`, `kind`, `rateDate` en chaînes, `version` entier et `publishedRate` facultatifs"
);

/// Le détail d'audit d'une écriture d'un cours retenu (AC-5) : `{currency,
/// kind, rateDate, before: {rate, source} | null, after: {rate, source} |
/// null, publishedRate}`, cours en chaînes.
fn details_cours(
    devise: &str,
    kind: RateKind,
    rate_date: NaiveDate,
    change: &RateChange,
    publiee: Option<Decimal>,
) -> serde_json::Value {
    let etat = |r: &Option<CompanyRate>| {
        r.as_ref()
            .map(|r| json!({ "rate": r.rate.to_string(), "source": r.source.as_str() }))
    };
    json!({
        "currency": devise,
        "kind": kind.as_str(),
        "rateDate": rate_date.format("%Y-%m-%d").to_string(),
        "before": etat(&change.before),
        "after": etat(&change.after),
        "publishedRate": publiee.map(|p| p.to_string()),
    })
}

/// Identifiant d'audit d'une écriture d'un cours : la ligne après, ou celle
/// d'avant (suppression).
fn id_ligne(change: &RateChange) -> Result<i64, DbError> {
    change
        .after
        .as_ref()
        .or(change.before.as_ref())
        .map(|r| r.id)
        .ok_or_else(|| DbError::Invariant("écriture d'un cours sans ligne".into()))
}

/// `PUT /api/v1/exchange-rates/rates` — **saisit** un cours retenu par la
/// société (`source = 'manual'`, AC-5), audité `exchange_rate.created` ou
/// `exchange_rate.updated` dans la transaction ; rend la `Cell` à jour. Ne
/// réécrit aucune écriture comptable.
///
/// Rejouée sur interblocage : elle écrit `exchange_rates` (`companies` en
/// partage par la clé étrangère), en cycle possible avec la copie d'une
/// récupération ou d'une route d'exercice (mêmes clés uniques).
pub async fn put_rate(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    PutRateBodyExtractor(body): PutRateBodyExtractor,
) -> Result<Json<Cell>, AppError> {
    exiger_multi_devises(&state, current_user.company_id).await?;
    let kind = lire_genre(&body.kind)?;
    let rate_date = lire_date_de_cours(&body.rate_date, kind)?;
    refuser_chf(&body.currency)?;
    let cours = lire_cours(&body.rate, "rate")?;
    exiger_devise_connue(&body.currency)?;
    juger_cours(&cours, &body.currency)?;

    let user = &current_user;
    let devise = body.currency.as_str();
    let version = body.version;
    let (change, publiee) = kesh_db::retry::retry_on_deadlock("exchange_rates::put_rate", || {
        let pool = state.pool.clone();
        async move {
            let mut tx = pool.begin().await.map_err(map_db_error)?;
            let change = depot::set_manual_rate(
                &mut tx,
                user.company_id,
                user.user_id,
                devise,
                kind,
                rate_date,
                cours,
                version,
            )
            .await?;
            let publiee =
                depot::find_fetched_rate_in_tx(&mut tx, SOURCE_AFC, devise, kind, rate_date)
                    .await?;
            let details = details_cours(
                devise,
                kind,
                rate_date,
                &change,
                publiee.as_ref().map(|p| p.rate_per_unit),
            );
            let id = id_ligne(&change)?;
            let entree = if change.before.is_none() {
                NewAuditLogEntry::from_current_user(
                    user,
                    "exchange_rate.created",
                    "exchange_rate",
                    id,
                    Some(details),
                )
            } else {
                NewAuditLogEntry::from_current_user(
                    user,
                    "exchange_rate.updated",
                    "exchange_rate",
                    id,
                    Some(details),
                )
            };
            audit_log::insert_in_tx(&mut tx, entree).await?;
            tx.commit().await.map_err(map_db_error)?;
            Ok((change, publiee))
        }
    })
    .await?;
    Ok(Json(Cell::de(
        kind,
        rate_date,
        change.after.as_ref(),
        publiee.as_ref(),
    )))
}

/// `POST /api/v1/exchange-rates/rates/reset` — le **↑** (AC-5, U1, U3) :
/// la ligne prend la dernière valeur publiée, `source = 'afc'` (créée si
/// absente) ; sans valeur publiée, un cours saisi (ou d'une autre source) est
/// **supprimé**, une ligne `afc` → 404. `version` obligatoire si la ligne
/// existe (409 sinon), interdite sans ligne (400) ; `publishedRate` périmé
/// (changé, ou disparu depuis) → 409. Audit `exchange_rate.reset` dans la transaction ; rend la `Cell` à
/// jour (`effectiveRate` `null` après une suppression). Portée : la société
/// de l'utilisateur seule.
pub async fn reset_rate(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    ResetRateBodyExtractor(body): ResetRateBodyExtractor,
) -> Result<Json<Cell>, AppError> {
    exiger_multi_devises(&state, current_user.company_id).await?;
    let kind = lire_genre(&body.kind)?;
    let rate_date = lire_date_de_cours(&body.rate_date, kind)?;
    refuser_chf(&body.currency)?;
    let montree = body
        .published_rate
        .as_deref()
        .map(|p| lire_cours(p, "publishedRate"))
        .transpose()?;
    exiger_devise_connue(&body.currency)?;

    let user = &current_user;
    let devise = body.currency.as_str();
    let version = body.version;
    let resultat = kesh_db::retry::retry_on_deadlock("exchange_rates::reset_rate", || {
        let pool = state.pool.clone();
        async move {
            let mut tx = pool.begin().await.map_err(map_db_error)?;
            let change = depot::reset_rate(
                &mut tx,
                user.company_id,
                user.user_id,
                devise,
                kind,
                rate_date,
                version,
                montree,
            )
            .await?;
            let publiee =
                depot::find_fetched_rate_in_tx(&mut tx, SOURCE_AFC, devise, kind, rate_date)
                    .await?;
            let details = details_cours(devise, kind, rate_date, &change, change.published_rate);
            audit_log::insert_in_tx(
                &mut tx,
                NewAuditLogEntry::from_current_user(
                    user,
                    "exchange_rate.reset",
                    "exchange_rate",
                    id_ligne(&change)?,
                    Some(details),
                ),
            )
            .await?;
            tx.commit().await.map_err(map_db_error)?;
            Ok((change, publiee))
        }
    })
    .await;
    let (change, publiee) = match resultat {
        Err(DbError::InvalidInput(code)) if code == "versionForbidden" => {
            return Err(AppError::Validation(t(
                "error-exchange-rate-version-forbidden",
                "Aucun cours n'est encore retenu pour cette case : la version ne doit pas être envoyée.",
            )));
        }
        r => r?,
    };
    Ok(Json(Cell::de(
        kind,
        rate_date,
        change.after.as_ref(),
        publiee.as_ref(),
    )))
}

// ---------------------------------------------------------------------------
// Corrections de l'AFC : notices et acquittement
// ---------------------------------------------------------------------------

/// Une correction à annoncer.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoticeResponse {
    /// Code ISO 4217.
    pub currency: String,
    /// Genre.
    pub kind: RateKind,
    /// 1er du mois ou jour.
    pub rate_date: NaiveDate,
    /// Le cours retenu par la société (`afc`).
    pub effective_rate: Decimal,
    /// La valeur publiée, par unité.
    pub published_rate: Decimal,
    /// L'unité publiée.
    pub published_unit: i32,
    /// La récupération qui a apporté la valeur publiée (UTC).
    pub value_changed_at: DateTime<Utc>,
}

/// Réponse de `GET /exchange-rates/notices`.
#[derive(Debug, Serialize)]
pub struct NoticesResponse {
    /// Triées par date puis devise ; vide sans correction à annoncer.
    pub notices: Vec<NoticeResponse>,
}

/// `GET /api/v1/exchange-rates/notices` — les corrections de l'AFC à annoncer
/// à la société : ses lignes `afc` dont la valeur publiée diffère et n'est pas
/// acquittée à cette valeur. Lecture seule ; Administrateur ou Comptable.
pub async fn list_notices(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<NoticesResponse>, AppError> {
    exiger_multi_devises(&state, current_user.company_id).await?;
    let mut conn = state.pool.acquire().await.map_err(map_db_error)?;
    let notices = depot::list_notices(&mut conn, current_user.company_id)
        .await?
        .into_iter()
        .map(|n| NoticeResponse {
            currency: n.currency,
            kind: n.kind,
            rate_date: n.rate_date,
            effective_rate: n.effective_rate,
            published_rate: n.published_rate,
            published_unit: n.published_unit,
            value_changed_at: n.value_changed_at.and_utc(),
        })
        .collect();
    Ok(Json(NoticesResponse { notices }))
}

/// Une entrée de l'acquittement — champs lus en chaînes.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcknowledgeItem {
    /// Code ISO 4217, lu tel quel.
    pub currency: String,
    /// `month` ou `day`.
    pub kind: String,
    /// 1er du mois ou jour.
    pub rate_date: String,
    /// La valeur publiée (par unité) que l'écran a montrée.
    pub published_rate: String,
}

/// Corps de `POST /exchange-rates/notices/acknowledge`.
#[derive(Debug, Clone, Deserialize)]
pub struct AcknowledgeBody {
    /// 1 à 500 entrées.
    pub notices: Vec<AcknowledgeItem>,
}

impl_validation_extractor!(
    AcknowledgeBodyExtractor,
    AcknowledgeBody,
    "`notices` : [{ currency, kind, rateDate, publishedRate }] en chaînes"
);

/// Réponse de `POST /exchange-rates/notices/acknowledge`.
#[derive(Debug, Serialize)]
pub struct AcknowledgeResponse {
    /// Nombre de corrections acquittées.
    pub acknowledged: usize,
}

/// `POST /api/v1/exchange-rates/notices/acknowledge` — acquitte des
/// corrections pour la société, à la valeur publiée que l'écran a montrée
/// (une valeur changée entre-temps n'est pas acquittée). Ne change ni le
/// cours, ni la source, ni la version. Audit
/// `exchange_rates.corrections_acknowledged` (entité `company`) des seules
/// acquittées, aucun si rien ne l'est. 1 à 500 entrées, sinon 400.
pub async fn acknowledge_notices(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    AcknowledgeBodyExtractor(body): AcknowledgeBodyExtractor,
) -> Result<Json<AcknowledgeResponse>, AppError> {
    exiger_multi_devises(&state, current_user.company_id).await?;
    if body.notices.is_empty() || body.notices.len() > MAX_ACKNOWLEDGE_NOTICES {
        return Err(AppError::Validation(format!(
            "`notices` : de 1 à {MAX_ACKNOWLEDGE_NOTICES} entrées"
        )));
    }
    let mut items = Vec::with_capacity(body.notices.len());
    for n in &body.notices {
        let kind = lire_genre(&n.kind)?;
        let rate_date = lire_date_de_cours(&n.rate_date, kind)?;
        refuser_chf(&n.currency)?;
        let published_rate = lire_cours(&n.published_rate, "publishedRate")?;
        items.push(NoticeAck {
            currency: n.currency.clone(),
            kind,
            rate_date,
            published_rate,
        });
    }
    for n in &items {
        exiger_devise_connue(&n.currency)?;
    }

    let user = &current_user;
    let items = &items;
    let acquittees =
        kesh_db::retry::retry_on_deadlock("exchange_rates::acknowledge_notices", || {
            let pool = state.pool.clone();
            async move {
                let mut tx = pool.begin().await.map_err(map_db_error)?;
                let acquittees = depot::acknowledge_corrections(
                    &mut tx,
                    user.company_id,
                    items,
                    recuperation::maintenant_ms(),
                )
                .await?;
                if !acquittees.is_empty() {
                    let notices: Vec<_> = acquittees
                        .iter()
                        .map(|a| {
                            json!({
                                "currency": a.currency,
                                "kind": a.kind.as_str(),
                                "rateDate": a.rate_date.format("%Y-%m-%d").to_string(),
                                "publishedRate": a.published_rate.to_string(),
                            })
                        })
                        .collect();
                    audit_log::insert_in_tx(
                        &mut tx,
                        NewAuditLogEntry::from_current_user(
                            user,
                            "exchange_rates.corrections_acknowledged",
                            "company",
                            user.company_id,
                            Some(json!({ "notices": notices })),
                        ),
                    )
                    .await?;
                }
                tx.commit().await.map_err(map_db_error)?;
                Ok(acquittees.len())
            }
        })
        .await?;
    Ok(Json(AcknowledgeResponse {
        acknowledged: acquittees,
    }))
}

// ---------------------------------------------------------------------------
// GET /fiscal-years/{id}/exchange-rates — le tableau annuel (R6)
// ---------------------------------------------------------------------------

/// Paramètres du tableau annuel.
#[derive(Debug, Deserialize)]
pub struct FiscalYearRatesQuery {
    /// `true` : toutes les devises qui ont une ligne ou une valeur publiée
    /// pour une unité de l'exercice.
    pub all: Option<String>,
}

/// Une ligne du tableau annuel : une devise.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FiscalYearRatesRow {
    /// Code ISO 4217.
    pub currency: String,
    /// Au moins un compte de la société, archivé compris, porte la devise.
    pub has_account: bool,
    /// Une case par mois de l'exercice, dans l'ordre de `months`.
    pub months: Vec<Cell>,
    /// La case d'ouverture.
    pub opening: Cell,
    /// La case de clôture.
    pub closing: Cell,
}

/// Réponse du tableau annuel.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FiscalYearRatesResponse {
    /// L'exercice.
    pub fiscal_year_id: i64,
    /// Premier jour.
    pub start_date: NaiveDate,
    /// Dernier jour.
    pub end_date: NaiveDate,
    /// Genre du cours d'ouverture.
    pub opening_rate_kind: RateKind,
    /// Genre du cours de clôture.
    pub closing_rate_kind: RateKind,
    /// Date du cours d'ouverture (fonction pure de `kesh-core`).
    pub opening_date: NaiveDate,
    /// Date du cours de clôture.
    pub closing_date: NaiveDate,
    /// Les mois de l'exercice, du mois de `startDate` au mois de `endDate`,
    /// en 1ers du mois, quelle que soit la durée.
    pub months: Vec<NaiveDate>,
    /// Une ligne par devise, triées.
    pub rows: Vec<FiscalYearRatesRow>,
}

/// `GET /api/v1/fiscal-years/{id}/exchange-rates[?all=true]` — le tableau
/// annuel d'un exercice (AC-5, R6), en lecture seule. Exercice d'une autre
/// société → 404 ; exercice mono-devise → `409 MULTI_CURRENCY_DISABLED`
/// (`details {fiscalYearId}`). Les cases se lisent en **une** connexion, par
/// une lecture groupée par genre et plage de dates
/// (`exchange_rates::list_cells_in_tx`), jamais une requête par case.
///
/// Lignes : les devises des comptes actifs, celles des comptes archivés qui
/// ont une ligne dans l'exercice, et celles qui ont une ligne `manual` ou
/// d'une autre source que `afc` dans l'exercice ; `all=true` ajoute toute
/// devise qui a une ligne ou une valeur publiée pour une unité de l'exercice.
pub async fn fiscal_year_rates(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Query(q): Query<FiscalYearRatesQuery>,
) -> Result<Json<FiscalYearRatesResponse>, AppError> {
    let tout = q.all.as_deref() == Some("true");
    let fy = kesh_db::repositories::fiscal_years::find_by_id_in_company(
        &state.pool,
        current_user.company_id,
        id,
    )
    .await?
    .ok_or(AppError::Database(DbError::NotFound))?;
    if !fy.multi_currency {
        return Err(AppError::Database(DbError::MultiCurrencyDisabled {
            fiscal_year_id: Some(id),
        }));
    }
    let ex = fy.exercice_devises();
    let months = exercise_months(fy.start_date, fy.end_date);
    let (ouverture, cloture) = (ex.opening_date(), ex.closing_date());
    let mut conn = state.pool.acquire().await.map_err(map_db_error)?;

    // Les unités de l'exercice, et les dates à lire par genre.
    let mut unites: BTreeSet<RateUnit> = months.iter().map(|m| RateUnit::month_of(*m)).collect();
    unites.insert(RateUnit {
        date: ouverture,
        kind: ex.opening_kind,
    });
    unites.insert(RateUnit {
        date: cloture,
        kind: ex.closing_kind,
    });
    let mut cases: BTreeMap<(RateKind, NaiveDate, String), RateCell> = BTreeMap::new();
    for kind in [RateKind::Month, RateKind::Day] {
        let dates: Vec<NaiveDate> = unites
            .iter()
            .filter(|u| u.kind == kind)
            .map(|u| u.date)
            .collect();
        let (Some(de), Some(a)) = (dates.iter().min(), dates.iter().max()) else {
            continue;
        };
        for c in depot::list_cells_in_tx(&mut conn, current_user.company_id, kind, *de, *a).await? {
            if unites.contains(&RateUnit {
                date: c.rate_date,
                kind,
            }) {
                cases.insert((kind, c.rate_date, c.currency.clone()), c);
            }
        }
    }
    let comptes: BTreeMap<String, bool> =
        depot::list_account_currency_states(&mut conn, current_user.company_id)
            .await?
            .into_iter()
            .collect();

    let mut devises: BTreeSet<String> = comptes
        .iter()
        .filter(|(_, actif)| **actif)
        .map(|(d, _)| d.clone())
        .collect();
    for c in cases.values() {
        let ligne = c.company.as_ref();
        let archivee = comptes.get(&c.currency) == Some(&false) && ligne.is_some();
        let non_afc = ligne.is_some_and(|r| !r.source.is_afc());
        if archivee || non_afc || tout {
            devises.insert(c.currency.clone());
        }
    }

    let case = |kind: RateKind, date: NaiveDate, devise: &str| {
        Cell::depuis(kind, date, cases.get(&(kind, date, devise.to_string())))
    };
    let rows = devises
        .into_iter()
        .map(|devise| FiscalYearRatesRow {
            has_account: comptes.contains_key(&devise),
            months: months
                .iter()
                .map(|m| case(RateKind::Month, *m, &devise))
                .collect(),
            opening: case(ex.opening_kind, ouverture, &devise),
            closing: case(ex.closing_kind, cloture, &devise),
            currency: devise,
        })
        .collect();
    Ok(Json(FiscalYearRatesResponse {
        fiscal_year_id: fy.id,
        start_date: fy.start_date,
        end_date: fy.end_date,
        opening_rate_kind: fy.opening_rate_kind,
        closing_rate_kind: fy.closing_rate_kind,
        opening_date: ouverture,
        closing_date: cloture,
        months,
        rows,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, j: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, j).unwrap()
    }

    fn corps(year: Option<i32>, month: Option<u32>, date: Option<&str>) -> FetchRatesBody {
        FetchRatesBody {
            year,
            month,
            date: date.map(str::to_string),
        }
    }

    /// Bornes de l'AC-5 : mois 2000-01 au mois suivant ; jour 2000-01-01 à
    /// hier ; une seule des deux formes.
    #[test]
    fn bornes_de_l_unite_demandee() {
        let auj = d(2026, 11, 16);
        assert_eq!(
            unite_demandee(&corps(Some(2026), Some(12), None), auj),
            Ok(RateUnit::month_of(d(2026, 12, 1)))
        );
        assert_eq!(
            unite_demandee(&corps(Some(2000), Some(1), None), auj),
            Ok(RateUnit::month_of(d(2000, 1, 1)))
        );
        assert!(unite_demandee(&corps(Some(2027), Some(1), None), auj).is_err());
        assert!(unite_demandee(&corps(Some(1999), Some(12), None), auj).is_err());
        assert!(unite_demandee(&corps(Some(2026), Some(13), None), auj).is_err());
        assert!(unite_demandee(&corps(Some(2026), Some(0), None), auj).is_err());
        assert_eq!(
            unite_demandee(&corps(None, None, Some("2026-11-15")), auj),
            Ok(RateUnit::day(d(2026, 11, 15)))
        );
        assert_eq!(
            unite_demandee(&corps(None, None, Some("2000-01-01")), auj),
            Ok(RateUnit::day(d(2000, 1, 1)))
        );
        assert!(unite_demandee(&corps(None, None, Some("2026-11-16")), auj).is_err());
        assert!(unite_demandee(&corps(None, None, Some("2026-11-17")), auj).is_err());
        assert!(unite_demandee(&corps(None, None, Some("1999-12-31")), auj).is_err());
        assert!(unite_demandee(&corps(None, None, Some("2026-2-01")), auj).is_err());
        assert!(unite_demandee(&corps(None, None, Some("2026-02-30")), auj).is_err());
        assert!(unite_demandee(&corps(Some(2026), Some(11), Some("2026-11-01")), auj).is_err());
        assert!(unite_demandee(&corps(Some(2026), None, None), auj).is_err());
        assert!(unite_demandee(&corps(None, None, None), auj).is_err());
    }
}
