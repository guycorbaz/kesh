//! Réévaluation de fin d'exercice des comptes en devise (Story 37-5) — calcul
//! pur, sans I/O.
//!
//! # La règle, en débit net
//!
//! Pour un compte de bilan en devise, à la date de fin d'exercice :
//!
//! - `devise_net` = Σ montants en devise au débit − Σ au crédit, sur **toutes**
//!   les lignes du compte depuis l'origine ;
//! - `chf_net` = Σ débit − Σ crédit en CHF, sur les mêmes lignes ;
//! - CHF réévalué = `round(devise_net × cours, 0.01)`, l'équidistant loin de
//!   zéro ;
//! - **écart** `D` = CHF réévalué − `chf_net`.
//!
//! `D > 0` : débit du compte réévalué, crédit du compte de **gains** ; `D < 0` :
//! crédit du compte réévalué, débit du compte de **pertes** ; `D = 0` : rien.
//! Ce sens vaut pour **tout** signe de solde et tout type de compte (un actif à
//! solde créditeur dont le cours monte donne une perte) : il n'y a aucune
//! conversion de signe par type, le signe naturel ne sert qu'à l'affichage.
//!
//! # Bornes
//!
//! `devise_net` est un **cumul**, non borné par ligne : le produit se calcule
//! en entiers `i128` par multiplication **contrôlée** (`checked_mul`), jamais
//! par `Decimal::checked_mul`, qui réduit l'échelle en silence au-delà de
//! 28 chiffres. Un CHF réévalué ou un `|D|` au-delà de 10¹³ est refusé
//! ([`RevaluationError::OutOfRange`], `FX_VALUE_OUT_OF_RANGE` chez l'appelant).
//!
//! # Le genre d'un cours reçu
//!
//! [`received_rate_kind`] est la **règle unique** de l'aperçu, de la passation
//! et de l'audit : un cours reçu égal (numériquement) au cours par défaut prend
//! le genre de ce défaut (`day` ou `month`) ; différent, ou sans défaut, il est
//! `request`.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::accounting::fx::{FX_AMOUNT_MAX_SCALE, FX_CHF_MAX};
use crate::exchange_rates::{RATE_MAX_SCALE, RateKind, check_rate_bounds};

/// Genre d'un cours **retenu** pour une réévaluation : le genre du cours par
/// défaut (`day`, `month`), ou `request` pour un cours reçu qui en diffère.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RevaluationRateKind {
    /// Le cours journalier par défaut (la date de fin d'exercice).
    Day,
    /// Le cours mensuel par défaut (le dernier mois de l'exercice).
    Month,
    /// Un cours reçu différent du défaut, ou sans défaut.
    Request,
}

impl RevaluationRateKind {
    /// La valeur au JSON et à l'audit : `day`, `month` ou `request`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Day => "day",
            Self::Month => "month",
            Self::Request => "request",
        }
    }
}

impl From<RateKind> for RevaluationRateKind {
    fn from(kind: RateKind) -> Self {
        match kind {
            RateKind::Day => Self::Day,
            RateKind::Month => Self::Month,
        }
    }
}

/// Refus du calcul, neutre (l'appelant le traduit avec la devise et le compte).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevaluationError {
    /// Cours hors bornes, solde en devise à plus de 4 décimales, produit hors
    /// de l'`i128`, CHF réévalué ou `|D|` au-delà de 10¹³.
    OutOfRange,
}

/// Résultat du calcul d'un compte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevaluationDelta {
    /// CHF réévalué : `round(devise_net × cours, 0.01)`.
    pub target_chf: Decimal,
    /// Écart `D` = CHF réévalué − `chf_net`, en débit net.
    pub delta: Decimal,
}

/// Côté d'une écriture de réévaluation, lu sur le signe de `D`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevaluationSide {
    /// `D > 0` : débit du compte réévalué, crédit du compte de gains.
    Gain,
    /// `D < 0` : crédit du compte réévalué, débit du compte de pertes.
    Loss,
}

/// Le côté de `delta`, `None` pour un écart nul.
pub fn revaluation_side(delta: &Decimal) -> Option<RevaluationSide> {
    if delta.is_zero() {
        None
    } else if delta.is_sign_positive() {
        Some(RevaluationSide::Gain)
    } else {
        Some(RevaluationSide::Loss)
    }
}

/// Mantisse entière d'un décimal à l'échelle `scale` exacte (`None` si son
/// échelle normalisée la dépasse ou si la mise à l'échelle déborde).
fn scaled_int(d: &Decimal, scale: u32) -> Option<i128> {
    let n = d.normalize();
    if n.scale() > scale {
        return None;
    }
    n.mantissa()
        .checked_mul(10i128.checked_pow(scale - n.scale())?)
}

/// L'écart de réévaluation d'un compte (règle de la doc du module).
///
/// ```
/// use kesh_core::accounting::fx_revaluation::revaluation_delta;
/// use rust_decimal_macros::dec;
/// // 1 000 USD tenus à 880 CHF, cours de clôture 0.91193 : gain de 31.93.
/// let r = revaluation_delta(&dec!(1000), &dec!(880), &dec!(0.91193)).unwrap();
/// assert_eq!(r.target_chf, dec!(911.93));
/// assert_eq!(r.delta, dec!(31.93));
/// ```
pub fn revaluation_delta(
    devise_net: &Decimal,
    chf_net: &Decimal,
    rate: &Decimal,
) -> Result<RevaluationDelta, RevaluationError> {
    check_rate_bounds(rate).map_err(|_| RevaluationError::OutOfRange)?;
    let a = scaled_int(devise_net, FX_AMOUNT_MAX_SCALE).ok_or(RevaluationError::OutOfRange)?;
    let r = scaled_int(rate, RATE_MAX_SCALE).ok_or(RevaluationError::OutOfRange)?;
    // Produit à l'échelle 4 + 10 = 14 ; arrondi au centime (échelle 2) :
    // division par 10¹², l'équidistant loin de zéro.
    let p = a.checked_mul(r).ok_or(RevaluationError::OutOfRange)?;
    let div: i128 = 10i128.pow(FX_AMOUNT_MAX_SCALE + RATE_MAX_SCALE - 2);
    let q = p / div;
    let rem = p % div;
    let cents = if rem.abs() * 2 >= div {
        q + p.signum()
    } else {
        q
    };
    let target_chf =
        Decimal::try_from_i128_with_scale(cents, 2).map_err(|_| RevaluationError::OutOfRange)?;
    if target_chf.abs() > FX_CHF_MAX {
        return Err(RevaluationError::OutOfRange);
    }
    let delta = target_chf
        .checked_sub(*chf_net)
        .ok_or(RevaluationError::OutOfRange)?;
    if delta.abs() > FX_CHF_MAX {
        return Err(RevaluationError::OutOfRange);
    }
    Ok(RevaluationDelta {
        target_chf,
        delta: chf_cents(delta),
    })
}

/// Écrit un montant CHF à deux décimales au moins (`31.9` → `31.90`, `0` →
/// `0.00`), sans en perdre (un résidu à 4 décimales reste écrit tel quel).
pub fn chf_cents(amount: Decimal) -> Decimal {
    let mut n = amount.normalize();
    if n.scale() < 2 {
        n.rescale(2);
    }
    n
}

/// Le genre d'un cours **reçu** (règle unique de l'aperçu, de la passation et
/// de l'audit) : égal numériquement au cours par défaut (zéros terminaux
/// ignorés) → genre de ce défaut ; différent, ou sans défaut → `request`.
///
/// ```
/// use kesh_core::accounting::fx_revaluation::{received_rate_kind, RevaluationRateKind};
/// use kesh_core::exchange_rates::RateKind;
/// use rust_decimal_macros::dec;
/// let defaut = Some((dec!(0.8846), RateKind::Month));
/// assert_eq!(received_rate_kind(&dec!(0.884600), defaut), RevaluationRateKind::Month);
/// assert_eq!(received_rate_kind(&dec!(0.91193), defaut), RevaluationRateKind::Request);
/// assert_eq!(received_rate_kind(&dec!(0.91193), None), RevaluationRateKind::Request);
/// ```
pub fn received_rate_kind(
    received: &Decimal,
    default: Option<(Decimal, RateKind)>,
) -> RevaluationRateKind {
    match default {
        Some((rate, kind)) if rate.normalize() == received.normalize() => kind.into(),
        _ => RevaluationRateKind::Request,
    }
}

/// Totaux affichés : Σ des écarts positifs (gains) et Σ des |écarts négatifs|
/// (pertes), en `Decimal`, sans écriture (aucune somme n'est passée : une
/// écriture par compte).
pub fn revaluation_totals<'a>(deltas: impl IntoIterator<Item = &'a Decimal>) -> (Decimal, Decimal) {
    let mut gains = Decimal::ZERO;
    let mut losses = Decimal::ZERO;
    for d in deltas {
        if d.is_sign_positive() {
            gains += *d;
        } else {
            losses += d.abs();
        }
    }
    (gains, losses)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(devise: Decimal, chf: Decimal, rate: Decimal) -> Decimal {
        revaluation_delta(&devise, &chf, &rate).unwrap().delta
    }

    // --- Les cinq cas de signe (AC-7) ---------------------------------------

    #[test]
    fn actif_debiteur_cours_en_hausse_donne_un_gain() {
        // 1000 USD au débit, tenus à 880 CHF ; cours 0.92 → 920.
        let delta = d(dec!(1000), dec!(880), dec!(0.92));
        assert_eq!(delta, dec!(40.00));
        assert_eq!(revaluation_side(&delta), Some(RevaluationSide::Gain));
    }

    #[test]
    fn actif_debiteur_cours_en_baisse_donne_une_perte() {
        let delta = d(dec!(1000), dec!(880), dec!(0.85));
        assert_eq!(delta, dec!(-30.00));
        assert_eq!(revaluation_side(&delta), Some(RevaluationSide::Loss));
    }

    #[test]
    fn actif_a_solde_crediteur_cours_en_hausse_donne_une_perte() {
        // Débit net −500 USD (−440 CHF) ; cours 0.92 → −460 : D = −20.
        let delta = d(dec!(-500), dec!(-440), dec!(0.92));
        assert_eq!(delta, dec!(-20.00));
        assert_eq!(revaluation_side(&delta), Some(RevaluationSide::Loss));
    }

    #[test]
    fn passif_crediteur_cours_en_hausse_donne_une_perte() {
        // Passif : débit net −2000 USD (−1760 CHF) ; cours 0.90 → −1800.
        let delta = d(dec!(-2000), dec!(-1760), dec!(0.90));
        assert_eq!(delta, dec!(-40.00));
        assert_eq!(revaluation_side(&delta), Some(RevaluationSide::Loss));
    }

    #[test]
    fn passif_a_solde_debiteur_cours_en_hausse_donne_un_gain() {
        let delta = d(dec!(300), dec!(264), dec!(0.90));
        assert_eq!(delta, dec!(6.00));
        assert_eq!(revaluation_side(&delta), Some(RevaluationSide::Gain));
    }

    #[test]
    fn devise_nulle_et_chf_residuel_donne_moins_le_chf() {
        assert_eq!(d(dec!(0), dec!(12.35), dec!(0.9)), dec!(-12.35));
        assert_eq!(d(dec!(0), dec!(-7.10), dec!(0.9)), dec!(7.10));
    }

    #[test]
    fn ecart_nul_n_a_pas_de_cote() {
        let delta = d(dec!(1000), dec!(911.93), dec!(0.91193));
        assert!(delta.is_zero());
        assert_eq!(revaluation_side(&delta), None);
    }

    // --- Arrondi et bornes ---------------------------------------------------

    #[test]
    fn arrondi_au_centime_l_equidistant_loin_de_zero() {
        // 1 × 0.125 = 0.125 → 0.13 ; −1 × 0.125 → −0.13.
        assert_eq!(
            revaluation_delta(&dec!(1), &dec!(0), &dec!(0.125))
                .unwrap()
                .target_chf,
            dec!(0.13)
        );
        assert_eq!(
            revaluation_delta(&dec!(-1), &dec!(0), &dec!(0.125))
                .unwrap()
                .target_chf,
            dec!(-0.13)
        );
    }

    #[test]
    fn cours_hors_bornes_refuse() {
        for rate in [dec!(0), dec!(-1), dec!(0.12345678901), dec!(1000000.1)] {
            assert_eq!(
                revaluation_delta(&dec!(1), &dec!(0), &rate),
                Err(RevaluationError::OutOfRange),
                "{rate}"
            );
        }
    }

    #[test]
    fn produit_ou_ecart_au_dela_de_dix_puissance_treize_refuse() {
        // 10¹³ USD × 2 = 2·10¹³ CHF : au-delà.
        assert_eq!(
            revaluation_delta(&dec!(10000000000000), &dec!(0), &dec!(2)),
            Err(RevaluationError::OutOfRange)
        );
        // Un cumul énorme qui déborde l'i128 : refusé, jamais tronqué.
        assert_eq!(
            revaluation_delta(&Decimal::MAX, &dec!(0), &dec!(1000000)),
            Err(RevaluationError::OutOfRange)
        );
        // CHF réévalué dans les bornes, mais |D| au-delà.
        assert_eq!(
            revaluation_delta(&dec!(9000000000000), &dec!(-9000000000000), &dec!(1)),
            Err(RevaluationError::OutOfRange)
        );
    }

    #[test]
    fn solde_en_devise_a_plus_de_quatre_decimales_refuse() {
        assert_eq!(
            revaluation_delta(&dec!(1.00001), &dec!(0), &dec!(1)),
            Err(RevaluationError::OutOfRange)
        );
    }

    // --- Genre d'un cours reçu ----------------------------------------------

    #[test]
    fn genre_d_un_cours_recu() {
        let jour = Some((dec!(0.91193), RateKind::Day));
        assert_eq!(
            received_rate_kind(&dec!(0.9119300), jour),
            RevaluationRateKind::Day
        );
        assert_eq!(
            received_rate_kind(&dec!(0.9119), jour),
            RevaluationRateKind::Request
        );
        assert_eq!(
            received_rate_kind(&dec!(0.8846), Some((dec!(0.8846), RateKind::Month))),
            RevaluationRateKind::Month
        );
        assert_eq!(
            received_rate_kind(&dec!(1), None),
            RevaluationRateKind::Request
        );
        assert_eq!(RevaluationRateKind::Request.as_str(), "request");
    }

    #[test]
    fn totaux_des_gains_et_des_pertes() {
        let deltas = [dec!(10.50), dec!(-3.25), dec!(0), dec!(-1.75)];
        assert_eq!(revaluation_totals(&deltas), (dec!(10.50), dec!(5.00)));
    }
}
