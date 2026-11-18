//! Story 37-2 (AC-5, T4) — vecteurs communs des dates d'ouverture et de
//! clôture d'un exercice.
//!
//! La fixture `fixtures/fx_opening_dates.json` est lue ici par la fonction de
//! `kesh-core` et, en 37-3b, par le test Vitest de son jumeau TypeScript (qui ne
//! contrôle que `openingDate`) : un seul fichier, aucune copie.

use chrono::NaiveDate;
use kesh_core::exchange_rates::{ExerciceDevises, RateKind, closing_rate_date, opening_rate_date};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Vecteur {
    start_date: NaiveDate,
    end_date: NaiveDate,
    opening_rate_kind: RateKind,
    closing_rate_kind: RateKind,
    opening_date: NaiveDate,
    closing_date: NaiveDate,
}

fn vecteurs() -> Vec<Vecteur> {
    serde_json::from_str(include_str!("fixtures/fx_opening_dates.json"))
        .expect("fixture fx_opening_dates.json lisible")
}

#[test]
fn les_dates_d_ouverture_et_de_cloture_suivent_la_fixture() {
    let vs = vecteurs();
    assert!(
        vs.len() >= 16,
        "au moins huit exercices, en day et en month"
    );
    for v in &vs {
        assert_eq!(
            opening_rate_date(v.start_date, v.opening_rate_kind),
            v.opening_date,
            "{v:?}"
        );
        assert_eq!(
            closing_rate_date(v.end_date, v.closing_rate_kind),
            v.closing_date,
            "{v:?}"
        );
        let ex = ExerciceDevises {
            start: v.start_date,
            end: v.end_date,
            opening_kind: v.opening_rate_kind,
            closing_kind: v.closing_rate_kind,
        };
        assert_eq!(ex.opening_date(), v.opening_date, "{v:?}");
        assert_eq!(ex.closing_date(), v.closing_date, "{v:?}");
    }
}

/// Les cas exigés par l'AC-5 sont tous dans la fixture, chacun en `day` et en
/// `month` : la fixture ne peut pas maigrir sans que ce test le dise.
#[test]
fn la_fixture_porte_les_cas_exiges() {
    let vs = vecteurs();
    let d = |s: &str| NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap();
    for (debut, fin) in [
        ("2024-01-01", "2024-12-31"),
        ("2024-04-01", "2025-03-31"),
        ("2024-07-01", "2025-06-30"),
        ("2024-03-15", "2025-03-14"),
        ("2024-03-01", "2025-02-28"),
        ("2024-07-01", "2024-12-31"),
        ("2023-07-01", "2024-12-31"),
        ("2000-01-01", "2000-12-31"),
    ] {
        for kind in [RateKind::Day, RateKind::Month] {
            assert!(
                vs.iter().any(|v| v.start_date == d(debut)
                    && v.end_date == d(fin)
                    && v.opening_rate_kind == kind
                    && v.closing_rate_kind == kind),
                "cas manquant : {debut}–{fin} en {kind}"
            );
        }
    }
    // Veille du 01.03.2024 : le 29 février (année bissextile).
    assert!(vs.iter().any(|v| v.opening_date == d("2024-02-29")));
    // Exercice au 01.01.2000 : la date se calcule (31.12.1999).
    assert!(vs.iter().any(|v| v.opening_date == d("1999-12-31")));
}
