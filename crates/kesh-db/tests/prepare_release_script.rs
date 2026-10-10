//! Enveloppe le test shell de `scripts/prepare-release.sh` pour qu'il tourne au
//! gate (`cargo test` / nextest) et en CI.
//!
//! Le test lui-même vit dans `scripts/tests/prepare-release.test.sh` : il éprouve
//! le script sur des dépôts git **jetables** (`mktemp -d`), avec un `cargo`
//! factice en tête du PATH — jamais sur le dépôt réel, que le script modifierait
//! (bump des crates, datation du CHANGELOG).
//!
//! Il loge ici, dans `kesh-db`, parce que c'est ce crate qui porte l'inventaire
//! que le pré-vol du script lit (`examples/perishable_exemptions.rs`, registre
//! `EXEMPT_MIGRATIONS`). Un test shell que rien ne lance serait un test muet.
//!
//! *(Issue #566 : le script refusait une version déjà bumpée et sautait alors
//! son pré-vol.)*

use std::path::Path;
use std::process::Command;

#[test]
fn prepare_release_script_runs_preflight_even_when_already_bumped() {
    let test_script =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/tests/prepare-release.test.sh");
    assert!(
        test_script.is_file(),
        "test shell introuvable : {}",
        test_script.display()
    );

    let output = Command::new("bash")
        .arg(&test_script)
        .output()
        .expect("impossible de lancer bash");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "le test de prepare-release.sh a échoué ({}):\n{stdout}\n{stderr}",
        output.status
    );
    // Garde contre un test muet : le résumé final doit avoir été atteint.
    assert!(
        stdout.contains("aucune assertion en échec"),
        "le test shell n'a pas rendu son résumé :\n{stdout}\n{stderr}"
    );
}
