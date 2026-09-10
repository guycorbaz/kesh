//! Inventaire des exemptions de rejeu dont le fondement **se périme**.
//!
//! Lu par `scripts/prepare-release.sh`, qui doit vérifier avant chaque tag
//! qu'aucune version publiée ne se situe dans l'intervalle qu'une telle
//! justification déclare vide.
//!
//! ⛔ **Cet exemple existe pour que le script n'ait RIEN à greper.** Le rappel
//! reposait jusqu'ici sur un marqueur textuel (« SE PÉRIME ») cherché par
//! `grep` dans le source Rust, et sur un `awk` qui en déduisait la version.
//! Quatre modes d'échec en deux passes de revue — citations du marqueur par son
//! propre test, entrée collapsée par `rustfmt`, continuation `\` coupant la
//! chaîne, mutation non couverte — tous imputables au fait qu'un shell lisait du
//! source Rust. Ici, le registre se lit comme la donnée qu'il est.
//!
//! Sortie : une ligne par exemption périssable, `<version> <borne_basse>`.
//! Aucune ligne si le registre n'en compte aucune — le script le gère.
//!
//! *(Story 24-5, #375 — passe 3 de revue de code, findings P3-2 et P3-3.)*

use kesh_db::post_restore::{EXEMPT_MIGRATIONS, ExemptionBasis};

fn main() {
    for (version, basis, _) in EXEMPT_MIGRATIONS {
        if let ExemptionBasis::PerishableSince(borne) = basis {
            println!("{version} {borne}");
        }
    }
}
