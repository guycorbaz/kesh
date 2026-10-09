//! Lancement du binaire `kesh-api` dans un environnement maîtrisé — Story
//! 15-13a (factorisé de `configuration_transmise.rs`).
//!
//! Inclus par `#[path = "common/binaire.rs"] mod binaire;` — **pas** par
//! `mod common;`, qui tirerait l'amorce de base de `common/mod.rs` dans des
//! binaires de test qui ne l'emploient pas.
//!
//! Le helper ne fait **aucune** assertion : chaque appelant pose la sienne
//! (code de sortie, contenu de la sortie).

/// Lance `kesh-api` dans un répertoire temporaire (aucun `.env` à charger),
/// avec un environnement **vidé** (`env_clear`), `NO_COLOR=1`, puis les
/// couples `env` dans l'ordre donné. Attend la fin du processus.
pub fn lancer_binaire(env: &[(&str, &str)]) -> std::process::Output {
    let dir = tempfile::tempdir().expect("répertoire temporaire");
    let mut commande = std::process::Command::new(env!("CARGO_BIN_EXE_kesh-api"));
    commande
        .current_dir(dir.path())
        .env_clear()
        .env("NO_COLOR", "1");
    for (cle, valeur) in env {
        commande.env(cle, valeur);
    }
    commande.output().expect("lancement du binaire kesh-api")
}

/// Sortie complète d'un lancement (stdout puis stderr), en texte.
pub fn texte(sortie: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&sortie.stdout),
        String::from_utf8_lossy(&sortie.stderr)
    )
}
