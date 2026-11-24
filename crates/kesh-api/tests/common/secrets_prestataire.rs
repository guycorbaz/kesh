//! Marqueur commun des secrets de prestataire — socle S9 (Story 38-12a, AC-9),
//! réutilisé par les tests de Wise (39-12a) et de Stripe (40-8a).
//!
//! Les identifiants, secrets et jetons des serveurs simulés **portent** le
//! marqueur [`MARQUEUR_SECRET`] ; un test rejoue des échecs (authentification,
//! page) et exige qu'aucune réponse, aucun corps d'erreur de tâche ni aucun
//! évènement `tracing` capté ne le contienne ([`aucun_secret`]).

/// Le marqueur que porte toute valeur secrète des serveurs simulés.
pub const MARQUEUR_SECRET: &str = "KESHSECRETDETEST";

/// Une valeur secrète de test : `<genre>-<marqueur>-<suffixe>`.
pub fn secret(genre: &str) -> String {
    format!("{genre}-{MARQUEUR_SECRET}-0123456789")
}

/// Exige qu'aucun des textes ne contienne le marqueur ; nomme le premier
/// fautif.
pub fn aucun_secret<'a>(textes: impl IntoIterator<Item = &'a str>) {
    for t in textes {
        assert!(
            !t.contains(MARQUEUR_SECRET),
            "un secret de prestataire fuit : {t}"
        );
    }
}
