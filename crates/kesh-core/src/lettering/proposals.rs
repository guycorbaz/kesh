//! Le moteur **pur** des propositions de lettrage (Story 15-1b, #518, AC5).
//!
//! Entrée : les lignes **candidates** d'un compte — ouvertes aujourd'hui,
//! lettrables à la main (R5), au plus [`MAX_PROPOSAL_CANDIDATES`] —, chacune
//! avec son écriture, l'écriture que celle-ci contre-passe, sa date, ses
//! montants et son état de période (R7). Sortie : des **paires** débit/crédit
//! de montants égaux, classées, chaque ligne n'apparaissant que dans **une**.
//!
//! ⛔ Kesh **n'écrit rien** : une proposition s'accepte par `POST
//! /api/v1/letterings`, une à une.
//!
//! # Règles (AC5)
//!
//! - **sens opposés, montants égaux** : une ligne au débit seul, une au crédit
//!   seul, `debit == credit` — **aucune tolérance** (un règlement amputé de
//!   frais n'est pas une paire) ;
//! - **acceptable** : au moins une des deux lignes est en période ouverte (R7) —
//!   une paire tout entière en période close serait refusée au clic, elle n'est
//!   pas proposée. ⛔ Ce filtre s'applique **AVANT** l'appariement glouton : une
//!   ligne dont la meilleure paire est inacceptable garde sa meilleure paire
//!   acceptable ;
//! - **classement** : paire contre-passation/origine d'abord (l'écriture de
//!   l'une a pour `reverses_entry_id` celle de l'autre), puis écart de dates
//!   croissant, puis `line_id` du débit, puis `line_id` du crédit — un départage
//!   total, donc **stable** ;
//! - **appariement glouton** dans l'ordre du classement : une paire est retenue
//!   si aucune de ses deux lignes ne l'est déjà.
//!
//! **Coût** : groupement par montant, puis toutes les paires débit × crédit de
//! chaque groupe — `O(Σ n_k·m_k)`, borné par le plafond des candidates (au pire
//! 1 000 × 1 000 d'un même montant, 10⁶ paires). Aucune auto-jointure SQL.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use rust_decimal::Decimal;

/// Plafond des lignes **candidates** (après les filtres « ouverte » et R5) au-delà
/// duquel la couche de persistance refuse de proposer (422) — jamais une
/// troncature muette.
pub const MAX_PROPOSAL_CANDIDATES: usize = 2_000;

/// Une ligne candidate, telle que le moteur la juge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub line_id: i64,
    pub entry_id: i64,
    /// L'écriture que l'écriture de cette ligne contre-passe, s'il y en a une.
    pub reverses_entry_id: Option<i64>,
    pub date: NaiveDate,
    pub debit: Decimal,
    pub credit: Decimal,
    /// La ligne est-elle « en période ouverte » (R7) ?
    pub in_open_period: bool,
}

/// Une paire proposée : les **indices** de ses deux lignes dans la tranche
/// d'entrée, le montant, l'écart de dates et le drapeau contre-passation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pair {
    pub debit: usize,
    pub credit: usize,
    pub amount: Decimal,
    /// `|date du débit − date du crédit|`, en jours.
    pub days_apart: i64,
    /// L'écriture de l'une contre-passe celle de l'autre.
    pub reversal_pair: bool,
}

/// Les paires retenues, **dans l'ordre du classement**. Une ligne qui n'est
/// ni au débit seul ni au crédit seul (montants nuls, ou les deux non nuls —
/// données hors contrainte) n'est jamais appariée.
pub fn propose_pairs(candidates: &[Candidate]) -> Vec<Pair> {
    // Groupement par montant : débits et crédits, par indice.
    let mut groupes: BTreeMap<Decimal, (Vec<usize>, Vec<usize>)> = BTreeMap::new();
    for (i, c) in candidates.iter().enumerate() {
        let zero = Decimal::ZERO;
        if c.debit > zero && c.credit.is_zero() {
            groupes.entry(c.debit).or_default().0.push(i);
        } else if c.credit > zero && c.debit.is_zero() {
            groupes.entry(c.credit).or_default().1.push(i);
        }
    }

    // Toutes les paires ACCEPTABLES (R7 filtrée avant le glouton), avec leur
    // clé de classement.
    let mut paires: Vec<((bool, i64, i64, i64), Pair)> = Vec::new();
    for (montant, (debits, credits)) in &groupes {
        for &d in debits {
            for &c in credits {
                let (ld, lc) = (&candidates[d], &candidates[c]);
                if !(ld.in_open_period || lc.in_open_period) {
                    continue;
                }
                let reversal_pair = ld.reverses_entry_id == Some(lc.entry_id)
                    || lc.reverses_entry_id == Some(ld.entry_id);
                let days_apart = (ld.date - lc.date).num_days().abs();
                // `!reversal_pair` : `false` (contre-passation) trie en tête.
                let cle = (!reversal_pair, days_apart, ld.line_id, lc.line_id);
                paires.push((
                    cle,
                    Pair {
                        debit: d,
                        credit: c,
                        amount: *montant,
                        days_apart,
                        reversal_pair,
                    },
                ));
            }
        }
    }
    paires.sort_unstable_by_key(|(cle, _)| *cle);

    // Glouton : chaque ligne dans sa meilleure paire seulement.
    let mut prise = vec![false; candidates.len()];
    let mut retenues = Vec::new();
    for (_, p) in paires {
        if !prise[p.debit] && !prise[p.credit] {
            prise[p.debit] = true;
            prise[p.credit] = true;
            retenues.push(p);
        }
    }
    retenues
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 3, day).unwrap()
    }

    fn ligne(line_id: i64, montant: Decimal, day: u32, ouverte: bool) -> Candidate {
        let (debit, credit) = if montant > Decimal::ZERO {
            (montant, Decimal::ZERO)
        } else {
            (Decimal::ZERO, -montant)
        };
        Candidate {
            line_id,
            entry_id: line_id * 10,
            reverses_entry_id: None,
            date: d(day),
            debit,
            credit,
            in_open_period: ouverte,
        }
    }

    /// Les paires rendues, en identifiants de lignes `(débit, crédit)`.
    fn ids(cands: &[Candidate], paires: &[Pair]) -> Vec<(i64, i64)> {
        paires
            .iter()
            .map(|p| (cands[p.debit].line_id, cands[p.credit].line_id))
            .collect()
    }

    /// Test 11 (AC5) — classement stable, une ligne par paire, paire de
    /// contre-passation en tête.
    #[test]
    fn ranking_is_stable_one_pair_per_line_reversal_first() {
        let mut cands = vec![
            ligne(1, dec!(100), 1, true),
            ligne(2, dec!(-100), 2, true),  // écart 1 avec 1
            ligne(3, dec!(-100), 20, true), // écart 19 avec 1 ; contre-passe 4
            ligne(4, dec!(100), 20, true),  // écart 0 avec 3
            ligne(5, dec!(50), 5, true),
            ligne(6, dec!(-50), 5, true), // écart 0 avec 5
            ligne(7, dec!(-50), 5, true), // même écart : départage par line_id
            ligne(8, dec!(30), 1, true),  // pas de crédit de 30 : aucune paire
        ];
        cands[2].reverses_entry_id = Some(cands[0].entry_id); // 3 contre-passe 1
        let paires = propose_pairs(&cands);
        // 1–3 (contre-passation) d'abord, malgré l'écart de 19 jours ; puis les
        // écarts nuls par line_id du débit : 4–3 (3 est pris, écartée), 5–6, 5–7
        // (5 est pris) ; 1–2 (1 est pris) ; enfin 4–2 (écart 18). 7 et 8 restent
        // seuls.
        assert_eq!(ids(&cands, &paires), vec![(1, 3), (5, 6), (4, 2)]);
        assert!(paires[0].reversal_pair);
        assert!(!paires[1].reversal_pair && !paires[2].reversal_pair);
        assert_eq!(
            (
                paires[0].days_apart,
                paires[1].days_apart,
                paires[2].days_apart
            ),
            (19, 0, 18)
        );
        assert_eq!(paires[1].amount, dec!(50));
        // Stabilité : l'ordre d'entrée ne change pas la sortie.
        let mut inverse = cands.clone();
        inverse.reverse();
        assert_eq!(
            ids(&inverse, &propose_pairs(&inverse)),
            ids(&cands, &paires)
        );
        // Sens de la contre-passation indifférent (l'origine contre-passe).
        let mut autre = cands.clone();
        autre[2].reverses_entry_id = None;
        autre[0].reverses_entry_id = Some(autre[2].entry_id);
        assert_eq!(ids(&autre, &propose_pairs(&autre)), ids(&cands, &paires));
    }

    /// Test 11 (AC5) — le filtre R7 s'applique AVANT le glouton : `A` débit et
    /// `B` crédit en période close, `C` crédit en période ouverte, même montant
    /// et même date. Un filtre APRÈS le glouton aurait pris `A–B` (première
    /// rencontrée, écart nul), puis l'aurait écartée : rien.
    #[test]
    fn closed_period_pairs_are_filtered_before_greedy_matching() {
        let cands = vec![
            ligne(1, dec!(100), 3, false),  // A
            ligne(2, dec!(-100), 3, false), // B
            ligne(3, dec!(-100), 3, true),  // C
        ];
        assert_eq!(ids(&cands, &propose_pairs(&cands)), vec![(1, 3)]);
        // Une paire dont une seule ligne est ouverte est acceptable, dans les
        // deux sens.
        let cands = vec![ligne(1, dec!(100), 3, true), ligne(2, dec!(-100), 3, false)];
        assert_eq!(ids(&cands, &propose_pairs(&cands)), vec![(1, 2)]);
        // Toute close : rien.
        let cands = vec![
            ligne(1, dec!(100), 3, false),
            ligne(2, dec!(-100), 3, false),
        ];
        assert!(propose_pairs(&cands).is_empty());
    }

    /// AC5 — pas de tolérance de montant ; même sens jamais apparié ; une
    /// ligne à deux montants (hors contrainte) jamais appariée ; montants égaux
    /// à des échelles différentes appariés.
    #[test]
    fn amounts_must_be_exactly_equal_and_opposite() {
        let mut cands = vec![
            ligne(1, dec!(100), 1, true),
            ligne(2, dec!(-99.95), 1, true),
            ligne(3, dec!(100), 1, true),
            ligne(4, dec!(-100.0000), 9, true),
        ];
        assert_eq!(ids(&cands, &propose_pairs(&cands)), vec![(1, 4)]);
        cands[3].debit = dec!(1);
        assert!(propose_pairs(&cands).is_empty());
    }

    /// Test 14 (AC5) — volume au plafond : 1 000 débits et 1 000 crédits d'un
    /// même montant, 10⁶ paires classées, 1 000 retenues. Temps noté au Dev
    /// Agent Record (`cargo test … -- --nocapture`).
    #[test]
    fn volume_at_the_cap() {
        let mut cands = Vec::with_capacity(MAX_PROPOSAL_CANDIDATES);
        for i in 0..1_000i64 {
            cands.push(ligne(2 * i + 1, dec!(100), 1 + (i % 28) as u32, i % 3 != 0));
            cands.push(ligne(
                2 * i + 2,
                dec!(-100),
                1 + ((i * 7) % 28) as u32,
                true,
            ));
        }
        assert_eq!(cands.len(), MAX_PROPOSAL_CANDIDATES);
        let debut = std::time::Instant::now();
        let paires = propose_pairs(&cands);
        let duree = debut.elapsed();
        eprintln!("moteur des propositions, 1 000 × 1 000 : {duree:?}");
        assert_eq!(paires.len(), 1_000);
        let mut vues = std::collections::BTreeSet::new();
        for p in &paires {
            assert!(
                vues.insert(p.debit) && vues.insert(p.credit),
                "une ligne par paire"
            );
        }
        // Classement : écarts croissants (aucune contre-passation ici).
        assert!(
            paires
                .windows(2)
                .all(|w| w[0].days_apart <= w[1].days_apart)
        );
    }
}
