//! Routes des cours de change (Story 37-2, AC-5).
//!
//! - `POST /api/v1/exchange-rates/fetch` — récupérer les cours d'un mois
//!   (`{year, month}`) ou d'un jour (`{date}`), Administrateur ou Comptable ;
//! - `GET /api/v1/exchange-rates/status` — l'état de la dernière
//!   récupération, tout rôle authentifié.
//!
//! Toutes exigent une société **multi-devises** (`409
//! MULTI_CURRENCY_DISABLED`), dans l'ordre des refus de l'AC-5 :
//! authentification et rôle (401, 403 — couches du routeur) → société
//! mono-devise (409) → paramètres illisibles ou hors bornes (400). Seul écart :
//! un corps JSON illisible est refusé par l'extracteur (400) avant le handler.

use axum::Json;
use axum::extract::{Extension, State};
use chrono::{DateTime, NaiveDate, Utc};
use kesh_core::exchange_rates::{MIN_RATE_DATE, RateKind, RateUnit, next_month};
use kesh_db::errors::{DbError, map_db_error};
use kesh_db::repositories::exchange_rates::{
    self as depot, FetchOutcomeStatus, FetchStatus, FetchTrigger,
};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::errors::AppError;
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
async fn executer(
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
    let tache = tokio::spawn(async move { executer(&etat, current_user, unit, arrivee).await });
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
