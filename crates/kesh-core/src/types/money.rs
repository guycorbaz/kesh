//! Type monétaire avec arithmétique décimale exacte.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::iter::Sum;
use std::ops::{Add, Mul, Neg, Sub};
use std::str::FromStr;

use rust_decimal::prelude::*;

/// Montant monétaire avec arithmétique décimale exacte.
///
/// Utilise `rust_decimal::Decimal` en interne — jamais de `f64`.
/// Les montants négatifs sont valides (avoirs, contre-passations).
///
/// # Attention — Overflow
///
/// Les opérations arithmétiques (`Add`, `Sub`, `Mul`, `Sum`) **paniquent**
/// en cas de dépassement de `Decimal::MAX` (~7.92 × 10²⁸). Cela ne peut pas
/// arriver avec des montants comptables réalistes, mais les données en
/// provenance de sources externes (CAMT.053, CSV) doivent être validées
/// à la frontière avant d'être additionnées en volume.
///
/// # Exemples
///
/// ```
/// use rust_decimal_macros::dec;
/// use kesh_core::types::Money;
///
/// let price = Money::new(dec!(19.95));
/// let qty = dec!(3);
/// let total = price * qty;
/// assert_eq!(total.amount(), dec!(59.85));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Money(Decimal);

impl Money {
    /// Crée un nouveau montant. Tout `Decimal` est un montant valide.
    pub fn new(amount: Decimal) -> Self {
        Self(amount)
    }

    /// Retourne la valeur décimale interne.
    pub fn amount(&self) -> Decimal {
        self.0
    }

    /// Montant zéro.
    pub fn zero() -> Self {
        Self(Decimal::ZERO)
    }

    /// Vérifie si le montant est négatif.
    pub fn is_negative(&self) -> bool {
        self.0.is_sign_negative() && self.0 != Decimal::ZERO
    }

    /// Arrondi commercial au centime (2 décimales, MidpointAwayFromZero).
    ///
    /// Utilisé pour les montants CHF conformément aux règles de l'AFC
    /// (calcul TVA par ligne). À distinguer de l'arrondi cash aux 5 centimes
    /// (rappen) qui sera fourni séparément si nécessaire.
    pub fn round_to_centimes(&self) -> Self {
        Self(
            self.0
                .round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero),
        )
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Money {
    type Err = rust_decimal::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Decimal::from_str(s)?))
    }
}

impl Add for Money {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Money {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl Neg for Money {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl Mul<Decimal> for Money {
    type Output = Self;

    fn mul(self, rhs: Decimal) -> Self {
        Self(self.0 * rhs)
    }
}

impl Mul<Money> for Decimal {
    type Output = Money;

    fn mul(self, rhs: Money) -> Money {
        Money(self * rhs.0)
    }
}

impl Sum for Money {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, m| acc + m)
    }
}

impl<'a> Sum<&'a Money> for Money {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, m| acc + *m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn new_and_amount() {
        let m = Money::new(dec!(100.50));
        assert_eq!(m.amount(), dec!(100.50));
    }

    #[test]
    fn zero() {
        assert_eq!(Money::zero().amount(), Decimal::ZERO);
    }

    #[test]
    fn is_negative() {
        assert!(!Money::new(dec!(10)).is_negative());
        assert!(Money::new(dec!(-10)).is_negative());
        assert!(!Money::zero().is_negative());
    }

    #[test]
    fn addition() {
        let a = Money::new(dec!(10.50));
        let b = Money::new(dec!(20.30));
        assert_eq!((a + b).amount(), dec!(30.80));
    }

    #[test]
    fn subtraction() {
        let a = Money::new(dec!(100));
        let b = Money::new(dec!(30.50));
        assert_eq!((a - b).amount(), dec!(69.50));
    }

    #[test]
    fn negation() {
        let m = Money::new(dec!(42));
        assert_eq!((-m).amount(), dec!(-42));
    }

    #[test]
    fn multiply_by_decimal() {
        let price = Money::new(dec!(100));
        let qty = dec!(3);
        assert_eq!((price * qty).amount(), dec!(300));
    }

    #[test]
    fn decimal_multiply_money() {
        let rate = dec!(0.081);
        let amount = Money::new(dec!(1000));
        assert_eq!((rate * amount).amount(), dec!(81));
    }

    #[test]
    fn sum_iterator() {
        let amounts = vec![
            Money::new(dec!(10)),
            Money::new(dec!(20)),
            Money::new(dec!(30)),
        ];
        let total: Money = amounts.into_iter().sum();
        assert_eq!(total.amount(), dec!(60));
    }

    #[test]
    fn sum_ref_iterator() {
        let amounts = [
            Money::new(dec!(10)),
            Money::new(dec!(20)),
            Money::new(dec!(30)),
        ];
        let total: Money = amounts.iter().sum();
        assert_eq!(total.amount(), dec!(60));
    }

    #[test]
    fn round_to_centimes_standard() {
        let m = Money::new(dec!(19.954));
        assert_eq!(m.round_to_centimes().amount(), dec!(19.95));
    }

    #[test]
    fn round_to_centimes_midpoint_away_from_zero() {
        let m = Money::new(dec!(19.955));
        assert_eq!(m.round_to_centimes().amount(), dec!(19.96));
    }

    #[test]
    fn round_to_centimes_negative() {
        let m = Money::new(dec!(-19.955));
        assert_eq!(m.round_to_centimes().amount(), dec!(-19.96));
    }

    #[test]
    fn display() {
        let m = Money::new(dec!(1234.56));
        assert_eq!(m.to_string(), "1234.56");
    }

    #[test]
    fn from_str() {
        let m: Money = "1234.56".parse().unwrap();
        assert_eq!(m.amount(), dec!(1234.56));
    }

    #[test]
    fn serde_roundtrip() {
        let m = Money::new(dec!(1234.56));
        let json = serde_json::to_string(&m).unwrap();
        assert_eq!(json, r#""1234.56""#);
        let deserialized: Money = serde_json::from_str(&json).unwrap();
        assert_eq!(m, deserialized);
    }

    #[test]
    fn serde_negative() {
        let m = Money::new(dec!(-1234.56));
        let json = serde_json::to_string(&m).unwrap();
        assert_eq!(json, r#""-1234.56""#);
        let deserialized: Money = serde_json::from_str(&json).unwrap();
        assert_eq!(m, deserialized);
    }

    #[test]
    fn copy_semantics() {
        let a = Money::new(dec!(100));
        let b = a; // Copy
        assert_eq!(a, b); // a still usable
    }

    #[test]
    fn ordering() {
        let small = Money::new(dec!(10));
        let large = Money::new(dec!(100));
        assert!(small < large);
    }
}
