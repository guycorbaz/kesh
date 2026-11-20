//! L'horloge du serveur — la source de la date du jour des sites convertis (Story 35-5a0, #929,
//! #1022, #907).
//!
//! Pour la contre-passation, les annulations, le verrou de période, les soldes d'ouverture et le
//! premier exercice de l'assistant, le serveur ne lit plus l'heure lui-même : il la demande à une
//! [`Horloge`], injectée dans l'état de l'application et passée à `kesh-db` sous la forme d'une
//! date. Les sites qui lisent encore l'heure (Story 35-5b) sont inventoriés par la garde
//! `kesh-api/tests/date_du_jour_garde.rs`. Les tests fixent l'horloge par [`HorlogeFixe`].
//!
//! Le jour rendu par [`Horloge::aujourd_hui`] est **le jour de Zurich** de l'instant
//! ([`jour_suisse`], Story 35-5a) — jusqu'à la 35-5a0, le jour UTC. Le serveur et les tests qui
//! passent par l'horloge ont basculé ensemble.
//!
//! Le sens inverse — les instants qui **bornent** un jour de Zurich — est
//! [`debut_du_jour_suisse`] et [`fin_du_jour_suisse`] (Story 36-10, #1147, #1053) : définis par
//! [`jour_suisse`] et testés par elle, ce sont, avec [`heure_suisse`] (Story 38-12a), les seules
//! autres fonctions du fuseau. `kesh-db` les
//! appelle pour filtrer le journal d'audit et borner la fenêtre de la réconciliation ; il ne
//! recalcule aucun décalage.
//!
//! Le module est pur (aucune I/O) : la lecture de la variable d'environnement de test se fait chez
//! l'appelant, qui passe sa valeur à [`instant_de_test`].

use chrono::{DateTime, Duration, FixedOffset, LocalResult, NaiveDate, NaiveTime, TimeZone, Utc};

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
/// Kesh est un logiciel de comptabilité suisse : le fuseau est une constante
/// (C-e350-16), jamais `TZ` ni `chrono::Local`. Le sens instant → heure locale
/// n'est jamais ambigu (un instant UTC a une seule heure à Zurich), contrairement
/// au sens inverse. Entre 22 h ou 23 h UTC et minuit UTC, ce jour est le
/// lendemain du jour UTC.
///
/// ⚠️ Remplacer `Europe::Zurich` par `Europe::Paris`, `Berlin`, `Vienna` ou `Rome`
/// serait une mutation **équivalente** pour tout instant postérieur à 1981 (mêmes
/// règles d'heure d'été) : aucun test ne peut la distinguer, et ce n'est pas un trou
/// de couverture (revue P1, E4).
pub fn jour_suisse(instant: DateTime<Utc>) -> NaiveDate {
    instant
        .with_timezone(&chrono_tz::Europe::Zurich)
        .date_naive()
}

/// L'heure de Zurich d'un instant, avec son décalage (`+01:00` ou `+02:00`) — Story 38-12a :
/// l'API PayPal rend ses dates en UTC, le CSV les écrit à l'heure du compte ; la conversion les
/// ramène à cette heure. Même règle que [`jour_suisse`], dont elle donne la date :
/// `heure_suisse(t).date_naive() == jour_suisse(t)`.
pub fn heure_suisse(instant: DateTime<Utc>) -> DateTime<FixedOffset> {
    instant
        .with_timezone(&chrono_tz::Europe::Zurich)
        .fixed_offset()
}

/// Le **premier** instant, à la milliseconde, dont le [`jour_suisse`] vaut `jour` (Story 36-10,
/// #1147, #1053).
///
/// Contrat, tenu par les tests : `jour_suisse(debut_du_jour_suisse(j)) == j` et
/// `jour_suisse(debut_du_jour_suisse(j) − 1 ms) == j − 1`. Le jour de 23 heures (fin mars) et
/// celui de 25 heures (fin octobre) sortent justes par construction : minuit n'y est jamais
/// ambigu ni absent (le changement d'heure se fait à 2 h ou 3 h).
///
/// Algorithme : minuit de `jour` à Zurich, `Single(t)` → `t` ; `Ambiguous(a, b)` → le plus tôt
/// des deux ; `None` (minuit n'existe pas — le seul cas est le 1894-06-01, passage de l'heure de
/// Berne, `+00:29:46`, à l'heure d'Europe centrale) → recherche **dichotomique à la
/// milliseconde** du premier instant de `[j 00:00Z − 3 h, j 00:00Z + 1 h]` dont le jour suisse
/// est au moins `j`. Elle est exacte parce que [`jour_suisse`] est croissante au sens large et
/// que tout décalage de Zurich est compris entre `0` et `+2 h`. Le bras `Ambiguous` n'est
/// atteint par aucune date de la base de fuseaux de Zurich (aucun recul d'heure à minuit) : il
/// est écrit pour la sûreté, sans test qui l'exerce.
///
/// ⛔ **Précondition** : `jour` dans `[1000-01-01, 9999-12-31]`, la plage que les routes
/// garantissent (`kesh_core::dates::annee_stockable`). Hors d'elle, le résultat n'est pas défini
/// (il ne panique pas pour autant : les débordements retombent sur une borne ; constaté sur
/// `NaiveDate::MIN` et `NaiveDate::MAX` par un test).
///
/// ⚠️ La base de fuseaux de `chrono-tz` 0.10.4 ne modélise l'heure d'été de Zurich que jusqu'en
/// 2099 (dernière transition en octobre 2099) : au-delà, [`jour_suisse`] et ses deux inverses
/// comptent en CET, d'accord entre eux — sans effet sur des données réelles.
pub fn debut_du_jour_suisse(jour: NaiveDate) -> DateTime<Utc> {
    let minuit = jour.and_time(NaiveTime::MIN);
    match chrono_tz::Europe::Zurich.from_local_datetime(&minuit) {
        LocalResult::Single(t) => t.with_timezone(&Utc),
        LocalResult::Ambiguous(a, b) => a.with_timezone(&Utc).min(b.with_timezone(&Utc)),
        LocalResult::None => premier_instant_du_jour(jour, minuit.and_utc()),
    }
}

/// Le **dernier** instant, à la milliseconde, dont le [`jour_suisse`] vaut `jour` :
/// `debut_du_jour_suisse(jour + 1) − 1 ms` (Story 36-10).
///
/// Contrat, tenu par les tests : `jour_suisse(fin_du_jour_suisse(j)) == j` et
/// `jour_suisse(fin_du_jour_suisse(j) + 1 ms) == j + 1`. Même précondition et même limite de la
/// base de fuseaux que [`debut_du_jour_suisse`].
pub fn fin_du_jour_suisse(jour: NaiveDate) -> DateTime<Utc> {
    match jour.succ_opt() {
        Some(lendemain) => debut_du_jour_suisse(lendemain)
            .checked_sub_signed(Duration::milliseconds(1))
            .unwrap_or(DateTime::<Utc>::MIN_UTC),
        // Hors précondition (`NaiveDate::MAX`) : aucune panique, la borne la plus haute.
        None => DateTime::<Utc>::MAX_UTC,
    }
}

/// La dichotomie de [`debut_du_jour_suisse`], quand minuit n'existe pas à Zurich.
///
/// Invariant : `jour_suisse(bas) < jour <= jour_suisse(haut)`. `bas = j 00:00Z − 3 h` tombe, à
/// Zurich, au plus tard à 23 h la veille (décalage d'au plus `+2 h`) ; `haut = j 00:00Z + 1 h`, au
/// plus tôt à 1 h le jour même (décalage d'au moins `0`).
fn premier_instant_du_jour(jour: NaiveDate, minuit_utc: DateTime<Utc>) -> DateTime<Utc> {
    let mut bas = minuit_utc
        .checked_sub_signed(Duration::hours(3))
        .unwrap_or(DateTime::<Utc>::MIN_UTC);
    let mut haut = minuit_utc
        .checked_add_signed(Duration::hours(1))
        .unwrap_or(DateTime::<Utc>::MAX_UTC);
    while (haut - bas).num_milliseconds() > 1 {
        let milieu = bas + Duration::milliseconds((haut - bas).num_milliseconds() / 2);
        if jour_suisse(milieu) >= jour {
            haut = milieu;
        } else {
            bas = milieu;
        }
    }
    haut
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
    #[test]
    fn heure_suisse_porte_le_decalage_et_la_date_du_jour_suisse() {
        use chrono::TimeZone as _;
        let hiver = super::Utc.with_ymd_and_hms(2024, 1, 31, 23, 30, 0).unwrap();
        let h = super::heure_suisse(hiver);
        assert_eq!(h.to_rfc3339(), "2024-02-01T00:30:00+01:00");
        assert_eq!(h.date_naive(), super::jour_suisse(hiver));
        let ete = super::Utc.with_ymd_and_hms(2024, 7, 1, 21, 59, 59).unwrap();
        assert_eq!(
            super::heure_suisse(ete).to_rfc3339(),
            "2024-07-01T23:59:59+02:00"
        );
    }

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

    /// AC1 (Story 35-5a) — le jour suisse ne diffère du jour UTC qu'entre 22:00Z ou 23:00Z et
    /// minuit UTC : les instants sont pris dans cette fenêtre, de part et d'autre de chaque
    /// changement d'heure, plus les bornes à la seconde. Attendus en dates fixes (aucun
    /// `.date_naive()`).
    const AC1: &[(&str, (i32, u32, u32), &str)] = &[
        (
            "2025-12-31T23:30:00Z",
            (2026, 1, 1),
            "00 h 30 CET : jour UTC (M1)",
        ),
        (
            "2026-07-01T22:30:00Z",
            (2026, 7, 2),
            "00 h 30 CEST : fuseau fixe +1 h (M2)",
        ),
        (
            "2026-03-28T22:30:00Z",
            (2026, 3, 28),
            "23 h 30 CET : fuseau fixe +2 h",
        ),
        (
            "2026-03-28T23:30:00Z",
            (2026, 3, 29),
            "00 h 30 CET, veille de l'heure d'été",
        ),
        (
            "2026-03-29T22:30:00Z",
            (2026, 3, 30),
            "00 h 30 CEST, première nuit d'été",
        ),
        (
            "2026-10-24T22:30:00Z",
            (2026, 10, 25),
            "00 h 30 CEST, veille de l'heure d'hiver",
        ),
        (
            "2026-10-25T22:30:00Z",
            (2026, 10, 25),
            "23 h 30 CET : l'inverse de M2",
        ),
        ("2026-06-15T10:00:00Z", (2026, 6, 15), "le jour"),
        (
            "2026-06-30T21:59:59Z",
            (2026, 6, 30),
            "borne été, 23 h 59 min 59 s",
        ),
        ("2026-06-30T22:00:00Z", (2026, 7, 1), "borne été, minuit"),
        (
            "2025-12-31T22:59:59Z",
            (2025, 12, 31),
            "borne hiver, 23 h 59 min 59 s",
        ),
        ("2025-12-31T23:00:00Z", (2026, 1, 1), "borne hiver, minuit"),
        // Revue P1 (E2) : le premier minuit à l'heure d'hiver, à la seconde. La seconde
        // même du changement d'heure (01:00:00Z) tombe à 02 h ou 03 h locales : elle ne
        // peut pas changer le jour.
        (
            "2026-10-25T22:59:59Z",
            (2026, 10, 25),
            "premier soir d'hiver, 23 h 59 min 59 s CET",
        ),
        (
            "2026-10-25T23:00:00Z",
            (2026, 10, 26),
            "premier minuit d'hiver, 00 h CET",
        ),
    ];

    #[test]
    fn le_jour_est_celui_de_zurich() {
        for (i, (a, m, j), motif) in AC1 {
            let attendu = NaiveDate::from_ymd_opt(*a, *m, *j).unwrap();
            assert_eq!(jour_suisse(instant(i)), attendu, "{i} — {motif}");
        }
    }

    /// Story 37-2 (AC-4) — minuit de Zurich en UTC, aux deux changements d'heure, au passage
    /// d'année et en plein été ; et l'aller-retour avec [`jour_suisse`]. Écrit pour l'aide
    /// `debut_jour_suisse` de la 37-2, remplacée à l'intégration de la v0.18.0 par
    /// [`debut_du_jour_suisse`] de la Story 36-10, de même contrat (une seule fonction).
    #[test]
    fn debut_du_jour_suisse_aux_changements_d_heure() {
        let cas = [
            (
                (2026, 3, 29),
                "2026-03-28T23:00:00Z",
                "nuit du passage à l'heure d'été",
            ),
            (
                (2026, 3, 30),
                "2026-03-29T22:00:00Z",
                "premier minuit d'été",
            ),
            (
                (2026, 10, 25),
                "2026-10-24T22:00:00Z",
                "nuit du passage à l'heure d'hiver",
            ),
            (
                (2026, 10, 26),
                "2026-10-25T23:00:00Z",
                "premier minuit d'hiver",
            ),
            ((2027, 1, 1), "2026-12-31T23:00:00Z", "31 → 1er, hiver"),
            ((2026, 7, 1), "2026-06-30T22:00:00Z", "30 → 1er, été"),
        ];
        for ((a, m, j), attendu, motif) in cas {
            let jour = NaiveDate::from_ymd_opt(a, m, j).unwrap();
            assert_eq!(debut_du_jour_suisse(jour), instant(attendu), "{motif}");
            assert_eq!(jour_suisse(debut_du_jour_suisse(jour)), jour, "{motif}");
            assert_eq!(
                jour_suisse(debut_du_jour_suisse(jour) - chrono::TimeDelta::seconds(1)),
                jour.pred_opt().unwrap(),
                "{motif} : la seconde d'avant est la veille"
            );
        }
    }

    #[test]
    fn l_horloge_fixe_rend_le_jour_suisse() {
        for (i, _, motif) in AC1 {
            assert_eq!(
                HorlogeFixe(instant(i)).aujourd_hui(),
                jour_suisse(instant(i)),
                "{i} — {motif}"
            );
        }
        // Un instant donné avec un décalage est ramené en UTC, puis à Zurich.
        let h = HorlogeFixe(instant("2026-01-01T00:30:00+01:00"));
        assert_eq!(
            h.aujourd_hui(),
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
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
        // Le jour suisse de cet instant (Story 35-5a) : le 1er janvier.
        assert_eq!(
            HorlogeFixe(i).aujourd_hui(),
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
        );
    }

    #[test]
    #[should_panic(expected = "KESH_TEST_INSTANT")]
    fn instant_de_test_invalide_arrete_le_test_en_nommant_la_variable() {
        instant_de_test(Some("31.12.2025"));
    }

    fn jour(a: i32, m: u32, j: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(a, m, j).unwrap()
    }

    /// AC1 (Story 36-10) — les bornes d'un jour de Zurich, en dates fixes : hiver, été, le jour
    /// de 23 heures et celui de 25 heures. Attendus écrits en clair, sans appeler `jour_suisse`.
    #[test]
    fn les_bornes_d_un_jour_de_zurich_en_dates_fixes() {
        let cas: &[(NaiveDate, &str, &str, &str)] = &[
            (
                jour(2026, 1, 15),
                "2026-01-14T23:00:00.000Z",
                "2026-01-15T22:59:59.999Z",
                "hiver, CET",
            ),
            (
                jour(2026, 9, 15),
                "2026-09-14T22:00:00.000Z",
                "2026-09-15T21:59:59.999Z",
                "été, CEST",
            ),
            (
                jour(2026, 3, 29),
                "2026-03-28T23:00:00.000Z",
                "2026-03-29T21:59:59.999Z",
                "jour de 23 heures",
            ),
            (
                jour(2026, 10, 25),
                "2026-10-24T22:00:00.000Z",
                "2026-10-25T22:59:59.999Z",
                "jour de 25 heures",
            ),
        ];
        for (j, debut, fin, motif) in cas {
            assert_eq!(
                debut_du_jour_suisse(*j),
                instant(debut),
                "{j} début — {motif}"
            );
            assert_eq!(fin_du_jour_suisse(*j), instant(fin), "{j} fin — {motif}");
        }
    }

    /// AC1 (Story 36-10) — le contrat par `jour_suisse`, à ± 1 ms, sur chaque jour de 2025 à 2027
    /// et sur les jours extrêmes : `1000-01-01` (heure locale moyenne), `1894-06-01` (minuit
    /// absent, la dichotomie) et `9999-12-31`. Aucune panique.
    #[test]
    fn les_bornes_d_un_jour_suivent_jour_suisse_a_la_milliseconde() {
        let ms = Duration::milliseconds(1);
        let mut jours: Vec<NaiveDate> = Vec::new();
        let mut j = jour(2025, 1, 1);
        while j <= jour(2027, 12, 31) {
            jours.push(j);
            j = j.succ_opt().unwrap();
        }
        jours.extend([jour(1000, 1, 1), jour(1894, 6, 1), jour(9999, 12, 31)]);
        for j in jours {
            let veille = j.pred_opt().unwrap();
            let lendemain = j.succ_opt().unwrap();
            let debut = debut_du_jour_suisse(j);
            let fin = fin_du_jour_suisse(j);
            assert_eq!(jour_suisse(debut), j, "{j} : début");
            assert_eq!(jour_suisse(debut - ms), veille, "{j} : début − 1 ms");
            assert_eq!(jour_suisse(fin), j, "{j} : fin");
            assert_eq!(jour_suisse(fin + ms), lendemain, "{j} : fin + 1 ms");
        }
    }

    /// Le 1894-06-01, minuit n'existe pas à Zurich (passage de `+00:29:46` à `+01:00`) : le
    /// premier instant du jour est 00 h 30 min 14 s à l'heure d'Europe centrale (minuit à
    /// l'heure de Berne), soit 23 h 30 min 14 s UTC la veille. Attendu écrit en clair : une
    /// avance par minutes depuis minuit tomberait sur 00 h 31, 46 s plus tard (M8).
    #[test]
    fn le_jour_sans_minuit_de_1894_commence_au_bon_instant() {
        assert_eq!(
            debut_du_jour_suisse(jour(1894, 6, 1)),
            instant("1894-05-31T23:30:14.000Z")
        );
    }

    /// Revue P1 de la 36-10 (E1-3) — hors précondition, aucune panique : les bornes de `chrono`
    /// rendent un instant, dont la valeur n'est pas définie (aucune assertion de valeur).
    #[test]
    fn hors_precondition_les_bornes_ne_paniquent_pas() {
        for j in [NaiveDate::MIN, NaiveDate::MAX] {
            let _ = debut_du_jour_suisse(j);
            let _ = fin_du_jour_suisse(j);
        }
    }

    #[test]
    fn le_message_de_panique_nomme_la_constante() {
        let erreur =
            std::panic::catch_unwind(|| instant_de_test(Some("pas une date"))).unwrap_err();
        let message = erreur.downcast_ref::<String>().unwrap();
        assert!(message.contains(NOM_VARIABLE_INSTANT), "{message}");
    }
}
