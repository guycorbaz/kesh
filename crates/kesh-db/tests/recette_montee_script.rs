//! Enveloppe le test shell de `scripts/recette-montee.sh`, pour qu'il tourne au gate
//! (`cargo test` / nextest) et en CI ; un test shell que rien ne lance serait muet.
//!
//! Le test vit dans `scripts/tests/recette-montee.test.sh` : un mini-dépôt jetable,
//! `docker`, `curl` et un binaire factices. Il ne lance ni conteneur ni serveur.
//! *(Story 33-10, #925.)*

use std::path::Path;
use std::process::Command;

#[test]
fn recette_montee_refuse_ce_qui_sort_de_son_perimetre_et_enchaine_ses_phases() {
    let test_script =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/tests/recette-montee.test.sh");
    assert!(
        test_script.is_file(),
        "test shell introuvable : {}",
        test_script.display()
    );

    // Borné en mémoire et en durée, comme celui de `gate-ci.sh` (C-e334-54) : un processus de
    // fond resté vivant ou une relecture sans fin ne fige ni n'épuise la station.
    let output = Command::new("bash")
        .arg("-c")
        .arg(r#"ulimit -v 4000000 && exec timeout -k 10 600 bash "$0""#)
        .arg(&test_script)
        .env_remove("KESH_DOCKER")
        .env_remove("KESH_CURL")
        .env_remove("RECETTE_RACINE_JETABLE")
        .output()
        .expect("impossible de lancer bash");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "le test de recette-montee.sh a échoué ({}):\n{stdout}\n{stderr}",
        output.status
    );
    // Garde contre un test muet : le résumé final doit avoir été atteint.
    assert!(
        stdout.contains("aucune assertion en échec"),
        "le test shell n'a pas rendu son résumé :\n{stdout}\n{stderr}"
    );
}
