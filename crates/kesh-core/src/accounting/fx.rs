//! Lignes d'écriture en devise (Story 37-3a, AC-2, AC-3).
//!
//! Une ligne en devise porte, en plus de son débit ou crédit en devise
//! principale (CHF), un [`ForeignAmount`] : le montant en devise, le cours par
//! unité et la provenance du cours. **La devise n'est pas portée par la ligne**
//! : elle est celle du compte (T3, CR-4), lue et contrôlée par la persistance.
//!
//! Ce module ne fait aucune I/O. Il fournit :
//!
//! - [`fx_chf`] — le produit `montant × cours` arrondi au centime, calculé en
//!   **entiers `i128`** (jamais par `Decimal::checked_mul`, dont la
//!   multiplication réduit l'échelle en silence au-delà de 28 chiffres), avec
//!   les bornes d'entrée ;
//! - [`check_foreign_line`] — le contrôle d'**une** ligne en devise, dans
//!   l'ordre de l'AC-3 (bornes, signe, arrondi nul, égalité au CHF fourni) ;
//! - [`rate_from_chf`] — le cours déduit d'un CHF (V1, CR-6), jumeau du
//!   `rateFromChf` de l'écran (37-3b).
//!
//! Les erreurs de ce module sont **neutres** : [`crate::accounting::validate`]
//! et les routes les traduisent en `CoreError::Fx*` avec le rang de la ligne.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::exchange_rates::check_rate_bounds;

/// Nombre maximal de décimales d'un montant en devise (`DECIMAL(19,4)`).
pub const FX_AMOUNT_MAX_SCALE: u32 = 4;

/// Plus grand montant en devise admis : 10¹³.
pub const FX_AMOUNT_MAX: Decimal = Decimal::from_parts(1_316_134_912, 2_328, 0, false, 0);

/// Plus grand montant CHF admis pour une ligne en devise : 10¹³ (la colonne
/// `debit`/`credit` est `DECIMAL(19,4)` ; au-delà, ce serait une erreur SQL).
pub const FX_CHF_MAX: Decimal = FX_AMOUNT_MAX;

/// Nombre maximal de pas de 10⁻¹⁰ que [`rate_from_chf`] essaie de part et
/// d'autre du cours arrondi pour retrouver le CHF visé. Le jumeau TypeScript
/// (`rateFromChf` de `fx.ts`, 37-3b) emploie la **même** borne.
pub const RATE_FROM_CHF_MAX_STEPS: u32 = 10;

/// Provenance du cours d'une ligne en devise (colonne `rate_source`).
///
/// ⚠️ Nommée `RateProvenance` et non `RateSource` : `RateSource` est le trait
/// de la source des cours publiés (Story 37-2, `kesh-api`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RateProvenance {
    /// Le cours mensuel retenu par la société, de source `afc`, pour le mois
    /// de la date de l'écriture — contrôlé par la persistance.
    Afc,
    /// Un cours saisi (ou repris d'ailleurs) : jamais contrôlé.
    Manual,
    /// Ligne de réévaluation (Story 37-5) : montant en devise nul, CHF non nul.
    Revaluation,
}

impl RateProvenance {
    /// La valeur telle qu'en base (`rate_source`) et dans le JSON.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Afc => "afc",
            Self::Manual => "manual",
            Self::Revaluation => "revaluation",
        }
    }
}

impl std::str::FromStr for RateProvenance {
    type Err = String;

    /// Analyse stricte, sensible à la casse (comme le `CHECK BINARY`).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "afc" => Ok(Self::Afc),
            "manual" => Ok(Self::Manual),
            "revaluation" => Ok(Self::Revaluation),
            other => Err(format!("provenance de cours inconnue : {other}")),
        }
    }
}

/// Partie « devise » d'une ligne d'écriture (AC-2).
///
/// Montant et cours en `Decimal` (non en `Money`, réservé au CHF).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForeignAmount {
    /// Devise **annoncée** par le client, jamais stockée (T3) : `None` pour un
    /// client qui ne l'envoie pas et pour les chemins internes. La persistance
    /// la compare à la devise du compte (`FX_CURRENCY_MISMATCH`) ; ce module ne
    /// la lit pas.
    pub declared_currency: Option<String>,
    /// Montant en devise, ≥ 0 (0 pour une ligne de réévaluation seulement).
    pub amount: Decimal,
    /// Cours par unité de devise, > 0.
    pub rate: Decimal,
    /// Provenance du cours. `None` **seulement** pour une ligne venue d'une
    /// route de l'API sans `rateSource` : la persistance la qualifie `Afc` ou
    /// `Manual` avant l'`INSERT` (un `None` à l'`INSERT` est un invariant
    /// violé). Les chemins internes posent toujours `Some`.
    pub source: Option<RateProvenance>,
    /// Marqueur **non stocké** : le cours a été déduit du CHF par
    /// [`rate_from_chf`] parce que le corps ne portait pas de `rate`
    /// (CR6-P2-1). Faux sur tout chemin interne ; ni stocké, ni rendu.
    #[serde(skip)]
    pub rate_derived: bool,
}

/// Refus d'une ligne en devise, neutre (sans rang de ligne).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FxLineError {
    /// Échelle, borne ou signe hors limites (`FX_VALUE_OUT_OF_RANGE`).
    OutOfRange,
    /// Le CHF calculé s'arrondit à 0 (`FX_LINE_ROUNDS_TO_ZERO`).
    RoundsToZero,
    /// Le CHF fourni (`debit + credit`) diffère du CHF calculé
    /// (`FX_AMOUNT_MISMATCH`).
    Mismatch {
        /// CHF calculé, à deux décimales.
        expected: Decimal,
        /// CHF fourni.
        provided: Decimal,
    },
}

/// Refus de [`rate_from_chf`], neutre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RateFromChfError {
    /// Montant ou CHF hors bornes, ou cours obtenu hors bornes
    /// (`FX_VALUE_OUT_OF_RANGE`).
    OutOfRange,
    /// Aucun cours à 10 décimales ne redonne le CHF (`FX_AMOUNT_MISMATCH`) :
    /// `expected` est le CHF du cours le plus proche.
    Mismatch {
        /// CHF du cours le plus proche, à deux décimales.
        expected: Decimal,
    },
}

/// Mantisse entière d'un décimal à l'échelle `scale` exacte (`None` si
/// l'échelle normalisée la dépasse).
fn scaled_int(d: &Decimal, scale: u32) -> Option<i128> {
    let n = d.normalize();
    if n.scale() > scale {
        return None;
    }
    let m = n.mantissa();
    m.checked_mul(10i128.pow(scale - n.scale()))
}

/// Contrôle les bornes d'un montant en devise : au plus 4 décimales après
/// `normalize()`, au plus 10¹³ en valeur absolue. Le signe n'est pas jugé ici.
fn amount_in_bounds(amount: &Decimal) -> bool {
    amount.normalize().scale() <= FX_AMOUNT_MAX_SCALE && amount.abs() <= FX_AMOUNT_MAX
}

/// Le CHF d'une ligne : `montant × cours` arrondi au centime, l'équidistant
/// loin de zéro (`MidpointAwayFromZero`, comme le `ROUND` de MariaDB sur un
/// `DECIMAL`, que reprend `chk_jel_fx_chf`).
///
/// Bornes d'entrée (étape 1 de l'AC-3) : montant à au plus 4 décimales et au
/// plus 10¹³ en valeur absolue ; cours contrôlé par
/// [`check_rate_bounds`] (> 0, au plus 10 décimales, au plus 10⁶) ; CHF
/// arrondi au plus 10¹³. Sinon [`FxLineError::OutOfRange`].
///
/// Le produit se calcule en entiers : montant × 10⁴ (≤ 10¹⁷) fois cours × 10¹⁰
/// (≤ 10¹⁶), soit au plus 10³³ < 1,7·10³⁸ — exact, sans l'arrondi silencieux
/// de `Decimal` au-delà de 28 chiffres.
///
/// ```
/// use kesh_core::accounting::fx::fx_chf;
/// use rust_decimal_macros::dec;
/// assert_eq!(fx_chf(&dec!(1234.56), &dec!(0.91193)).unwrap(), dec!(1125.83));
/// assert_eq!(fx_chf(&dec!(0.5), &dec!(0.01)).unwrap(), dec!(0.01)); // 0.005 → 0.01
/// ```
pub fn fx_chf(amount: &Decimal, rate: &Decimal) -> Result<Decimal, FxLineError> {
    if !amount_in_bounds(amount) {
        return Err(FxLineError::OutOfRange);
    }
    check_rate_bounds(rate).map_err(|_| FxLineError::OutOfRange)?;
    let a = scaled_int(amount, FX_AMOUNT_MAX_SCALE).ok_or(FxLineError::OutOfRange)?;
    let r = scaled_int(rate, 10).ok_or(FxLineError::OutOfRange)?;
    // Produit à l'échelle 14 ; arrondi au centime (échelle 2) : division par
    // 10¹², l'équidistant loin de zéro.
    let p = a.checked_mul(r).ok_or(FxLineError::OutOfRange)?;
    let div: i128 = 10i128.pow(12);
    let q = p / div;
    let rem = p % div;
    let cents = if rem.abs() * 2 >= div {
        q + p.signum()
    } else {
        q
    };
    let chf = Decimal::try_from_i128_with_scale(cents, 2).map_err(|_| FxLineError::OutOfRange)?;
    if chf.abs() > FX_CHF_MAX {
        return Err(FxLineError::OutOfRange);
    }
    Ok(chf)
}

/// Contrôle **une** ligne en devise (AC-3), dans l'ordre :
///
/// 1. échelle et bornes du montant, du cours ([`check_rate_bounds`], signe du
///    cours compris) et du CHF calculé ([`fx_chf`]) ;
/// 2. signe : montant > 0 — pour une ligne `revaluation`, montant = 0 ;
/// 3. CHF calculé non nul (`FX_LINE_ROUNDS_TO_ZERO`) ;
/// 4. CHF fourni (`debit + credit`) égal au CHF calculé (`FX_AMOUNT_MISMATCH`).
///
/// Une ligne `Some(Revaluation)` est exemptée des étapes 3 et 4 ; elle exige
/// un montant nul, un cours > 0 et un CHF fourni > 0. Les montants CHF négatifs
/// sont refusés **avant** cette fonction (`EntryNegativeAmount`), l'exclusivité
/// débit/crédit **après**.
///
/// Fonction publique : la route du complément d'ouverture (37-3b) l'appelle
/// avant sa transaction.
pub fn check_foreign_line(
    debit: &Decimal,
    credit: &Decimal,
    foreign: &ForeignAmount,
) -> Result<(), FxLineError> {
    let revaluation = foreign.source == Some(RateProvenance::Revaluation);
    // Étape 1 — bornes (le signe du cours est jugé par `check_rate_bounds`).
    let chf = fx_chf(&foreign.amount, &foreign.rate)?;
    let provided = debit.checked_add(*credit).ok_or(FxLineError::OutOfRange)?;
    if revaluation {
        if !foreign.amount.is_zero() || provided <= Decimal::ZERO || provided > FX_CHF_MAX {
            return Err(FxLineError::OutOfRange);
        }
        return Ok(());
    }
    // Étape 2 — signe du montant.
    if foreign.amount <= Decimal::ZERO {
        return Err(FxLineError::OutOfRange);
    }
    // Étape 3 — CHF arrondi nul.
    if chf.is_zero() {
        return Err(FxLineError::RoundsToZero);
    }
    // Étape 4 — égalité au CHF fourni (en valeur, non en graphie).
    if provided != chf {
        return Err(FxLineError::Mismatch {
            expected: chf,
            provided,
        });
    }
    Ok(())
}

/// Le cours déduit d'un CHF (V1, CR-6) : CHF ÷ montant arrondi à 10 décimales
/// (`MidpointAwayFromZero`), puis, si `round(montant × cours, 2)` ([`fx_chf`])
/// ne redonne pas le CHF, ajusté au dernier chiffre, jusqu'à
/// [`RATE_FROM_CHF_MAX_STEPS`] pas de 10⁻¹⁰ de part et d'autre (le plus proche
/// d'abord, vers le haut avant le bas à distance égale).
///
/// Bornes d'entrée d'abord : montant > 0, au plus 4 décimales, au plus 10¹³ ;
/// CHF > 0 et au plus 10¹³ ; le cours rendu passe [`check_rate_bounds`].
/// Sinon [`RateFromChfError::OutOfRange`]. Un CHF à plus de deux décimales
/// n'est pas refusé ici : aucun cours ne le redonne →
/// [`RateFromChfError::Mismatch`], `expected` = CHF du cours arrondi.
///
/// ```
/// use kesh_core::accounting::fx::rate_from_chf;
/// use rust_decimal_macros::dec;
/// assert_eq!(rate_from_chf(&dec!(100), &dec!(91.19)).unwrap(), dec!(0.9119));
/// ```
pub fn rate_from_chf(amount: &Decimal, chf: &Decimal) -> Result<Decimal, RateFromChfError> {
    if *amount <= Decimal::ZERO || !amount_in_bounds(amount) {
        return Err(RateFromChfError::OutOfRange);
    }
    if *chf <= Decimal::ZERO || *chf > FX_CHF_MAX {
        return Err(RateFromChfError::OutOfRange);
    }
    let base = chf
        .checked_div(*amount)
        .ok_or(RateFromChfError::OutOfRange)?
        .round_dp_with_strategy(10, rust_decimal::RoundingStrategy::MidpointAwayFromZero);
    check_rate_bounds(&base).map_err(|_| RateFromChfError::OutOfRange)?;
    let expected = fx_chf(amount, &base).map_err(|_| RateFromChfError::OutOfRange)?;
    if expected == *chf {
        return Ok(base.normalize());
    }
    let step = Decimal::new(1, 10);
    for k in 1..=RATE_FROM_CHF_MAX_STEPS {
        let delta = step * Decimal::from(k);
        for candidate in [base + delta, base - delta] {
            if check_rate_bounds(&candidate).is_err() {
                continue;
            }
            if let Ok(c) = fx_chf(amount, &candidate)
                && c == *chf
            {
                return Ok(candidate.normalize());
            }
        }
    }
    Err(RateFromChfError::Mismatch { expected })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn fa(amount: Decimal, rate: Decimal, source: RateProvenance) -> ForeignAmount {
        ForeignAmount {
            declared_currency: None,
            amount,
            rate,
            source: Some(source),
            rate_derived: false,
        }
    }

    #[test]
    fn constante_de_borne_vaut_dix_puissance_treize() {
        assert_eq!(FX_AMOUNT_MAX, dec!(10000000000000));
    }

    #[test]
    fn fx_chf_arrondit_l_equidistant_loin_de_zero() {
        assert_eq!(fx_chf(&dec!(1), &dec!(0.125)).unwrap(), dec!(0.13));
        assert_eq!(fx_chf(&dec!(1), &dec!(0.124)).unwrap(), dec!(0.12));
        assert_eq!(fx_chf(&dec!(-1), &dec!(0.125)).unwrap(), dec!(-0.13));
    }

    #[test]
    fn fx_chf_refuse_les_bornes() {
        assert_eq!(
            fx_chf(&dec!(1.00001), &dec!(1)),
            Err(FxLineError::OutOfRange)
        );
        assert_eq!(
            fx_chf(&dec!(10000000000000.0001), &dec!(1)),
            Err(FxLineError::OutOfRange)
        );
        assert_eq!(fx_chf(&dec!(1), &dec!(0)), Err(FxLineError::OutOfRange));
        assert_eq!(
            fx_chf(&dec!(1), &dec!(1000001)),
            Err(FxLineError::OutOfRange)
        );
        // CHF > 10¹³ par le produit.
        assert_eq!(
            fx_chf(&dec!(10000000000000), &dec!(1.0000000001)),
            Err(FxLineError::OutOfRange)
        );
        // Zéros terminaux ignorés.
        assert!(fx_chf(&dec!(1.50000), &dec!(0.9100000000000)).is_ok());
    }

    #[test]
    fn ligne_dans_l_ordre_des_refus() {
        let ok = fa(dec!(100), dec!(0.91193), RateProvenance::Afc);
        assert_eq!(check_foreign_line(&dec!(91.19), &dec!(0), &ok), Ok(()));
        // Étape 4.
        assert_eq!(
            check_foreign_line(&dec!(91.20), &dec!(0), &ok),
            Err(FxLineError::Mismatch {
                expected: dec!(91.19),
                provided: dec!(91.20)
            })
        );
        // Étape 2 : montant nul hors réévaluation.
        let zero = fa(dec!(0), dec!(1), RateProvenance::Manual);
        assert_eq!(
            check_foreign_line(&dec!(1), &dec!(0), &zero),
            Err(FxLineError::OutOfRange)
        );
        // Étape 3.
        let tiny = fa(dec!(0.0001), dec!(0.01), RateProvenance::Manual);
        assert_eq!(
            check_foreign_line(&dec!(0), &dec!(0), &tiny),
            Err(FxLineError::RoundsToZero)
        );
        // Étape 1 avant tout : cours négatif.
        let neg = fa(dec!(100), dec!(-1), RateProvenance::Manual);
        assert_eq!(
            check_foreign_line(&dec!(1), &dec!(0), &neg),
            Err(FxLineError::OutOfRange)
        );
    }

    #[test]
    fn reevaluation_exemptee_mais_bornee() {
        let r = fa(dec!(0), dec!(0.9), RateProvenance::Revaluation);
        assert_eq!(check_foreign_line(&dec!(12.34), &dec!(0), &r), Ok(()));
        assert_eq!(
            check_foreign_line(&dec!(0), &dec!(0), &r),
            Err(FxLineError::OutOfRange)
        );
        let r_montant = fa(dec!(1), dec!(0.9), RateProvenance::Revaluation);
        assert_eq!(
            check_foreign_line(&dec!(12.34), &dec!(0), &r_montant),
            Err(FxLineError::OutOfRange)
        );
        let r_cours = fa(dec!(0), dec!(0), RateProvenance::Revaluation);
        assert_eq!(
            check_foreign_line(&dec!(12.34), &dec!(0), &r_cours),
            Err(FxLineError::OutOfRange)
        );
    }

    #[test]
    fn cours_deduit_simple_et_ajuste() {
        assert_eq!(
            rate_from_chf(&dec!(100), &dec!(91.19)).unwrap(),
            dec!(0.9119)
        );
        assert_eq!(
            rate_from_chf(&dec!(1234.56), &dec!(1125.83)).unwrap(),
            dec!(0.9119281363)
        );
        assert_eq!(
            rate_from_chf(&dec!(0), &dec!(1)),
            Err(RateFromChfError::OutOfRange)
        );
        assert_eq!(
            rate_from_chf(&dec!(1), &dec!(0)),
            Err(RateFromChfError::OutOfRange)
        );
        assert!(matches!(
            rate_from_chf(&dec!(1), &dec!(0.123)),
            Err(RateFromChfError::Mismatch { .. })
        ));
    }

    #[test]
    fn provenance_aller_retour() {
        use std::str::FromStr;
        for p in [
            RateProvenance::Afc,
            RateProvenance::Manual,
            RateProvenance::Revaluation,
        ] {
            assert_eq!(RateProvenance::from_str(p.as_str()), Ok(p));
        }
        assert!(RateProvenance::from_str("AFC").is_err());
    }
}
