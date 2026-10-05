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

/// L'écart d'arrondi à 5 centimes du total d'une facture émise
/// (Story 25-4-c4-a, #494) : `arrondi(ttc_brut) − ttc_brut`, ou `0` si le réglage
/// de la société est inactif. Positif quand le total monte (123.44 → +0.01),
/// négatif quand il descend (234.52 → −0.02).
///
/// ⛔ Calculé **une fois**, à la validation, puis figé sur la pièce
/// (`invoices.rounding_amount`) : le recalculer d'après le réglage courant ferait
/// diverger le reste dû et la QR de l'écriture déjà passée.
pub fn invoice_rounding(ttc_brut: Decimal, round_to_5_centimes: bool) -> Decimal {
    if !round_to_5_centimes {
        return Decimal::ZERO;
    }
    Money::new(ttc_brut).round_to_5_centimes().amount() - ttc_brut
}

/// Le TTC d'une pièce, **arrondi figé compris** : [`invoice_total_ttc`] des lignes
/// plus l'écart d'arrondi stocké sur la pièce (`rounding_amount`, 0 pour les
/// pièces émises sans arrondi). Toute lecture du total d'une facture ou d'un
/// avoir **émis** passe par ici.
pub fn invoice_total_ttc_rounded<I>(lines: I, rounding_amount: Decimal) -> Decimal
where
    I: IntoIterator<Item = (Decimal, Decimal)>,
{
    invoice_total_ttc(lines) + rounding_amount
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

/// La part de TVA corrigée, pour un taux, d'un **solde** (Story 25-4-d2a, #384).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VatRateShare {
    /// Taux en **pourcent** (ex. `dec!(8.10)`).
    pub rate_percent: Decimal,
    /// Part de la base HT soldée — **informative** (rapport TVA, Story 25-4-d2c) :
    /// l'écriture n'en dépend pas.
    pub base_ht: Decimal,
    /// TVA corrigée à ce taux, au centime.
    pub vat_amount: Decimal,
}

/// Ventile, au prorata des taux de la facture, la TVA que corrige le **solde**
/// d'un montant `amount` (escompte accordé, perte sur débiteur — Story 25-4-d2a).
///
/// Une diminution de contre-prestation (LTVA art. 41) corrige la TVA dans la
/// proportion où elle réduit le prix : pour chaque taux > 0 de
/// [`vat_breakdown_by_rate`], `vat_amount = amount × TVA du taux / total_ttc` et
/// `base_ht = amount × base du taux / total_ttc`, chacun arrondi au centime
/// ([`Money::round_to_centimes`]). `total_ttc` est le TTC **figé** de la facture
/// ([`invoice_total_ttc_rounded`], arrondi à 5 centimes compris : cette part, sans
/// TVA, n'en porte pas). La formule vaut aussi pour un reste après règlement
/// partiel : le reste suit la composition du TTC.
///
/// Les parts dont la TVA arrondie est nulle sont omises. Le reliquat de centimes
/// n'est réparti nulle part : l'appelant impute `amount − Σ vat_amount` au compte
/// de la nature, ce qui garde l'écriture équilibrée.
///
/// `total_ttc <= 0` ou `amount <= 0` → aucune part.
pub fn write_off_vat_shares<I>(lines: I, total_ttc: Decimal, amount: Decimal) -> Vec<VatRateShare>
where
    I: IntoIterator<Item = (Decimal, Decimal)>,
{
    if total_ttc <= Decimal::ZERO || amount <= Decimal::ZERO {
        return Vec::new();
    }
    vat_breakdown_by_rate(lines)
        .into_iter()
        .filter_map(|rate| {
            let vat_amount = Money::new(amount * rate.vat_amount / total_ttc)
                .round_to_centimes()
                .amount();
            if vat_amount.is_zero() {
                return None;
            }
            Some(VatRateShare {
                rate_percent: rate.rate_percent,
                base_ht: Money::new(amount * rate.base_ht / total_ttc)
                    .round_to_centimes()
                    .amount(),
                vat_amount,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn invoice_rounding_examples_and_switch() {
        use rust_decimal_macros::dec;
        assert_eq!(invoice_rounding(dec!(123.44), true), dec!(0.01));
        assert_eq!(invoice_rounding(dec!(234.52), true), dec!(-0.02));
        assert_eq!(invoice_rounding(dec!(10.0050), true), dec!(-0.0050));
        assert_eq!(invoice_rounding(dec!(108.10), true), dec!(0));
        assert_eq!(invoice_rounding(dec!(123.44), false), dec!(0));
        assert_eq!(
            invoice_total_ttc_rounded([(dec!(114.20), dec!(8.1))], dec!(0.01)),
            invoice_total_ttc([(dec!(114.20), dec!(8.1))]) + dec!(0.01)
        );
    }

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

    // --- Story 25-4-d2a : le prorata de TVA d'un solde ---

    #[test]
    fn write_off_vat_shares_single_rate() {
        // 1000.00 HT à 8.1 % → TTC 1081.00 ; un escompte de 21.62 (2 %) corrige 1.62.
        let lines = vec![(dec!(1000.00), dec!(8.10))];
        let shares = write_off_vat_shares(lines, dec!(1081.00), dec!(21.62));
        assert_eq!(
            shares,
            vec![VatRateShare {
                rate_percent: dec!(8.10),
                base_ht: dec!(20.00),
                vat_amount: dec!(1.62),
            }]
        );
    }

    #[test]
    fn write_off_vat_shares_whole_ttc_returns_the_invoiced_vat() {
        // Solder tout le TTC corrige exactement la TVA facturée, taux par taux.
        let lines = vec![
            (dec!(100.00), dec!(8.10)),
            (dec!(50.00), dec!(2.60)),
            (dec!(30.00), dec!(0)),
        ];
        let ttc = invoice_total_ttc(lines.clone());
        let shares = write_off_vat_shares(lines.clone(), ttc, ttc);
        let breakdown = vat_breakdown_by_rate(lines);
        assert_eq!(shares.len(), 2, "le taux à 0 % ne porte aucune part");
        for (share, rate) in shares.iter().zip(breakdown.iter()) {
            assert_eq!(share.rate_percent, rate.rate_percent);
            assert_eq!(share.vat_amount, rate.vat_amount);
            assert_eq!(share.base_ht, rate.base_ht);
        }
    }

    #[test]
    fn write_off_vat_shares_several_rates_on_a_partial_remainder() {
        // TTC 100×1.081 + 100×1.026 = 210.70 ; reste 21.07 (10 %) → 0.81 et 0.26.
        let lines = vec![(dec!(100.00), dec!(8.10)), (dec!(100.00), dec!(2.60))];
        let shares = write_off_vat_shares(lines, dec!(210.70), dec!(21.07));
        let vats: Vec<_> = shares
            .iter()
            .map(|s| (s.rate_percent, s.vat_amount))
            .collect();
        assert_eq!(
            vats,
            vec![(dec!(8.10), dec!(0.81)), (dec!(2.60), dec!(0.26))]
        );
    }

    #[test]
    fn write_off_vat_shares_rounding_part_carries_no_vat() {
        // 10.00 HT à 8.1 % → 10.81, arrondi figé −0.01 → TTC 10.80 : solder ce TTC
        // corrige la TVA des lignes (0.81), l'arrondi n'en portant aucune.
        let lines = vec![(dec!(10.00), dec!(8.10))];
        let ttc_rounded = invoice_total_ttc_rounded(lines.clone(), dec!(-0.01)); // 10.81 → 10.80
        let shares = write_off_vat_shares(lines, ttc_rounded, ttc_rounded);
        assert_eq!(shares[0].vat_amount, dec!(0.81));
    }

    #[test]
    fn write_off_vat_shares_four_decimal_amount_and_tiny_amounts() {
        let lines = vec![(dec!(1000.00), dec!(8.10))];
        // Montant à quatre décimales : la TVA reste au centime.
        let shares = write_off_vat_shares(lines.clone(), dec!(1081.00), dec!(10.0050));
        assert_eq!(shares[0].vat_amount, dec!(0.75));
        // Un montant minuscule : part de TVA nulle → omise.
        assert!(write_off_vat_shares(lines.clone(), dec!(1081.00), dec!(0.0040)).is_empty());
        // Montant ou TTC non positif : aucune part.
        assert!(write_off_vat_shares(lines.clone(), dec!(1081.00), dec!(0)).is_empty());
        assert!(write_off_vat_shares(lines, dec!(0), dec!(10)).is_empty());
    }
}
