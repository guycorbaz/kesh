//! Calcul de la TVA par ligne (FR55).
//!
//! Conformément aux règles de l'AFC (Administration fédérale des
//! contributions), la TVA est arrondie au centime **par ligne** (arrondi
//! commercial, demi vers le haut), puis les montants arrondis sont sommés.
//! NE PAS sommer les bases puis arrondir une seule fois (le résultat
//! diffère et n'est pas conforme).
//!
//! Toute l'arithmétique est en [`rust_decimal::Decimal`] — jamais de `f64`.

use crate::types::Money;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Calcule le montant de TVA d'une ligne à partir de sa base HT et de son
/// taux, arrondi au centime en **arrondi commercial** (`MidpointAwayFromZero`,
/// demi vers le haut) via [`Money::round_to_centimes`].
///
/// # Unité du taux
///
/// `rate_percent` est exprimé en **pourcent** (ex. `8.1` pour 8.1 %, comme la
/// colonne `invoice_lines.vat_rate DECIMAL(5,2)` qui stocke `8.10`), **pas** en
/// décimal (`0.081`). La division par 100 en dépend.
///
/// # Arrondi par ligne (FR55)
///
/// Cette fonction réalise l'arrondi **d'une seule ligne**. Pour un total par
/// taux, appeler cette fonction sur chaque ligne puis sommer les résultats
/// arrondis — ne pas arrondir une base agrégée.
///
/// # Exemples
///
/// ```
/// use rust_decimal_macros::dec;
/// use kesh_core::accounting::vat::line_vat_amount;
///
/// assert_eq!(line_vat_amount(dec!(100), dec!(8.1)), dec!(8.10));
/// ```
pub fn line_vat_amount(base_ht: Decimal, rate_percent: Decimal) -> Decimal {
    Money::new(base_ht * rate_percent / dec!(100))
        .round_to_centimes()
        .amount()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_rate() {
        // 1000.00 HT à 8.1 % = 81.00
        assert_eq!(line_vat_amount(dec!(1000), dec!(8.1)), dec!(81.00));
    }

    #[test]
    fn unit_is_percent_not_decimal() {
        // Le taux est en pourcent : 100 × 8.1 / 100 = 8.10 (et NON 0.081 × 100).
        assert_eq!(line_vat_amount(dec!(100), dec!(8.1)), dec!(8.10));
    }

    #[test]
    fn commercial_rounding_half_up() {
        // 12345.5 × 1.0 / 100 = 123.455 → 123.46 (demi vers le haut).
        assert_eq!(line_vat_amount(dec!(12345.5), dec!(1.0)), dec!(123.46));
    }

    #[test]
    fn zero_base() {
        assert_eq!(line_vat_amount(dec!(0), dec!(8.1)), dec!(0.00));
    }

    #[test]
    fn zero_rate_exempt() {
        // Taux 0 % (exonéré) → TVA nulle, quelle que soit la base.
        assert_eq!(line_vat_amount(dec!(1000), dec!(0)), dec!(0.00));
    }

    #[test]
    fn negative_base_credit_note() {
        // Avoir / contre-passation : base négative, arrondi symétrique.
        assert_eq!(line_vat_amount(dec!(-100), dec!(8.1)), dec!(-8.10));
        assert_eq!(line_vat_amount(dec!(-12345.5), dec!(1.0)), dec!(-123.46));
    }
}
