//! Les dates reçues du client : le refus d'une année que la base ne sait pas
//! tenir (Story 33-4, #606, C-e334-1).
//!
//! Le refus dépend de ce que le site répond déjà, relevé à la story sur le code
//! d'avant :
//!
//! - **classe C** (aucun refus JSON) : [`date`] et [`date_opt`], à poser en
//!   `#[serde(deserialize_with = …)]`. Le refus est celui d'une date mal formée
//!   sur le site : 400 en texte sur un `Query`, 422 en texte sur un corps `Json`.
//! - **classe B** (un refus JSON existe, une autre valeur passait) : [`borner`]
//!   et [`borner_opt`], appelés dans le handler avant tout usage de la date,
//!   400 JSON `VALIDATION_ERROR`. Un `deserialize_with` y rendrait 422 en texte
//!   là où le site répond en JSON.
//! - **classe A** (déjà refusé en 4xx) : rien n'est posé.
//!
//! La garde `tests/dates_recues_garde.rs` tient l'inventaire : tout champ daté
//! d'un item `Deserialize` porte l'une de ces bornes, ou figure à sa liste
//! d'exceptions avec son motif.
//!
//! ⚠️ **Un `Option` en `deserialize_with` perd le « champ absent = `None` »** :
//! chaque usage de [`date_opt`] porte `default` en plus, sans quoi un champ
//! facultatif devient requis, en 422 sur des appels qui marchaient.

use chrono::NaiveDate;
use kesh_core::dates::annee_stockable;
use serde::{Deserialize, Deserializer};

use crate::errors::AppError;

/// Le texte commun à tous les refus de plage : c'est lui qui rend décidable un
/// test sur corps partiel (un champ requis manquant rend aussi 422).
const HORS_PLAGE: &str = "année hors de 1000 à 9999";

fn dans_la_plage<'de, D: Deserializer<'de>>(jour: NaiveDate) -> Result<NaiveDate, D::Error> {
    if annee_stockable(jour) {
        Ok(jour)
    } else {
        Err(serde::de::Error::custom(HORS_PLAGE))
    }
}

/// Une date obligatoire, dans la plage de la base.
pub fn date<'de, D: Deserializer<'de>>(deserializer: D) -> Result<NaiveDate, D::Error> {
    dans_la_plage::<D>(NaiveDate::deserialize(deserializer)?)
}

/// Une date facultative, dans la plage de la base. **Toujours avec `default`.**
pub fn date_opt<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<NaiveDate>, D::Error> {
    Option::<NaiveDate>::deserialize(deserializer)?
        .map(dans_la_plage::<D>)
        .transpose()
}

/// Le refus de plage d'un handler : `nom` est le nom du champ dans l'API.
pub fn borner(jour: NaiveDate, nom: &str) -> Result<(), AppError> {
    if annee_stockable(jour) {
        Ok(())
    } else {
        Err(AppError::Validation(format!(
            "{nom} invalide ({HORS_PLAGE})"
        )))
    }
}

/// [`borner`] sur une date facultative.
pub fn borner_opt(jour: Option<NaiveDate>, nom: &str) -> Result<(), AppError> {
    jour.map_or(Ok(()), |j| borner(j, nom))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize)]
    struct Corps {
        #[serde(deserialize_with = "date")]
        requise: NaiveDate,
        #[serde(default, deserialize_with = "date_opt")]
        facultative: Option<NaiveDate>,
    }

    fn lire(texte: &str) -> Result<Corps, String> {
        serde_json::from_str(texte).map_err(|e| e.to_string())
    }

    #[test]
    fn les_bornes_passent_et_leurs_voisines_non() {
        for bon in ["1000-01-01", "9999-12-31"] {
            let c = lire(&format!(r#"{{"requise":"{bon}","facultative":"{bon}"}}"#)).unwrap();
            assert_eq!(c.requise.to_string(), bon);
            assert_eq!(c.facultative.unwrap().to_string(), bon);
        }
        for hors in ["0999-12-31", "+10000-01-01", "-0001-01-01", "+70000-01-01"] {
            let e = lire(&format!(r#"{{"requise":"{hors}"}}"#)).unwrap_err();
            assert!(e.contains(HORS_PLAGE), "{e}");
            let e = lire(&format!(
                r#"{{"requise":"2026-01-01","facultative":"{hors}"}}"#
            ))
            .unwrap_err();
            assert!(e.contains(HORS_PLAGE), "{e}");
        }
    }

    #[test]
    fn le_champ_absent_et_le_null_restent_none() {
        assert!(
            lire(r#"{"requise":"2026-01-01"}"#)
                .unwrap()
                .facultative
                .is_none()
        );
        let c = lire(r#"{"requise":"2026-01-01","facultative":null}"#).unwrap();
        assert!(c.facultative.is_none());
    }

    #[test]
    fn une_date_mal_formee_reste_le_refus_de_chrono() {
        let e = lire(r#"{"requise":"2026-13-01"}"#).unwrap_err();
        assert!(!e.contains(HORS_PLAGE), "{e}");
    }

    #[test]
    fn borner_nomme_le_champ_et_la_plage() {
        let jour = NaiveDate::from_ymd_opt(999, 12, 31).unwrap();
        match borner(jour, "dateFrom") {
            Err(AppError::Validation(m)) => {
                assert_eq!(m, "dateFrom invalide (année hors de 1000 à 9999)");
            }
            _ => panic!("refus attendu"),
        }
        assert!(borner_opt(None, "dateFrom").is_ok());
        assert!(borner_opt(Some(jour), "dateFrom").is_err());
    }
}
