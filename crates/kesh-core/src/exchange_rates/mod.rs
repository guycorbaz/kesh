//! Cours de change (Story 37-2, Epic 37 multi-devises) — logique pure.
//!
//! - [`RateKind`] : le genre d'un cours (journalier ou mensuel), type unique du
//!   dépôt — `kesh-db` le réexporte (`entities::RateKind`) et le lit en base
//!   par sa forme texte ([`TryFrom<String>`]).
//! - [`check_rate_bounds`] : les bornes d'**un** cours (> 0, au plus
//!   10 décimales, au plus 10⁶), seule fonction des bornes, partagée par
//!   l'analyse des cours publiés, la saisie d'un cours, 37-3a et 37-5.
//! - [`opening_rate_date`], [`closing_rate_date`] : la date du cours
//!   d'ouverture et de clôture d'un exercice selon ses genres — une seule
//!   fonction pour le tableau annuel et pour [`unites_utiles`].
//! - [`unites_utiles`], [`unites_utiles_exercice`] : les unités (genre, date)
//!   à récupérer pour des exercices multi-devises ouverts, et celles à copier
//!   pour un exercice.
//! - [`parse`] : l'analyse du XML de l'AFC / OFDF (cours mensuels et
//!   journaliers), tenu pour hostile.
//!
//! **Pourquoi `RateKind` vit ici** : les fonctions pures de ce module en ont
//! besoin, et `kesh-core` ne peut dépendre de `kesh-db`. Plutôt qu'un second
//! enum (la duplication qu'impose l'orphan rule à `AccountRole`, gardée par un
//! test de cohérence), `kesh-db` lit la colonne en texte et la convertit par
//! [`TryFrom<String>`] (`#[sqlx(try_from = "String")]`), et l'écrit par
//! [`RateKind::as_str`] : un seul type, aucune implémentation `sqlx` requise.

use std::collections::BTreeSet;
use std::fmt;

use chrono::{Datelike, Months, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod parse;

pub use parse::{
    DailyRates, MAX_BODY_BYTES, MAX_DEVISES, MonthlyRates, PublishedRate, RatesError, RejectReason,
    RejectedRate, check_daily_date, check_monthly_month, parse_daily_rates, parse_monthly_rates,
};

/// Genre d'un cours de change : cours **journalier** ou cours **mensuel**
/// (Story 37-1 pour les genres d'exercice, 37-2 pour les cours). Écrit en
/// minuscules au HTTP comme en base (`day`, `month`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RateKind {
    /// Cours journalier.
    Day,
    /// Cours mensuel.
    Month,
}

/// Une valeur de genre inconnue (ni `day`, ni `month`), gardée telle quelle.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("RateKind inconnu : {0}")]
pub struct UnknownRateKind(pub String);

impl RateKind {
    /// Valeur en base et au HTTP : `day` ou `month`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Month => "month",
        }
    }
}

impl fmt::Display for RateKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for RateKind {
    type Err = UnknownRateKind;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "day" => Ok(Self::Day),
            "month" => Ok(Self::Month),
            other => Err(UnknownRateKind(other.to_string())),
        }
    }
}

/// Lecture d'une colonne texte (`ENUM('day','month')` ou `VARCHAR`) par
/// `#[sqlx(try_from = "String")]` dans `kesh-db`.
impl TryFrom<String> for RateKind {
    type Error = UnknownRateKind;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

/// Une unité de cours : un genre et une date — le 1er du mois pour un cours
/// mensuel, le jour pour un cours journalier. C'est l'unité de récupération
/// (une requête à la source) et la clé d'un cours, avec la devise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub struct RateUnit {
    /// La date (1er du mois pour [`RateKind::Month`]). Premier champ : l'ordre
    /// dérivé est chronologique.
    pub date: NaiveDate,
    /// Le genre.
    pub kind: RateKind,
}

impl RateUnit {
    /// Unité mensuelle du mois de `date` (ramenée au 1er).
    pub fn month_of(date: NaiveDate) -> Self {
        Self {
            date: first_of_month(date),
            kind: RateKind::Month,
        }
    }

    /// Unité journalière du jour `date`.
    pub fn day(date: NaiveDate) -> Self {
        Self {
            date,
            kind: RateKind::Day,
        }
    }
}

/// Le 1er du mois de `date`.
pub fn first_of_month(date: NaiveDate) -> NaiveDate {
    date.with_day(1).expect("le 1er existe dans tout mois")
}

/// Le 1er du mois qui suit celui de `date`.
pub fn next_month(date: NaiveDate) -> NaiveDate {
    first_of_month(date)
        .checked_add_months(Months::new(1))
        .expect("mois suivant dans la plage de chrono")
}

/// Plus petite date d'un cours (AC-5 : `POST /fetch`, `PUT …/rates`) :
/// 2000-01-01 — et 2000-01 pour un mois.
pub const MIN_RATE_DATE: NaiveDate = match NaiveDate::from_ymd_opt(2000, 1, 1) {
    Some(d) => d,
    None => panic!("2000-01-01"),
};

// ---------------------------------------------------------------------------
// Bornes d'un cours
// ---------------------------------------------------------------------------

/// Nombre maximal de décimales d'un cours (par unité), la précision de
/// `DECIMAL(20,10)`.
pub const RATE_MAX_SCALE: u32 = 10;

/// Plus grand cours admis (par unité) : 10⁶, la borne du cours d'une ligne en
/// devise (37-3a AC-3), plus stricte que `DECIMAL(20,10)`.
pub const RATE_MAX: Decimal = Decimal::from_parts(1_000_000, 0, 0, false, 0);

/// Pourquoi un cours sort de ses bornes. Erreur **neutre** : chaque appelant la
/// traduit (`FX_RATE_INVALID` du `PUT` d'un cours, `FX_VALUE_OUT_OF_RANGE` de
/// 37-3a, devise écartée par l'analyse des cours publiés).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RateBoundsError {
    /// Nul ou négatif.
    #[error("le cours doit être strictement positif")]
    NotPositive,
    /// Plus de [`RATE_MAX_SCALE`] décimales significatives.
    #[error("le cours a plus de 10 décimales")]
    Scale,
    /// Au-delà de [`RATE_MAX`].
    #[error("le cours dépasse 1 000 000")]
    TooLarge,
}

/// Contrôle les bornes d'**un** cours (par unité) : strictement positif, au plus
/// 10 décimales **après `normalize()`** (les zéros terminaux ne comptent pas :
/// `0.91000000000000` passe), au plus 10⁶.
///
/// ```
/// use kesh_core::exchange_rates::{check_rate_bounds, RateBoundsError};
/// use rust_decimal_macros::dec;
/// assert!(check_rate_bounds(&dec!(0.9123456789)).is_ok());
/// assert_eq!(check_rate_bounds(&dec!(0)), Err(RateBoundsError::NotPositive));
/// assert_eq!(check_rate_bounds(&dec!(0.12345678901)), Err(RateBoundsError::Scale));
/// assert_eq!(check_rate_bounds(&dec!(1000000.1)), Err(RateBoundsError::TooLarge));
/// ```
pub fn check_rate_bounds(rate: &Decimal) -> Result<(), RateBoundsError> {
    let n = rate.normalize();
    if n <= Decimal::ZERO {
        return Err(RateBoundsError::NotPositive);
    }
    if n.scale() > RATE_MAX_SCALE {
        return Err(RateBoundsError::Scale);
    }
    if n > RATE_MAX {
        return Err(RateBoundsError::TooLarge);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Dates d'ouverture et de clôture, unités utiles
// ---------------------------------------------------------------------------

/// Date du cours d'**ouverture** d'un exercice qui commence le `start` (T4,
/// CR-4 : le cours d'ouverture est le cours de clôture de l'exercice
/// précédent) :
/// - [`RateKind::Day`] : la veille de `start` ;
/// - [`RateKind::Month`] : le 1er du mois de cette veille — le dernier mois de
///   l'exercice précédent (pour un exercice qui commence en cours de mois, le
///   mois de `start` lui-même).
///
/// La date se calcule sans borne (un exercice au 01.01.2000 rend le
/// 31.12.1999) : c'est [`unites_utiles`] qui écarte ce qui précède 2000.
///
/// ```
/// use chrono::NaiveDate;
/// use kesh_core::exchange_rates::{opening_rate_date, RateKind};
/// let d = |y, m, j| NaiveDate::from_ymd_opt(y, m, j).unwrap();
/// assert_eq!(opening_rate_date(d(2024, 3, 1), RateKind::Day), d(2024, 2, 29));
/// assert_eq!(opening_rate_date(d(2024, 3, 1), RateKind::Month), d(2024, 2, 1));
/// assert_eq!(opening_rate_date(d(2024, 3, 15), RateKind::Month), d(2024, 3, 1));
/// ```
pub fn opening_rate_date(start: NaiveDate, kind: RateKind) -> NaiveDate {
    let veille = start.pred_opt().expect("veille dans la plage de chrono");
    match kind {
        RateKind::Day => veille,
        RateKind::Month => first_of_month(veille),
    }
}

/// Date du cours de **clôture** d'un exercice qui finit le `end` :
/// [`RateKind::Day`] → `end` ; [`RateKind::Month`] → le 1er du mois de `end`.
pub fn closing_rate_date(end: NaiveDate, kind: RateKind) -> NaiveDate {
    match kind {
        RateKind::Day => end,
        RateKind::Month => first_of_month(end),
    }
}

/// Les mois d'un exercice, du mois de `start` au mois de `end` inclus, dans
/// l'ordre, en 1ers du mois — quelle que soit la durée (S4 : aucune constante
/// de douze mois). Vide si `end < start`.
pub fn exercise_months(start: NaiveDate, end: NaiveDate) -> Vec<NaiveDate> {
    let mut out = Vec::new();
    let last = first_of_month(end);
    let mut m = first_of_month(start);
    while m <= last {
        out.push(m);
        m = next_month(m);
    }
    out
}

/// Ce qu'il faut savoir d'un exercice multi-devises pour ses unités utiles :
/// ses dates et ses deux genres (colonnes de 37-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExerciceDevises {
    /// Premier jour.
    pub start: NaiveDate,
    /// Dernier jour.
    pub end: NaiveDate,
    /// Genre du cours d'ouverture.
    pub opening_kind: RateKind,
    /// Genre du cours de clôture.
    pub closing_kind: RateKind,
}

impl ExerciceDevises {
    /// Date du cours d'ouverture ([`opening_rate_date`]).
    pub fn opening_date(&self) -> NaiveDate {
        opening_rate_date(self.start, self.opening_kind)
    }

    /// Date du cours de clôture ([`closing_rate_date`]).
    pub fn closing_date(&self) -> NaiveDate {
        closing_rate_date(self.end, self.closing_kind)
    }
}

/// Bornes d'« aujourd'hui » (AC-4) : un jour n'est utile que s'il est
/// **strictement antérieur** à aujourd'hui, un mois que s'il ne passe pas le
/// mois suivant le mois courant.
#[derive(Debug, Clone, Copy)]
struct BornesDuJour {
    aujourdhui: NaiveDate,
    mois_suivant: NaiveDate,
}

/// Ajoute à `out` les unités d'un exercice : ses mois, son ouverture et sa
/// clôture selon ses genres ; bornes 2000-01 / 2000-01-01 toujours ; bornes
/// d'aujourd'hui si `jour` est donné.
fn ajouter_unites(ex: &ExerciceDevises, jour: Option<BornesDuJour>, out: &mut BTreeSet<RateUnit>) {
    let mois_admis = |m: NaiveDate| m >= MIN_RATE_DATE && jour.is_none_or(|b| m <= b.mois_suivant);
    let jour_admis = |d: NaiveDate| d >= MIN_RATE_DATE && jour.is_none_or(|b| d < b.aujourdhui);
    let mut ajouter = |u: RateUnit| {
        let admis = match u.kind {
            RateKind::Month => mois_admis(u.date),
            RateKind::Day => jour_admis(u.date),
        };
        if admis {
            out.insert(u);
        }
    };
    for m in exercise_months(ex.start, ex.end) {
        ajouter(RateUnit::month_of(m));
    }
    let ouverture = ex.opening_date();
    let cloture = ex.closing_date();
    ajouter(RateUnit {
        date: ouverture,
        kind: ex.opening_kind,
    });
    ajouter(RateUnit {
        date: cloture,
        kind: ex.closing_kind,
    });
}

/// Les unités **à récupérer** pour des exercices multi-devises **ouverts**
/// (AC-4, R2) — l'appelant ne passe que ceux-là :
/// - les mois de chaque exercice, de son premier mois au plus petit de son
///   dernier mois et du **mois suivant** le mois courant ;
/// - l'ouverture et la clôture selon les genres ([`opening_rate_date`],
///   [`closing_rate_date`]) : un jour seulement s'il est **strictement
///   antérieur** à `aujourdhui`, un mois sous la même borne que les autres ;
/// - rien avant 2000-01 (mois) ni avant 2000-01-01 (jour) : la source ne les
///   sert pas, et la tâche les redemanderait sans fin.
///
/// Sans doublon (un exercice qui commence en cours de mois a pour mois
/// d'ouverture `month` son propre premier mois), dans l'ordre chronologique.
/// `aujourdhui` est le jour de Zurich (`horloge::Horloge::aujourd_hui`).
pub fn unites_utiles(exercices: &[ExerciceDevises], aujourdhui: NaiveDate) -> Vec<RateUnit> {
    let bornes = BornesDuJour {
        aujourdhui,
        mois_suivant: next_month(aujourdhui),
    };
    let mut out = BTreeSet::new();
    for ex in exercices {
        ajouter_unites(ex, Some(bornes), &mut out);
    }
    out.into_iter().collect()
}

/// Les unités utiles **d'un** exercice, pour la copie des cours récupérés
/// (AC-3 bis) : le même ensemble que [`unites_utiles`], **sans** les bornes
/// d'aujourd'hui et du mois suivant (une valeur déjà publiée se copie quelle
/// que soit sa date) ; les bornes de 2000 restent.
pub fn unites_utiles_exercice(exercice: &ExerciceDevises) -> Vec<RateUnit> {
    let mut out = BTreeSet::new();
    ajouter_unites(exercice, None, &mut out);
    out.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(y: i32, m: u32, j: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, j).unwrap()
    }

    fn ex(start: NaiveDate, end: NaiveDate, o: RateKind, c: RateKind) -> ExerciceDevises {
        ExerciceDevises {
            start,
            end,
            opening_kind: o,
            closing_kind: c,
        }
    }

    fn mois(us: &[RateUnit]) -> Vec<NaiveDate> {
        us.iter()
            .filter(|u| u.kind == RateKind::Month)
            .map(|u| u.date)
            .collect()
    }

    fn jours(us: &[RateUnit]) -> Vec<NaiveDate> {
        us.iter()
            .filter(|u| u.kind == RateKind::Day)
            .map(|u| u.date)
            .collect()
    }

    fn plage(debut: NaiveDate, fin: NaiveDate) -> Vec<NaiveDate> {
        exercise_months(debut, fin)
    }

    #[test]
    fn rate_kind_texte_aller_retour() {
        for k in [RateKind::Day, RateKind::Month] {
            assert_eq!(k.as_str().parse::<RateKind>().unwrap(), k);
            assert_eq!(RateKind::try_from(k.as_str().to_string()).unwrap(), k);
            assert_eq!(k.to_string(), k.as_str());
            assert_eq!(serde_json::to_string(&k).unwrap(), format!("\"{k}\""));
        }
        assert_eq!(
            "Month".parse::<RateKind>(),
            Err(UnknownRateKind("Month".into()))
        );
        assert!(serde_json::from_str::<RateKind>("\"week\"").is_err());
    }

    #[test]
    fn bornes_d_un_cours() {
        assert_eq!(check_rate_bounds(&dec!(0.0000000001)), Ok(()));
        assert_eq!(check_rate_bounds(&dec!(1000000)), Ok(()));
        assert_eq!(check_rate_bounds(&dec!(1000000.0000)), Ok(()));
        // Zéros terminaux ignorés.
        assert_eq!(check_rate_bounds(&dec!(0.912300000000000)), Ok(()));
        assert_eq!(
            check_rate_bounds(&dec!(0)),
            Err(RateBoundsError::NotPositive)
        );
        assert_eq!(
            check_rate_bounds(&dec!(-0.5)),
            Err(RateBoundsError::NotPositive)
        );
        assert_eq!(
            check_rate_bounds(&dec!(0.00000000001)),
            Err(RateBoundsError::Scale)
        );
        assert_eq!(
            check_rate_bounds(&dec!(0.12345678901)),
            Err(RateBoundsError::Scale)
        );
        assert_eq!(
            check_rate_bounds(&dec!(1000000.0000000001)),
            Err(RateBoundsError::TooLarge)
        );
        assert_eq!(RATE_MAX, dec!(1000000));
    }

    #[test]
    fn mois_d_un_exercice_quelle_que_soit_sa_duree() {
        assert_eq!(exercise_months(d(2024, 7, 1), d(2024, 12, 31)).len(), 6);
        assert_eq!(exercise_months(d(2023, 7, 1), d(2024, 12, 31)).len(), 18);
        let m = exercise_months(d(2024, 3, 15), d(2025, 3, 14));
        assert_eq!(m.len(), 13);
        assert_eq!(m.first(), Some(&d(2024, 3, 1)));
        assert_eq!(m.last(), Some(&d(2025, 3, 1)));
        let juillet = exercise_months(d(2024, 7, 1), d(2025, 6, 30));
        assert_eq!(juillet.first(), Some(&d(2024, 7, 1)));
        assert_eq!(juillet.last(), Some(&d(2025, 6, 1)));
        assert!(exercise_months(d(2025, 1, 1), d(2024, 1, 1)).is_empty());
    }

    /// Exercice civil (1er janvier, T2) : veille = 31.12 de l'année
    /// précédente, mois d'ouverture `month` = décembre précédent.
    #[test]
    fn unites_exercice_civil() {
        let auj = d(2026, 6, 10);
        let jour = ex(d(2025, 1, 1), d(2025, 12, 31), RateKind::Day, RateKind::Day);
        let u = unites_utiles(&[jour], auj);
        assert_eq!(mois(&u), plage(d(2025, 1, 1), d(2025, 12, 1)));
        assert_eq!(jours(&u), vec![d(2024, 12, 31), d(2025, 12, 31)]);
        let mens = ex(
            d(2025, 1, 1),
            d(2025, 12, 31),
            RateKind::Month,
            RateKind::Month,
        );
        let u = unites_utiles(&[mens], auj);
        assert_eq!(mois(&u), plage(d(2024, 12, 1), d(2025, 12, 1)));
        assert!(jours(&u).is_empty());
        // Ordre chronologique strict, sans doublon.
        assert!(u.windows(2).all(|w| w[0] < w[1]));
    }

    /// Exercice au 1er avril et au 1er juillet : veilles 31.03 / 30.06, mois
    /// d'ouverture `month` mars / juin, jour de clôture.
    #[test]
    fn unites_exercice_avril_et_juillet() {
        let auj = d(2026, 9, 1);
        let avril = ex(d(2025, 4, 1), d(2026, 3, 31), RateKind::Day, RateKind::Day);
        let u = unites_utiles(&[avril], auj);
        assert_eq!(jours(&u), vec![d(2025, 3, 31), d(2026, 3, 31)]);
        assert_eq!(mois(&u), plage(d(2025, 4, 1), d(2026, 3, 1)));
        let avril_m = ex(
            d(2025, 4, 1),
            d(2026, 3, 31),
            RateKind::Month,
            RateKind::Day,
        );
        let u = unites_utiles(&[avril_m], auj);
        assert_eq!(mois(&u).first(), Some(&d(2025, 3, 1)));
        assert_eq!(mois(&u).len(), 13);
        assert_eq!(jours(&u), vec![d(2026, 3, 31)]);

        let juillet = ex(
            d(2025, 7, 1),
            d(2026, 6, 30),
            RateKind::Month,
            RateKind::Day,
        );
        let u = unites_utiles(&[juillet], auj);
        assert_eq!(mois(&u), plage(d(2025, 6, 1), d(2026, 6, 1)));
        assert_eq!(jours(&u), vec![d(2026, 6, 30)]);
        let juillet_j = ex(
            d(2025, 7, 1),
            d(2026, 6, 30),
            RateKind::Day,
            RateKind::Month,
        );
        let u = unites_utiles(&[juillet_j], auj);
        assert_eq!(jours(&u), vec![d(2025, 6, 30)]);
        assert_eq!(mois(&u), plage(d(2025, 7, 1), d(2026, 6, 1)));
    }

    /// Six et dix-huit mois (S4) ; exercice ouvert jusqu'au mois suivant
    /// seulement ; jour de clôture futur écarté.
    #[test]
    fn unites_six_et_dix_huit_mois_et_bornes_du_jour() {
        let auj = d(2024, 9, 14);
        let six = ex(d(2024, 7, 1), d(2024, 12, 31), RateKind::Day, RateKind::Day);
        let u = unites_utiles(&[six], auj);
        // Juillet à octobre (mois suivant le mois courant) ; clôture future.
        assert_eq!(mois(&u), plage(d(2024, 7, 1), d(2024, 10, 1)));
        assert_eq!(jours(&u), vec![d(2024, 6, 30)]);
        // Sans bornes du jour : les six mois et les deux jours.
        let u = unites_utiles_exercice(&six);
        assert_eq!(mois(&u), plage(d(2024, 7, 1), d(2024, 12, 1)));
        assert_eq!(jours(&u), vec![d(2024, 6, 30), d(2024, 12, 31)]);

        let dix_huit = ex(
            d(2023, 7, 1),
            d(2024, 12, 31),
            RateKind::Month,
            RateKind::Month,
        );
        let u = unites_utiles(&[dix_huit], d(2025, 2, 1));
        assert_eq!(mois(&u), plage(d(2023, 6, 1), d(2024, 12, 1)));
        assert_eq!(mois(&u).len(), 19);
        assert!(jours(&u).is_empty());
    }

    /// Un jour égal à aujourd'hui n'est pas utile (strictement antérieur).
    #[test]
    fn jour_d_aujourd_hui_ecarte() {
        let e = ex(d(2025, 1, 1), d(2025, 12, 31), RateKind::Day, RateKind::Day);
        assert_eq!(
            jours(&unites_utiles(&[e], d(2025, 12, 31))),
            vec![d(2024, 12, 31)]
        );
        assert_eq!(
            jours(&unites_utiles(&[e], d(2026, 1, 1))),
            vec![d(2024, 12, 31), d(2025, 12, 31)]
        );
        assert_eq!(
            jours(&unites_utiles(&[e], d(2024, 12, 31))),
            Vec::<NaiveDate>::new()
        );
    }

    /// Exercice au 15.03 : treize mois de mars à mars ; veille 14.03 ; en genre
    /// `month`, le mois d'ouverture est mars, déjà le premier mois — pas de
    /// doublon.
    #[test]
    fn unites_exercice_au_15_mars() {
        let auj = d(2026, 1, 1);
        let m = ex(
            d(2024, 3, 15),
            d(2025, 3, 14),
            RateKind::Month,
            RateKind::Month,
        );
        let u = unites_utiles(&[m], auj);
        assert_eq!(mois(&u), plage(d(2024, 3, 1), d(2025, 3, 1)));
        assert_eq!(u.len(), 13);
        let j = ex(d(2024, 3, 15), d(2025, 3, 14), RateKind::Day, RateKind::Day);
        let u = unites_utiles(&[j], auj);
        assert_eq!(jours(&u), vec![d(2024, 3, 14), d(2025, 3, 14)]);
        assert_eq!(u.len(), 15);
    }

    /// Exercice au 01.01.2000 : ni la veille (31.12.1999) ni son mois.
    #[test]
    fn unites_avant_2000_ecartees() {
        let auj = d(2026, 1, 1);
        for o in [RateKind::Day, RateKind::Month] {
            let e = ex(d(2000, 1, 1), d(2000, 12, 31), o, RateKind::Day);
            let u = unites_utiles(&[e], auj);
            assert!(u.iter().all(|x| x.date >= MIN_RATE_DATE), "{o:?} : {u:?}");
            assert_eq!(mois(&u), plage(d(2000, 1, 1), d(2000, 12, 1)));
            assert_eq!(jours(&u), vec![d(2000, 12, 31)]);
            let u = unites_utiles_exercice(&e);
            assert!(u.iter().all(|x| x.date >= MIN_RATE_DATE), "{o:?} : {u:?}");
        }
    }

    /// Deux exercices consécutifs : l'ouverture du second (mois de la veille)
    /// est un mois du premier — un seul exemplaire.
    #[test]
    fn unites_de_plusieurs_exercices_sans_doublon() {
        let a = ex(
            d(2024, 1, 1),
            d(2024, 12, 31),
            RateKind::Day,
            RateKind::Month,
        );
        let b = ex(
            d(2025, 1, 1),
            d(2025, 12, 31),
            RateKind::Month,
            RateKind::Day,
        );
        let u = unites_utiles(&[b, a], d(2026, 3, 1));
        assert_eq!(mois(&u), plage(d(2024, 1, 1), d(2025, 12, 1)));
        assert_eq!(jours(&u), vec![d(2023, 12, 31), d(2025, 12, 31)]);
        assert!(u.windows(2).all(|w| w[0] < w[1]));
        assert!(unites_utiles(&[], d(2026, 3, 1)).is_empty());
    }

    /// Exercice futur : aucun mois au-delà du mois suivant, mois d'ouverture
    /// compris.
    #[test]
    fn exercice_futur_borne_au_mois_suivant() {
        let e = ex(
            d(2027, 1, 1),
            d(2027, 12, 31),
            RateKind::Month,
            RateKind::Month,
        );
        assert!(unites_utiles(&[e], d(2026, 10, 16)).is_empty());
        // Mois suivant = décembre : le mois d'ouverture seul.
        assert_eq!(
            unites_utiles(&[e], d(2026, 11, 16)),
            vec![RateUnit::month_of(d(2026, 12, 1))]
        );
        assert_eq!(
            unites_utiles(&[e], d(2026, 12, 5)),
            vec![
                RateUnit::month_of(d(2026, 12, 1)),
                RateUnit::month_of(d(2027, 1, 1))
            ]
        );
    }

    #[test]
    fn mois_suivant_et_premier_du_mois() {
        assert_eq!(next_month(d(2024, 12, 31)), d(2025, 1, 1));
        assert_eq!(next_month(d(2024, 1, 31)), d(2024, 2, 1));
        assert_eq!(first_of_month(d(2024, 2, 29)), d(2024, 2, 1));
        assert_eq!(RateUnit::month_of(d(2024, 5, 17)).date, d(2024, 5, 1));
    }
}
