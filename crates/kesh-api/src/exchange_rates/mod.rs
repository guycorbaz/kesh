//! Cours de change — source, récupération, tâche automatique (Story 37-2,
//! AC-3, AC-4).
//!
//! - [`source`] : le trait [`RateSource`], le client HTTP de production
//!   [`HttpRateSource`] et la source factice [`MockRateSource`] ;
//! - [`recuperation`] : la récupération d'**une** unité (genre et date),
//!   partagée par `POST /exchange-rates/fetch`, la tâche et la connexion ;
//! - ici : la tâche automatique ([`spawn`], lancée par `main.rs` seulement),
//!   un tour ([`un_tour`]), le calcul du prochain réveil ([`prochain_reveil`],
//!   fonction pure) et le déclenchement à la connexion
//!   ([`declencher_a_la_connexion`]).
//!
//! # Calendrier (AC-4, S1)
//!
//! La tâche ne tourne pas en boucle fixe. Elle se réveille 60 s après le
//! démarrage, puis : chaque jour à 00:00 (Europe/Zurich) tant que le mois
//! courant ou le mois suivant manque pour un exercice multi-devises ouvert qui
//! le couvre ; le mois suivant `T` obtenu, au plus tôt de « date d'obtention +
//! 24 jours » et du 25 de `T`, jamais avant le 1er de `T` ; le 1er du mois
//! suivant si le mois suivant n'est couvert par aucun exercice. Un tour ne se
//! lance que si une unité utile manque, et traite au plus [`MAX_UNITES_PAR_TOUR`]
//! unités, dans l'ordre chronologique, plus une **revérification** du mois
//! courant (hors plafond) s'il est présent et couvert.
//!
//! # Angles morts assumés
//!
//! - La tâche ne tourne que si Kesh tourne ; un réveil manqué (machine en
//!   veille) se rattrape au démarrage suivant ou à la prochaine connexion.
//! - Un jour d'ouverture ou de clôture manquant ne réveille pas la tâche : il
//!   se récupère au premier tour qui suit (limite écrite au manuel).
//! - Un tour dont toutes les unités sont rendues par la mémoire des 15 minutes,
//!   ou jugées présentes sous le verrou, n'écrit pas `_kesh_exchange_rate_fetch`
//!   (aucun appel sortant : rien de neuf à dire).

pub mod recuperation;
pub mod source;

use std::time::Instant;

use chrono::{DateTime, Datelike, NaiveDate, TimeDelta, Utc};
use kesh_core::exchange_rates::{RateUnit, next_month, unites_utiles};
use kesh_core::horloge::{debut_du_jour_suisse, jour_suisse};
use kesh_db::errors::{DbError, map_db_error};
use kesh_db::repositories::exchange_rates::{
    self as depot, FetchOutcomeStatus, FetchStatus, FetchTrigger,
};

use crate::AppState;

pub use recuperation::{
    Demande, Echec, Issue, MEMO_TTL, Manuelle, MemoEntry, RateFetchMemo, Recuperee,
};
pub use source::{
    HttpRateSource, MOCK_DAILY_RATES, MOCK_MONTHLY_RATES, MockBehavior, MockRateSource, MockReply,
    RateFuture, RateSource, SourceError, UnavailableRateSource, production_rate_source,
};

/// Délai avant le premier tour après le démarrage (AC-4 (a)).
pub const DELAI_DEMARRAGE: std::time::Duration = std::time::Duration::from_secs(60);

/// Plafond d'unités manquantes traitées par tour (AC-4) ; le reste au tour
/// suivant. La revérification du mois courant s'y ajoute, hors plafond.
pub const MAX_UNITES_PAR_TOUR: usize = 36;

/// Pause après l'obtention du mois suivant (AC-4 (c)).
pub const PAUSE_JOURS: i64 = 24;

/// Jour du mois au-delà duquel la pause ne va jamais (AC-4 (c)).
pub const JOUR_LIMITE_PAUSE: u32 = 25;

/// Intervalle minimal entre deux tours lancés à la connexion (AC-4).
pub const INTERVALLE_CONNEXION: std::time::Duration = std::time::Duration::from_secs(15 * 60);

// ---------------------------------------------------------------------------
// Prochain réveil (fonction pure)
// ---------------------------------------------------------------------------

/// Ce que [`prochain_reveil`] doit savoir de l'installation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EtatReveil {
    /// Le mois courant ou le mois suivant manque pour un exercice
    /// multi-devises ouvert qui le couvre.
    pub mois_utile_manquant: bool,
    /// Le mois suivant est couvert par un exercice multi-devises ouvert.
    pub mois_suivant_couvert: bool,
    /// Date d'obtention (jour de Zurich du plus petit `first_fetched_at`) du
    /// mois suivant, s'il est présent.
    pub obtention_mois_suivant: Option<NaiveDate>,
}

/// Le prochain réveil de la tâche (AC-4, S1), à 00:00 Europe/Zurich, rendu en
/// UTC :
/// - mois utile manquant → le lendemain ;
/// - mois suivant non couvert → le 1er du mois suivant ;
/// - mois suivant `T` obtenu le `o` → `max(1er de T, min(o + 24 j, 25 de T))`.
///
/// Un jour d'ouverture ou de clôture manquant n'y entre pas. Le réveil est
/// toujours postérieur à `now`.
pub fn prochain_reveil(now: DateTime<Utc>, etat: &EtatReveil) -> DateTime<Utc> {
    let aujourdhui = jour_suisse(now);
    let lendemain = aujourdhui
        .succ_opt()
        .expect("lendemain dans la plage de chrono");
    let t = next_month(aujourdhui);
    let jour = if etat.mois_utile_manquant {
        lendemain
    } else if !etat.mois_suivant_couvert {
        t
    } else {
        match etat.obtention_mois_suivant {
            Some(o) => {
                let plus_24 = o + TimeDelta::days(PAUSE_JOURS);
                let limite = t
                    .with_day(JOUR_LIMITE_PAUSE)
                    .expect("le 25 existe dans tout mois");
                plus_24.min(limite).max(t)
            }
            // Incohérent (mois suivant présent sans date d'obtention) : le
            // régime quotidien, le plus prudent.
            None => lendemain,
        }
    };
    debut_du_jour_suisse(jour)
}

/// Lit l'état du calendrier ([`EtatReveil`]) au jour `aujourdhui` (Zurich).
pub async fn etat_reveil(state: &AppState, aujourdhui: NaiveDate) -> Result<EtatReveil, DbError> {
    let mut conn = state.pool.acquire().await.map_err(map_db_error)?;
    let exercices = depot::list_open_multi_currency_fiscal_years(&mut conn, None).await?;
    let ex: Vec<_> = exercices.iter().map(|e| e.exercice).collect();
    let utiles = unites_utiles(&ex, aujourdhui);
    let courant = RateUnit::month_of(aujourdhui);
    let suivant = RateUnit::month_of(next_month(aujourdhui));
    let mois_couverts: Vec<RateUnit> = [courant, suivant]
        .into_iter()
        .filter(|u| utiles.contains(u))
        .collect();
    let manquants = depot::missing_units(&mut conn, &mois_couverts).await?;
    let obtention = depot::month_first_fetched_at(&mut conn, suivant.date)
        .await?
        .map(|t| jour_suisse(t.and_utc()));
    Ok(EtatReveil {
        mois_utile_manquant: !manquants.is_empty(),
        mois_suivant_couvert: utiles.contains(&suivant),
        obtention_mois_suivant: obtention,
    })
}

// ---------------------------------------------------------------------------
// Un tour
// ---------------------------------------------------------------------------

/// Ce qui lance un tour automatique (journal du serveur seulement : tous
/// écrivent `triggered_by = 'auto'`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Declencheur {
    /// La tâche (démarrage, réveil quotidien, fin de pause).
    Tache,
    /// Une connexion.
    Connexion,
}

/// Le bilan d'un tour, pour le journal et les tests.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BilanTour {
    /// Les unités demandées (manquantes ; au plus [`MAX_UNITES_PAR_TOUR`], mois courant et mois suivant manquants compris).
    pub demandees: Vec<RateUnit>,
    /// La revérification du mois courant, si elle a été demandée.
    pub reverifiee: Option<RateUnit>,
    /// Le statut écrit (`None` : aucun appel sortant, rien d'écrit).
    pub statut: Option<FetchOutcomeStatus>,
}

/// Les unités demandées par un tour : au plus [`MAX_UNITES_PAR_TOUR`] des
/// `manquantes`, dans l'ordre chronologique — mais les unités `prioritaires`
/// (mois courant et mois suivant) qui manquent y sont **toujours** : sans
/// cela, 36 unités plus anciennes en échec durable (un jour d'ouverture que la
/// source ne sert pas, par exemple) occupaient chaque tour, et le mois courant
/// n'était jamais demandé (revue de code P1, 37-2-CR-P1-4). Le plafond et
/// l'ordre chronologique de la liste rendue sont gardés.
pub(crate) fn choisir_demandees(
    manquantes: &[RateUnit],
    prioritaires: &[RateUnit],
) -> Vec<RateUnit> {
    let avant: Vec<RateUnit> = manquantes
        .iter()
        .copied()
        .filter(|u| prioritaires.contains(u))
        .collect();
    let place = MAX_UNITES_PAR_TOUR.saturating_sub(avant.len());
    let mut demandees: Vec<RateUnit> = manquantes
        .iter()
        .copied()
        .filter(|u| !prioritaires.contains(u))
        .take(place)
        .chain(avant)
        .collect();
    demandees.sort_by_key(|u| (u.date, u.kind));
    demandees
}

/// Un tour de récupération automatique (AC-4) : unités utiles manquantes des
/// exercices multi-devises ouverts (toutes sociétés), choisies par
/// [`choisir_demandees`] (au plus [`MAX_UNITES_PAR_TOUR`], dans l'ordre chronologique), chacune jugée
/// manquante **sous le verrou** ; puis revérification du mois courant s'il
/// était présent et couvert. Rien ne manque → aucun appel, revérification
/// comprise. Écrit le pire statut (`triggered_by = 'auto'`), non audité.
///
/// Appelée par la tâche et par la connexion ; exposée pour les tests
/// d'intégration (`#[doc(hidden)]`).
#[doc(hidden)]
pub async fn un_tour(state: &AppState, declencheur: Declencheur) -> Result<BilanTour, DbError> {
    let aujourdhui = state.horloge.aujourd_hui();
    let (utiles, manquantes) = {
        let mut conn = state.pool.acquire().await.map_err(map_db_error)?;
        let exercices = depot::list_open_multi_currency_fiscal_years(&mut conn, None).await?;
        if exercices.is_empty() {
            return Ok(BilanTour::default());
        }
        let ex: Vec<_> = exercices.iter().map(|e| e.exercice).collect();
        let utiles = unites_utiles(&ex, aujourdhui);
        let manquantes = depot::missing_units(&mut conn, &utiles).await?;
        (utiles, manquantes)
    };
    if manquantes.is_empty() {
        return Ok(BilanTour::default());
    }
    let suivant = RateUnit::month_of(next_month(aujourdhui));
    let courant = RateUnit::month_of(aujourdhui);
    let demandees = choisir_demandees(&manquantes, &[courant, suivant]);
    let reverifiee =
        (utiles.contains(&courant) && !manquantes.contains(&courant)).then_some(courant);

    // (statut, unité) de chaque appel fait : le pire l'emporte, la plus
    // récente à égalité.
    let mut pire: Option<(FetchOutcomeStatus, RateUnit, Option<String>)> = None;
    let mut noter = |statut: FetchOutcomeStatus, unit: RateUnit, erreur: Option<String>| {
        let remplace = match &pire {
            None => true,
            Some((s, u, _)) => (statut, unit.date) > (*s, u.date),
        };
        if remplace {
            pire = Some((statut, unit, erreur));
        }
    };
    let demandes = demandees
        .iter()
        .map(|u| (*u, true))
        .chain(reverifiee.map(|u| (u, false)));
    for (unit, seulement_si_manquante) in demandes {
        let issue = recuperation::recuperer_unite(
            state,
            Demande {
                unit,
                seulement_si_manquante,
                manuelle: None,
            },
        )
        .await?;
        match issue {
            Issue::Recuperee(r) => noter(FetchOutcomeStatus::Ok, unit, r.error_code()),
            Issue::Echouee(echec) => {
                if echec == Echec::NotPublished && unit == suivant {
                    tracing::info!(rate_date = %unit.date, "cours du mois suivant pas encore publiés");
                } else {
                    tracing::warn!(
                        kind = %unit.kind,
                        rate_date = %unit.date,
                        erreur = ?echec,
                        "récupération automatique des cours : échec"
                    );
                }
                noter(echec.status(), unit, echec.error_code());
            }
            Issue::Memorisee(_) | Issue::DejaPresente | Issue::Raccourci(_) => {}
        }
    }
    let statut = match pire {
        Some((status, unit, error)) => {
            let mut conn = state.pool.acquire().await.map_err(map_db_error)?;
            depot::record_fetch_status(
                &mut conn,
                &FetchStatus {
                    trigger: FetchTrigger::Auto,
                    last_run_at: recuperation::maintenant_ms(),
                    status,
                    error,
                    kind: unit.kind,
                    rate_date: unit.date,
                },
            )
            .await?;
            Some(status)
        }
        None => None,
    };
    tracing::debug!(
        ?declencheur,
        demandees = demandees.len(),
        ?statut,
        "tour des cours terminé"
    );
    Ok(BilanTour {
        demandees,
        reverifiee,
        statut,
    })
}

// ---------------------------------------------------------------------------
// La tâche
// ---------------------------------------------------------------------------

/// Lance la tâche automatique (AC-4) : premier tour [`DELAI_DEMARRAGE`] après
/// le démarrage, puis réveil selon [`prochain_reveil`]. Chaque tour s'exécute
/// dans son propre `tokio::spawn` : une panique y est journalisée et n'arrête
/// pas la tâche. Appelée par `main.rs` **seulement**, hors `KESH_TEST_MODE`
/// (garde lexicale `tests::lancement_par_main_seulement`).
pub fn spawn(state: AppState) {
    tokio::spawn(async move {
        tokio::time::sleep(DELAI_DEMARRAGE).await;
        loop {
            let etat = state.clone();
            match tokio::spawn(async move { un_tour(&etat, Declencheur::Tache).await }).await {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => tracing::error!("récupération automatique des cours : base : {e}"),
                Err(e) => tracing::error!("tour de récupération des cours interrompu : {e}"),
            }
            let now = state.horloge.maintenant();
            let reveil = match etat_reveil(&state, jour_suisse(now)).await {
                Ok(e) => prochain_reveil(now, &e),
                Err(e) => {
                    tracing::error!("récupération automatique des cours : calendrier : {e}");
                    debut_du_jour_suisse(jour_suisse(now) + TimeDelta::days(1))
                }
            };
            let attente = (reveil - state.horloge.maintenant())
                .to_std()
                .unwrap_or(std::time::Duration::ZERO);
            tracing::debug!(%reveil, "prochaine récupération automatique des cours");
            tokio::time::sleep(attente).await;
        }
    });
}

// ---------------------------------------------------------------------------
// Déclenchement à la connexion
// ---------------------------------------------------------------------------

/// Après une connexion réussie (AC-4) : une tâche **détachée**, jamais
/// attendue, juge si une unité utile manque pour un exercice multi-devises
/// ouvert de la société de l'utilisateur — le seul mois suivant ne suffit pas
/// — et lance alors un tour, au plus une fois par [`INTERVALLE_CONNEXION`]
/// pour l'installation. Rien en `KESH_TEST_MODE`. Rend la poignée de la tâche
/// (les tests l'attendent ; la connexion l'ignore).
pub fn declencher_a_la_connexion(
    state: &AppState,
    company_id: i64,
) -> Option<tokio::task::JoinHandle<()>> {
    if state.config.test_mode {
        return None;
    }
    let state = state.clone();
    Some(tokio::spawn(async move {
        match unite_manquante_pour_la_connexion(&state, company_id).await {
            Ok(false) => {}
            Ok(true) => {
                if !reserver_le_tour_de_connexion(&state) {
                    return;
                }
                if let Err(e) = un_tour(&state, Declencheur::Connexion).await {
                    tracing::warn!("récupération des cours à la connexion : base : {e}");
                }
            }
            Err(e) => tracing::warn!("récupération des cours à la connexion : base : {e}"),
        }
    }))
}

/// Une unité utile manque-t-elle pour la société, hors le seul mois suivant ?
async fn unite_manquante_pour_la_connexion(
    state: &AppState,
    company_id: i64,
) -> Result<bool, DbError> {
    let aujourdhui = state.horloge.aujourd_hui();
    let mut conn = state.pool.acquire().await.map_err(map_db_error)?;
    let exercices =
        depot::list_open_multi_currency_fiscal_years(&mut conn, Some(company_id)).await?;
    if exercices.is_empty() {
        return Ok(false);
    }
    let ex: Vec<_> = exercices.iter().map(|e| e.exercice).collect();
    let suivant = RateUnit::month_of(next_month(aujourdhui));
    let utiles: Vec<RateUnit> = unites_utiles(&ex, aujourdhui)
        .into_iter()
        .filter(|u| *u != suivant)
        .collect();
    Ok(!depot::missing_units(&mut conn, &utiles).await?.is_empty())
}

/// Réserve le tour de connexion : vrai si aucun n'a été lancé depuis
/// [`INTERVALLE_CONNEXION`] (et note l'instant).
fn reserver_le_tour_de_connexion(state: &AppState) -> bool {
    let mut dernier = state
        .rate_login_trigger
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if dernier.is_some_and(|t| t.elapsed() < INTERVALLE_CONNEXION) {
        return false;
    }
    *dernier = Some(Instant::now());
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::code;

    fn d(y: i32, m: u32, j: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, j).unwrap()
    }

    /// Un instant à 10:00 UTC du jour donné.
    fn a(y: i32, m: u32, j: u32) -> DateTime<Utc> {
        d(y, m, j).and_hms_opt(10, 0, 0).unwrap().and_utc()
    }

    fn present(o: NaiveDate) -> EtatReveil {
        EtatReveil {
            mois_utile_manquant: false,
            mois_suivant_couvert: true,
            obtention_mois_suivant: Some(o),
        }
    }

    const MANQUANT: EtatReveil = EtatReveil {
        mois_utile_manquant: true,
        mois_suivant_couvert: true,
        obtention_mois_suivant: None,
    };

    /// Mois utile manquant → lendemain 00:00 Zurich, fin de mois et d'année
    /// comprises.
    #[test]
    fn mois_manquant_reveil_le_lendemain() {
        assert_eq!(
            prochain_reveil(a(2026, 11, 14), &MANQUANT),
            debut_du_jour_suisse(d(2026, 11, 15))
        );
        assert_eq!(
            prochain_reveil(a(2026, 11, 30), &MANQUANT),
            debut_du_jour_suisse(d(2026, 12, 1))
        );
        assert_eq!(
            prochain_reveil(a(2026, 12, 31), &MANQUANT),
            debut_du_jour_suisse(d(2027, 1, 1))
        );
    }

    /// Les exemples de l'AC-4 : décembre obtenu le 14.11, le 25.11, le 02.11.
    #[test]
    fn pause_apres_le_mois_suivant() {
        assert_eq!(
            prochain_reveil(a(2026, 11, 14), &present(d(2026, 11, 14))),
            debut_du_jour_suisse(d(2026, 12, 8))
        );
        assert_eq!(
            prochain_reveil(a(2026, 11, 25), &present(d(2026, 11, 25))),
            debut_du_jour_suisse(d(2026, 12, 19))
        );
        assert_eq!(
            prochain_reveil(a(2026, 11, 2), &present(d(2026, 11, 2))),
            debut_du_jour_suisse(d(2026, 12, 1)),
            "jamais avant le 1er du mois obtenu"
        );
    }

    /// Restauration en décembre : décembre obtenu le 03.12 (mois courant),
    /// janvier manquant → régime quotidien.
    #[test]
    fn mois_courant_present_mois_suivant_manquant() {
        let etat = EtatReveil {
            mois_utile_manquant: true,
            mois_suivant_couvert: true,
            obtention_mois_suivant: None,
        };
        assert_eq!(
            prochain_reveil(a(2026, 12, 3), &etat),
            debut_du_jour_suisse(d(2026, 12, 4))
        );
    }

    /// Mois suivant non couvert (et rien ne manque) → 1er du mois suivant.
    #[test]
    fn mois_suivant_non_couvert() {
        let etat = EtatReveil {
            mois_utile_manquant: false,
            mois_suivant_couvert: false,
            obtention_mois_suivant: None,
        };
        assert_eq!(
            prochain_reveil(a(2026, 12, 15), &etat),
            debut_du_jour_suisse(d(2027, 1, 1))
        );
    }

    /// Changements d'heure : le réveil est minuit de Zurich, en heure d'été
    /// comme en heure d'hiver.
    #[test]
    fn changements_d_heure() {
        let r = prochain_reveil(a(2026, 3, 28), &MANQUANT);
        assert_eq!(r, "2026-03-28T23:00:00Z".parse::<DateTime<Utc>>().unwrap());
        let r = prochain_reveil(a(2026, 3, 29), &MANQUANT);
        assert_eq!(r, "2026-03-29T22:00:00Z".parse::<DateTime<Utc>>().unwrap());
        let r = prochain_reveil(a(2026, 10, 24), &MANQUANT);
        assert_eq!(r, "2026-10-24T22:00:00Z".parse::<DateTime<Utc>>().unwrap());
        let r = prochain_reveil(a(2026, 10, 25), &MANQUANT);
        assert_eq!(r, "2026-10-25T23:00:00Z".parse::<DateTime<Utc>>().unwrap());
        // Le soir, à Zurich déjà le lendemain : le réveil est le surlendemain UTC.
        let soir = "2026-07-01T22:30:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(
            prochain_reveil(soir, &MANQUANT),
            debut_du_jour_suisse(d(2026, 7, 3))
        );
    }

    /// Propriété « réveil ≤ 25 de T, ≥ 1er de T, > now » pour toutes les dates
    /// d'obtention possibles d'une année (le mois suivant s'obtient au plus
    /// tard la veille de son 1er).
    #[test]
    fn propriete_reveil_au_plus_tard_le_25() {
        let mut o = d(2026, 1, 1);
        while o <= d(2026, 12, 31) {
            let now = o.and_hms_opt(12, 0, 0).unwrap().and_utc();
            let t = next_month(o);
            let r = prochain_reveil(now, &present(o));
            assert!(r <= debut_du_jour_suisse(t.with_day(25).unwrap()), "{o}");
            assert!(r >= debut_du_jour_suisse(t), "{o}");
            assert!(r > now, "{o}");
            o = o.succ_opt().unwrap();
        }
    }

    /// AC-4 — `exchange_rates::spawn(` apparaît exactement une fois dans
    /// `main.rs` et aucune dans `lib.rs` (aide `code()` partagée avec la
    /// garde de la sauvegarde automatique).
    #[test]
    fn lancement_par_main_seulement() {
        let main = code(include_str!("../main.rs"));
        let lib = code(include_str!("../lib.rs"));
        assert_eq!(
            main.matches("exchange_rates::spawn(").count(),
            1,
            "main.rs doit lancer la récupération automatique, une fois"
        );
        assert_eq!(
            lib.matches("exchange_rates::spawn(").count(),
            0,
            "lib.rs ne doit pas lancer la tâche (elle tournerait dans les tests)"
        );
        // Le lancement est gardé par le mode test.
        let avant = &main[..main.find("exchange_rates::spawn(").unwrap()];
        assert!(
            avant
                .rfind("if !")
                .is_some_and(|i| avant[i..].contains("test_mode")),
            "main.rs ne lance la tâche que hors KESH_TEST_MODE"
        );
    }
}
