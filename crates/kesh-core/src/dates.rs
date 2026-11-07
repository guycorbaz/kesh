//! La plage d'années que la base sait tenir (Story 33-4, #606, #755).
//!
//! Les colonnes `DATE` et `DATETIME` de MariaDB vont de l'an 1000 à l'an 9999.
//! `chrono` lit bien au-delà (`+10000-01-01`, `-0001-01-01`) ; et hors de cette
//! plage, une valeur ne produit pas d'erreur uniforme : une année négative ou
//! au-delà de 65535 ne s'encode pas, une année de 10000 à 65535 s'écrit `NULL`
//! sans erreur (relevé T1 de la story). Tout ce qui reçoit une date du client la
//! compare donc à cette plage **avant** de s'en servir.

use chrono::{DateTime, Datelike, NaiveDate, Utc};

/// Première année qu'une colonne de date de MariaDB sait tenir.
pub const ANNEE_MIN: i32 = 1000;

/// Dernière année qu'une colonne de date de MariaDB sait tenir.
pub const ANNEE_MAX: i32 = 9999;

/// Une année que la base sait tenir.
const fn dans_la_plage(annee: i32) -> bool {
    annee >= ANNEE_MIN && annee <= ANNEE_MAX
}

/// `date` tient-elle dans la plage de la base ?
#[must_use]
pub fn annee_stockable(date: NaiveDate) -> bool {
    dans_la_plage(date.year())
}

/// Une expiration tient-elle dans la plage de la base ? L'année se lit **en
/// UTC**, celle de la colonne : `9999-12-31T23:59:59-05:00` est en l'an 10000.
#[must_use]
pub fn expiration_stockable(instant: DateTime<Utc>) -> bool {
    dans_la_plage(instant.year())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jour(a: i32, m: u32, j: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(a, m, j).unwrap()
    }

    #[test]
    fn les_deux_bornes_et_leurs_voisines() {
        assert!(!annee_stockable(jour(999, 12, 31)));
        assert!(annee_stockable(jour(1000, 1, 1)));
        assert!(annee_stockable(jour(9999, 12, 31)));
        assert!(!annee_stockable(jour(10000, 1, 1)));
        assert!(!annee_stockable(jour(-1, 1, 1)));
        assert!(!annee_stockable(jour(70000, 1, 1)));
    }

    #[test]
    fn l_expiration_se_lit_en_utc() {
        let lire = |s: &str| DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc);
        assert!(expiration_stockable(lire("9999-12-31T23:59:59Z")));
        assert!(!expiration_stockable(lire("9999-12-31T23:59:59-05:00")));
        assert!(expiration_stockable(lire("1000-01-01T00:00:00Z")));
        assert!(!expiration_stockable(lire("1000-01-01T00:00:00+05:00")));
    }
}
