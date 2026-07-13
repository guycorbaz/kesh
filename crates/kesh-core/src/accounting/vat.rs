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

/// Total TTC canonique d'une facture (#246, Story 21-2a) : Σ par ligne de
/// `line_total + line_vat_amount(line_total, vat_rate)`.
///
/// # Équivalence avec le débit créance comptable
///
/// L'écriture de validation (`generate_invoice_journal_lines`) calcule
/// `total_ht + Σ_taux (Σ_lignes vat_arrondie)` — l'agrégation intermédiaire
/// par taux est **associative**, donc `Σ_lignes (ht + vat_arrondie)` donne
/// exactement le même TTC. Ce helper est LA définition du montant dû ; le
/// QR-bill, le PDF, `{amount}` des e-mails et l'échéancier le consomment,
/// et l'expression SQL miroir de `kesh-db` lui est asservie par un test de
/// parité.
///
/// # Interdit DC7
///
/// Ne JAMAIS arrondir une base agrégée (`round(Σ base × taux)`) : la TVA est
/// arrondie **par ligne** puis sommée (règle AFC, cf. module).
pub fn invoice_total_ttc<I>(lines: I) -> Decimal
where
    I: IntoIterator<Item = (Decimal, Decimal)>,
{
    lines
        .into_iter()
        .fold(Decimal::ZERO, |acc, (line_total, vat_rate)| {
            acc + line_total + line_vat_amount(line_total, vat_rate)
        })
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

    // --- invoice_total_ttc (Story 21-2a, #246) ---

    #[test]
    fn ttc_single_line() {
        // 100.00 @ 8.1 % → 108.10.
        assert_eq!(invoice_total_ttc([(dec!(100.00), dec!(8.1))]), dec!(108.10));
    }

    #[test]
    fn ttc_rounds_per_line_not_on_aggregate() {
        // Deux lignes 0.05 @ 8.1 % : TVA par ligne = round(0.00405) = 0.00,
        // TTC = 0.10. Un arrondi sur la base agrégée (0.10 × 8.1 % = 0.01)
        // donnerait 0.11 — l'interdit DC7 est précisément là.
        assert_eq!(
            invoice_total_ttc([(dec!(0.05), dec!(8.1)), (dec!(0.05), dec!(8.1))]),
            dec!(0.10)
        );
    }

    #[test]
    fn ttc_multi_lines_mixed_rates_matches_journal_computation() {
        // Parité avec le calcul du journal (agrégation par taux) : même
        // résultat par associativité.
        let lines = [
            (dec!(100.00), dec!(8.1)),
            (dec!(12345.5), dec!(1.0)), // arrondi half-away 123.455 → 123.46
            (dec!(50.00), dec!(2.6)),
            (dec!(10.00), dec!(0)),
        ];
        // Voie journal : total_ht + Σ_taux (Σ_lignes vat arrondie par ligne).
        let total_ht: Decimal = lines.iter().map(|(ht, _)| *ht).sum();
        let mut vat_by_rate = std::collections::BTreeMap::new();
        for (ht, rate) in lines {
            *vat_by_rate.entry(rate).or_insert(Decimal::ZERO) += line_vat_amount(ht, rate);
        }
        let total_vat: Decimal = vat_by_rate.values().copied().sum();
        assert_eq!(invoice_total_ttc(lines), total_ht + total_vat);
    }

    #[test]
    fn ttc_zero_rate_equals_ht() {
        assert_eq!(invoice_total_ttc([(dec!(1234.56), dec!(0))]), dec!(1234.56));
    }

    #[test]
    fn ttc_negative_lines_credit_note() {
        // Avoir : lignes négatives, symétrique.
        assert_eq!(
            invoice_total_ttc([(dec!(-100.00), dec!(8.1))]),
            dec!(-108.10)
        );
    }

    #[test]
    fn ttc_empty_is_zero() {
        assert_eq!(invoice_total_ttc([]), Decimal::ZERO);
    }
}
