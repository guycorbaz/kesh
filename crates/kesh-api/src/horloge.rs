//! L'horloge du serveur (Story 35-5a0, #929, #1022, #907).
//!
//! [`HorlogeSysteme`] lit l'heure réelle ; c'est celle que construit `main.rs`, et la seule que
//! le binaire servi connaisse. [`horloge_de_test`] est réservée aux tests : elle fixe l'instant
//! si la variable [`NOM_VARIABLE_INSTANT`] est posée, et rend l'horloge réelle sinon.
//!
//! ⛔ `main.rs` ne lit jamais [`NOM_VARIABLE_INSTANT`] et n'appelle ni [`horloge_depuis`] ni
//! [`horloge_de_test`] : la variable ne peut pas changer la date des livres. Des gardes de forme
//! le tiennent (`tests/date_du_jour_garde.rs`).

use std::sync::Arc;

use chrono::{DateTime, Utc};
use kesh_core::horloge::{Horloge, HorlogeFixe, NOM_VARIABLE_INSTANT, instant_de_test};

/// L'horloge réelle du serveur : `maintenant()` est `Utc::now()`.
#[derive(Debug, Clone, Copy, Default)]
pub struct HorlogeSysteme;

impl Horloge for HorlogeSysteme {
    fn maintenant(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// L'horloge que désigne la valeur d'une variable d'instant : arrêtée sur cet instant s'il y en
/// a un (cf. [`instant_de_test`]), l'horloge réelle sinon. Fonction pure, testable sans toucher
/// à l'environnement.
pub fn horloge_depuis(valeur: Option<&str>) -> Arc<dyn Horloge> {
    match instant_de_test(valeur) {
        Some(instant) => Arc::new(HorlogeFixe(instant)),
        None => Arc::new(HorlogeSysteme),
    }
}

/// L'horloge des **tests** : [`horloge_depuis`] la valeur de [`NOM_VARIABLE_INSTANT`], lue à
/// chaque appel (aucune mémoire). Employée par `AppState::new_for_tests` et les états de test.
///
/// **À ne jamais appeler depuis du code de production.**
#[doc(hidden)]
pub fn horloge_de_test() -> Arc<dyn Horloge> {
    horloge_depuis(std::env::var(NOM_VARIABLE_INSTANT).ok().as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn horloge_depuis_un_instant_est_arretee_sur_lui() {
        let h = horloge_depuis(Some("2025-12-31T23:30:00Z"));
        let attendu = DateTime::parse_from_rfc3339("2025-12-31T23:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(h.maintenant(), attendu);
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(h.maintenant(), attendu);
    }

    #[test]
    fn horloge_depuis_rien_n_est_pas_figee() {
        for valeur in [None, Some("")] {
            let h = horloge_depuis(valeur);
            let premiere = h.maintenant();
            std::thread::sleep(Duration::from_millis(10));
            assert_ne!(premiere, h.maintenant(), "{valeur:?}");
        }
    }
}
