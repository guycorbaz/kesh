//! L'horloge du serveur — la source de la date du jour des sites convertis (Story 35-5a0, #929,
//! #1022, #907).
//!
//! Pour la contre-passation, les annulations, le verrou de période, les soldes d'ouverture et le
//! premier exercice de l'assistant, le serveur ne lit plus l'heure lui-même : il la demande à une
//! [`Horloge`], injectée dans l'état de l'application et passée à `kesh-db` sous la forme d'une
//! date. Les sites qui lisent encore l'heure (Story 35-5b) sont inventoriés par la garde
//! `kesh-api/tests/date_du_jour_garde.rs`. Les tests fixent l'horloge par [`HorlogeFixe`].
//!
//! Le jour rendu par [`Horloge::aujourd_hui`] est, à ce stade, **le jour UTC** de l'instant —
//! exactement celui d'avant cette story. La Story 35-5a le remplacera par le jour de Zurich en ne
//! changeant que la fonction privée `jour` : le serveur et les tests qui passent par l'horloge
//! basculeront ensemble.
//!
//! Le module est pur (aucune I/O) : la lecture de la variable d'environnement de test se fait chez
//! l'appelant, qui passe sa valeur à [`instant_de_test`].

use chrono::{DateTime, NaiveDate, Utc};

/// Nom de la variable d'environnement qui fixe l'instant des **tests** (RFC 3339).
///
/// Seul littéral de ce nom dans le dépôt : tout le reste — message de panique, helpers de tests —
/// passe par cette constante. Le binaire servi ne la lit jamais.
pub const NOM_VARIABLE_INSTANT: &str = "KESH_TEST_INSTANT";

/// Source de l'instant présent, et donc de la date du jour.
pub trait Horloge: Send + Sync {
    /// L'instant présent.
    fn maintenant(&self) -> DateTime<Utc>;

    /// La date du jour, tirée de [`Horloge::maintenant`] : [`jour_suisse`].
    fn aujourd_hui(&self) -> NaiveDate {
        jour_suisse(self.maintenant())
    }
}

/// Horloge arrêtée sur un instant donné — pour les tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HorlogeFixe(pub DateTime<Utc>);

impl Horloge for HorlogeFixe {
    fn maintenant(&self) -> DateTime<Utc> {
        self.0
    }
}

/// Le jour civil d'un instant **à Zurich** (Story 35-5a, #1022, #907, #929).
///
/// *Squelette* : rend encore le jour UTC.
pub fn jour_suisse(instant: DateTime<Utc>) -> NaiveDate {
    instant.date_naive()
}

/// Interprète la valeur de la variable [`NOM_VARIABLE_INSTANT`].
///
/// La valeur est rognée de ses blancs, comme le fait `config::env_nonempty` côté serveur : `None`,
/// la chaîne vide et une chaîne blanche (variable exportée vide) rendent `None` — l'horloge n'est
/// pas fixée. Un instant à décalage (`+01:00`) est ramené en UTC.
///
/// # Panics
///
/// Une valeur qui n'est pas un instant RFC 3339 arrête le test, en nommant la variable.
pub fn instant_de_test(valeur: Option<&str>) -> Option<DateTime<Utc>> {
    let valeur = valeur?.trim();
    if valeur.is_empty() {
        return None;
    }
    match DateTime::parse_from_rfc3339(valeur) {
        Ok(instant) => Some(instant.with_timezone(&Utc)),
        Err(e) => panic!("{NOM_VARIABLE_INSTANT} n'est pas un instant RFC 3339 ({valeur:?}) : {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn instant(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    #[test]
    fn horloge_fixe_rend_son_instant_et_son_jour() {
        let h = HorlogeFixe(instant("2026-06-15T10:00:00Z"));
        assert_eq!(h.maintenant(), instant("2026-06-15T10:00:00Z"));
        assert_eq!(
            h.aujourd_hui(),
            NaiveDate::from_ymd_opt(2026, 6, 15).unwrap()
        );
    }

    #[test]
    fn le_jour_est_le_jour_utc() {
        // 23 h 30 UTC le 31 décembre : encore le 31 décembre en UTC (comportement d'avant).
        let h = HorlogeFixe(instant("2025-12-31T23:30:00Z"));
        assert_eq!(
            h.aujourd_hui(),
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()
        );
        // Un instant donné avec un décalage est ramené en UTC avant d'en tirer le jour.
        let h = HorlogeFixe(instant("2026-01-01T00:30:00+01:00"));
        assert_eq!(
            h.aujourd_hui(),
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()
        );
    }

    #[test]
    fn instant_de_test_lit_un_instant_rfc3339() {
        assert_eq!(
            instant_de_test(Some("2025-12-31T23:30:00Z")),
            Some(instant("2025-12-31T23:30:00Z"))
        );
    }

    #[test]
    fn instant_de_test_absent_ou_vide_rend_none() {
        assert_eq!(instant_de_test(None), None);
        assert_eq!(instant_de_test(Some("")), None);
        assert_eq!(instant_de_test(Some("   ")), None);
    }

    #[test]
    fn instant_de_test_rogne_et_ramene_un_decalage_en_utc() {
        assert_eq!(
            instant_de_test(Some(" 2025-12-31T23:30:00Z ")),
            Some(instant("2025-12-31T23:30:00Z"))
        );
        let i = instant_de_test(Some("2026-01-01T00:30:00+01:00")).unwrap();
        assert_eq!(i, instant("2025-12-31T23:30:00Z"));
        assert_eq!(
            HorlogeFixe(i).aujourd_hui(),
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()
        );
    }

    #[test]
    #[should_panic(expected = "KESH_TEST_INSTANT")]
    fn instant_de_test_invalide_arrete_le_test_en_nommant_la_variable() {
        instant_de_test(Some("31.12.2025"));
    }

    #[test]
    fn le_message_de_panique_nomme_la_constante() {
        let erreur =
            std::panic::catch_unwind(|| instant_de_test(Some("pas une date"))).unwrap_err();
        let message = erreur.downcast_ref::<String>().unwrap();
        assert!(message.contains(NOM_VARIABLE_INSTANT), "{message}");
    }
}
