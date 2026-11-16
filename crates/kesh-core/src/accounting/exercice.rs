//! Les dates et le nom par défaut d'un exercice (Story 37-1, AC-11).
//!
//! L'assistant de démarrage crée le premier exercice ; l'utilisateur en choisit
//! le début, et la fin se propose d'elle-même : **la veille du même jour un an
//! plus tard**. Le miroir de l'écran (`frontend/src/lib/features/onboarding/
//! premier-exercice.ts`) rend la même fin, et les mêmes cas limites sont testés
//! des deux côtés.

use chrono::{Datelike, NaiveDate};

/// La fin proposée pour un exercice qui commence le `debut` : la veille du même
/// jour un an plus tard (1er janvier → 31 décembre, 1er avril → 31 mars
/// suivant).
///
/// Le « même jour un an plus tard » d'un 29 février est le **1er mars** suivant,
/// d'où une fin au **28 février** (et non au 27, que donnerait l'ajout de douze
/// mois, qui ramène au 28 février). `None` seulement si l'année suivante sort de
/// ce que `chrono` sait représenter.
#[must_use]
pub fn fin_par_defaut(debut: NaiveDate) -> Option<NaiveDate> {
    let annee_suivante = debut.year().checked_add(1)?;
    let meme_jour = debut
        .with_year(annee_suivante)
        .or_else(|| NaiveDate::from_ymd_opt(annee_suivante, 3, 1))?;
    meme_jour.pred_opt()
}

/// Le nom d'un exercice : « Exercice AAAA » si début et fin tombent la même
/// année, « Exercice AAAA/AAAA » (années de début et de fin) sinon. Au plus
/// 22 caractères pour des années à quatre chiffres, sous la limite de la
/// colonne (`VARCHAR(50)`).
#[must_use]
pub fn nom_par_defaut(debut: NaiveDate, fin: NaiveDate) -> String {
    if debut.year() == fin.year() {
        format!("Exercice {}", debut.year())
    } else {
        format!("Exercice {}/{}", debut.year(), fin.year())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jour(a: i32, m: u32, j: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(a, m, j).unwrap()
    }

    #[test]
    fn le_1er_janvier_finit_le_31_decembre() {
        assert_eq!(fin_par_defaut(jour(2026, 1, 1)), Some(jour(2026, 12, 31)));
    }

    #[test]
    fn le_1er_avril_finit_le_31_mars_suivant() {
        assert_eq!(fin_par_defaut(jour(2026, 4, 1)), Some(jour(2027, 3, 31)));
    }

    #[test]
    fn le_1er_juillet_finit_le_30_juin_suivant() {
        assert_eq!(fin_par_defaut(jour(2026, 7, 1)), Some(jour(2027, 6, 30)));
    }

    #[test]
    fn le_29_fevrier_finit_le_28_fevrier_suivant() {
        assert_eq!(fin_par_defaut(jour(2028, 2, 29)), Some(jour(2029, 2, 28)));
    }

    #[test]
    fn le_1er_mars_avant_une_annee_bissextile_finit_le_29_fevrier() {
        assert_eq!(fin_par_defaut(jour(2027, 3, 1)), Some(jour(2028, 2, 29)));
    }

    #[test]
    fn le_28_fevrier_d_une_annee_bissextile_finit_le_27_fevrier_suivant() {
        assert_eq!(fin_par_defaut(jour(2028, 2, 28)), Some(jour(2029, 2, 27)));
    }

    #[test]
    fn la_fin_est_toujours_posterieure_au_debut() {
        let mut d = jour(2027, 1, 1);
        while d < jour(2029, 1, 1) {
            let f = fin_par_defaut(d).unwrap();
            assert!(f > d, "{d} → {f}");
            d = d.succ_opt().unwrap();
        }
    }

    #[test]
    fn le_nom_porte_une_ou_deux_annees() {
        assert_eq!(
            nom_par_defaut(jour(2026, 1, 1), jour(2026, 12, 31)),
            "Exercice 2026"
        );
        assert_eq!(
            nom_par_defaut(jour(2026, 4, 1), jour(2027, 3, 31)),
            "Exercice 2026/2027"
        );
        assert_eq!(
            nom_par_defaut(jour(2026, 7, 1), jour(2027, 12, 31)),
            "Exercice 2026/2027"
        );
    }
}
