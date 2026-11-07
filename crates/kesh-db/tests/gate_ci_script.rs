//! Enveloppe le test shell de `scripts/gate-ci.sh`, pour qu'il tourne au gate
//! (`cargo test` / nextest) et en CI ; un test shell que rien ne lance serait muet.
//!
//! Le test vit dans `scripts/tests/gate-ci.test.sh` : mini-dépôts jetables, `docker` et
//! `cargo` factices, jamais les vrais. Il garde aussi l'égalité entre le script et
//! `.github/workflows/ci.yml` (la commande `cargo test`, la boucle des migrations).
//! *(Story 33-10, #925.)*

use std::path::Path;
use std::process::Command;

#[test]
fn gate_ci_script_rejoue_la_commande_de_la_ci_dans_un_environnement_ferme() {
    let test_script =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/tests/gate-ci.test.sh");
    assert!(
        test_script.is_file(),
        "test shell introuvable : {}",
        test_script.display()
    );

    // Borné en mémoire et en durée : le 2026-11-07, un cas de ce test a pris 27 Go et fait
    // tomber la station, hors de tout plafond (C-e334-38). En CI, aucun autre ne s'applique.
    let output = Command::new("bash")
        .arg("-c")
        .arg(r#"ulimit -v 4000000 && exec timeout -k 10 600 bash "$0""#)
        .arg(&test_script)
        .env_remove("KESH_DOCKER")
        .env_remove("KESH_CARGO")
        .env_remove("KESH_MEMGUARD_ACTIVE")
        .output()
        .expect("impossible de lancer bash");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "le test de gate-ci.sh a échoué ({}):\n{stdout}\n{stderr}",
        output.status
    );
    // Garde contre un test muet : le résumé final doit avoir été atteint.
    assert!(
        stdout.contains("aucune assertion en échec"),
        "le test shell n'a pas rendu son résumé :\n{stdout}\n{stderr}"
    );
}
