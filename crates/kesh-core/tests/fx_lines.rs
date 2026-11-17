//! Story 37-3a (AC-3) — vecteurs communs des lignes en devise.
//!
//! - `fixtures/fx_rounding.json` : le CHF d'une ligne (`montant × cours`
//!   arrondi au centime, loin de zéro) et les refus de bornes ;
//! - `fixtures/fx_rate_from_chf.json` : le cours déduit d'un CHF.
//!
//! Les deux fichiers sont lus ici par `kesh-core` et, en 37-3b, par le Vitest
//! des jumeaux TypeScript (`fx.ts`, `big.js`) : un seul fichier, aucune copie.

use std::str::FromStr;

use kesh_core::accounting::fx::{
    ForeignAmount, FxLineError, RateFromChfError, RateProvenance, check_foreign_line, fx_chf,
    rate_from_chf,
};
use rust_decimal::Decimal;
use serde::Deserialize;

fn d(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap_or_else(|e| panic!("décimal invalide {s:?} : {e}"))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Arrondi {
    case: String,
    amount: String,
    rate: String,
    chf: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CoursDeduit {
    case: String,
    amount: String,
    chf: String,
    rate: Option<String>,
    error: Option<String>,
    expected: Option<String>,
}

fn ligne(amount: Decimal, rate: Decimal) -> ForeignAmount {
    ForeignAmount {
        declared_currency: None,
        amount,
        rate,
        source: Some(RateProvenance::Manual),
        rate_derived: false,
    }
}

#[test]
fn vecteurs_d_arrondi() {
    let vecteurs: Vec<Arrondi> =
        serde_json::from_str(include_str!("fixtures/fx_rounding.json")).unwrap();
    assert!(vecteurs.len() >= 20, "fixture complète");
    for v in &vecteurs {
        let (amount, rate) = (d(&v.amount), d(&v.rate));
        match (&v.chf, &v.error) {
            (Some(chf), None) => {
                let chf = d(chf);
                assert_eq!(fx_chf(&amount, &rate), Ok(chf), "{}", v.case);
                // Le contrôle de ligne ordinaire accepte ce CHF.
                assert_eq!(
                    check_foreign_line(&chf, &Decimal::ZERO, &ligne(amount, rate)),
                    Ok(()),
                    "{}",
                    v.case
                );
            }
            (None, Some(code)) => {
                assert_eq!(code, "FX_VALUE_OUT_OF_RANGE", "{}", v.case);
                assert_eq!(
                    check_foreign_line(&Decimal::ONE, &Decimal::ZERO, &ligne(amount, rate)),
                    Err(FxLineError::OutOfRange),
                    "{}",
                    v.case
                );
            }
            _ => panic!("vecteur mal formé : {}", v.case),
        }
    }
}

#[test]
fn vecteurs_du_cours_deduit() {
    let vecteurs: Vec<CoursDeduit> =
        serde_json::from_str(include_str!("fixtures/fx_rate_from_chf.json")).unwrap();
    assert!(vecteurs.len() >= 12, "fixture complète");
    for v in &vecteurs {
        let (amount, chf) = (d(&v.amount), d(&v.chf));
        let got = rate_from_chf(&amount, &chf);
        match (&v.rate, v.error.as_deref()) {
            (Some(rate), None) => {
                let rate = d(rate);
                assert_eq!(got, Ok(rate), "{}", v.case);
                // Chaque vecteur accepté repasse `round(amount × rate, 2) = chf`
                // par la fonction de l'étape 1.
                assert_eq!(fx_chf(&amount, &rate), Ok(chf), "{}", v.case);
            }
            (None, Some("FX_VALUE_OUT_OF_RANGE")) => {
                assert_eq!(got, Err(RateFromChfError::OutOfRange), "{}", v.case);
            }
            (None, Some("FX_AMOUNT_MISMATCH")) => {
                let expected = d(v.expected.as_deref().expect("expected"));
                assert_eq!(
                    got,
                    Err(RateFromChfError::Mismatch { expected }),
                    "{}",
                    v.case
                );
            }
            _ => panic!("vecteur mal formé : {}", v.case),
        }
    }
}

/// Propriété : pour des montants et des cours admis, le CHF de `fx_chf` est
/// celui d'un arrondi exact au centime, loin de zéro — comparé à un calcul de
/// référence en entiers indépendant (mantisses mises à l'échelle 14).
#[test]
fn propriete_arrondi_exact() {
    let montants = [
        "0.0001",
        "1",
        "12.5",
        "999.9999",
        "123456789.1234",
        "1000000",
    ];
    let cours = [
        "0.0000000001",
        "0.0057312",
        "0.5",
        "0.91193",
        "1.0000000005",
        "1234.5678901234",
    ];
    for m in montants {
        for c in cours {
            let (a, r) = (d(m), d(c));
            let ai = (a * Decimal::from(10_000)).trunc().mantissa();
            let ri = (r * Decimal::from(10_000_000_000i64)).trunc().mantissa();
            let p = ai * ri; // échelle 14
            let cents = (p + 500_000_000_000) / 1_000_000_000_000;
            let attendu = Decimal::from_i128_with_scale(cents, 2);
            assert_eq!(fx_chf(&a, &r).unwrap(), attendu, "{m} × {c}");
        }
    }
}

/// Revue de code P2 de la 37-3a — `debit + credit` ne panique pas au
/// débordement de `Decimal` : refus `OutOfRange`.
#[test]
fn somme_debit_credit_qui_deborde_est_refusee_sans_panique() {
    let foreign = ForeignAmount {
        declared_currency: None,
        amount: d("100"),
        rate: d("0.9"),
        rate_derived: false,
        source: Some(RateProvenance::Manual),
    };
    assert_eq!(
        check_foreign_line(&Decimal::MAX, &Decimal::MAX, &foreign),
        Err(FxLineError::OutOfRange)
    );
}
