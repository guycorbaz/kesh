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
use std::collections::BTreeMap;

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

/// Une ligne du récapitulatif de TVA d'une facture, agrégée par taux (#151).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VatRateBreakdown {
    /// Taux en **pourcent** (ex. `dec!(8.10)` pour 8.1 %).
    pub rate_percent: Decimal,
    /// Somme des bases HT des lignes à ce taux.
    pub base_ht: Decimal,
    /// Somme des **TVA de ligne arrondies** (DC7) des lignes à ce taux.
    pub vat_amount: Decimal,
}

/// Ventile la TVA d'une facture **par taux**, pour le récapitulatif du document
/// (#151 — obligation LTVA art. 26 d'afficher le montant de TVA par taux).
///
/// Regroupe les lignes par `vat_rate`, somme les bases HT et les **TVA arrondies
/// par ligne** (DC7 — cf. [`line_vat_amount`] ; ne JAMAIS arrondir une base
/// agrégée). La cohérence avec [`invoice_total_ttc`] est garantie : la somme des
/// `base_ht + vat_amount` de tous les taux, plus les lignes à 0 %, égale le TTC.
///
/// N'inclut que les taux **strictement positifs** : une ligne à 0 % (exonérée /
/// exclue) compte dans le sous-total HT mais ne produit pas de ligne de TVA.
/// Résultat trié par taux **décroissant** (convention suisse : taux normal avant
/// taux réduit / hébergement).
pub fn vat_breakdown_by_rate<I>(lines: I) -> Vec<VatRateBreakdown>
where
    I: IntoIterator<Item = (Decimal, Decimal)>,
{
    // BTreeMap : clé `Decimal` triée par valeur (8.10 == 8.1 → même taux), agrège
    // (base_ht, vat_amount) par taux. `line_total`/`vat_rate` viennent du même
    // schéma DECIMAL(_,2), donc pas d'ambiguïté d'échelle sur la clé.
    let mut by_rate: BTreeMap<Decimal, (Decimal, Decimal)> = BTreeMap::new();
    for (line_total, vat_rate) in lines {
        if vat_rate <= Decimal::ZERO {
            continue;
        }
        let entry = by_rate
            .entry(vat_rate)
            .or_insert((Decimal::ZERO, Decimal::ZERO));
        entry.0 += line_total;
        entry.1 += line_vat_amount(line_total, vat_rate);
    }
    by_rate
        .into_iter()
        .rev() // taux décroissant
        .map(|(rate_percent, (base_ht, vat_amount))| VatRateBreakdown {
            // Normalisé à 2 décimales : la clé BTreeMap conserve le scale de la
            // 1re ligne insérée (8.1 vs 8.10 sont égaux par `Ord` mais diffèrent
            // de scale) — on fige "X.YZ" pour un affichage/sérialisation stable.
            rate_percent: rate_percent.round_dp(2),
            base_ht,
            vat_amount,
        })
        .collect()
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
    fn ttc_negative_base_symmetric() {
        // Propriété générique du helper : robustesse sur base négative
        // (arrondi symétrique). NB : les avoirs ne passent PAS par ce chemin
        // avec des montants négatifs — `credit_note_lines.line_total` porte un
        // CHECK `>= 0` et la contre-passation gère le signe séparément. Test
        // de robustesse de la fonction, pas un scénario avoir réel.
        assert_eq!(
            invoice_total_ttc([(dec!(-100.00), dec!(8.1))]),
            dec!(-108.10)
        );
    }

    #[test]
    fn ttc_empty_is_zero() {
        assert_eq!(invoice_total_ttc([]), Decimal::ZERO);
    }

    // --- vat_breakdown_by_rate (récap TVA, #151) ---

    #[test]
    fn breakdown_single_rate_sums_base_and_vat() {
        let b = vat_breakdown_by_rate([(dec!(100.00), dec!(8.1)), (dec!(50.00), dec!(8.1))]);
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].rate_percent, dec!(8.1));
        assert_eq!(b[0].base_ht, dec!(150.00));
        assert_eq!(b[0].vat_amount, dec!(12.15)); // 8.10 + 4.05
    }

    #[test]
    fn breakdown_rounds_per_line_not_on_aggregate() {
        // Deux lignes 0.05 @ 8.1 % : TVA par ligne = round(0.00405) = 0.00 chacune
        // → 0.00 au total (et NON round(0.10 × 8.1 %) = 0.01). Interdit DC7.
        let b = vat_breakdown_by_rate([(dec!(0.05), dec!(8.1)), (dec!(0.05), dec!(8.1))]);
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].base_ht, dec!(0.10));
        assert_eq!(b[0].vat_amount, dec!(0.00));
    }

    #[test]
    fn breakdown_multi_rates_sorted_descending() {
        let b = vat_breakdown_by_rate([
            (dec!(50.00), dec!(2.6)),
            (dec!(100.00), dec!(8.1)),
            (dec!(200.00), dec!(3.8)),
        ]);
        // Taux décroissant : 8.1, 3.8, 2.6.
        assert_eq!(
            b.iter().map(|r| r.rate_percent).collect::<Vec<_>>(),
            vec![dec!(8.1), dec!(3.8), dec!(2.6)]
        );
    }

    #[test]
    fn breakdown_excludes_zero_rate_lines() {
        // Une ligne exonérée (0 %) compte au HT mais ne crée pas de ligne TVA.
        let b = vat_breakdown_by_rate([(dec!(1000.00), dec!(0)), (dec!(100.00), dec!(8.1))]);
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].rate_percent, dec!(8.1));
    }

    #[test]
    fn breakdown_reconciles_with_total_ttc() {
        // Σ(base_ht + vat_amount) des taux + Σ lignes 0 % == invoice_total_ttc.
        let lines = [
            (dec!(100.00), dec!(8.1)),
            (dec!(12345.5), dec!(1.0)),
            (dec!(50.00), dec!(2.6)),
            (dec!(10.00), dec!(0)), // exonérée
        ];
        let b = vat_breakdown_by_rate(lines);
        let taxed: Decimal = b.iter().map(|r| r.base_ht + r.vat_amount).sum();
        let exempt_ht: Decimal = lines
            .iter()
            .filter(|(_, r)| *r <= Decimal::ZERO)
            .map(|(ht, _)| *ht)
            .sum();
        assert_eq!(taxed + exempt_ht, invoice_total_ttc(lines));
    }

    #[test]
    fn breakdown_empty_is_empty() {
        assert!(vat_breakdown_by_rate([]).is_empty());
        // Une facture 100 % exonérée n'a aucun récap TVA.
        assert!(vat_breakdown_by_rate([(dec!(500.00), dec!(0))]).is_empty());
    }
}
