//! Enveloppe le test shell du garde des droits de dev, `scripts/check-dev-grants.sh`,
//! pour qu'il tourne au gate (`cargo test` / nextest) et en CI.
//!
//! Le test lui-même vit dans `scripts/tests/check-dev-grants.test.sh` : il éprouve le
//! garde dans des mini-dépôts **jetables** (`mktemp -d`), avec un `docker` factice
//! injecté par `KESH_DOCKER` — jamais le vrai `docker` ni le conteneur
//! `kesh-mariadb-dev`. Il est hermétique : il écrit son propre fichier de droits de
//! référence et ne lit, du dépôt réel, que `scripts/test-fast.sh` (assertion de câblage).
//!
//! Il loge ici, dans `kesh-db`, parce que le garde protège les droits dont dépendent
//! les tests de ce crate (`#[sqlx::test]` crée ses bases éphémères avec eux), et à
//! côté de l'enveloppe de `prepare-release.test.sh`, dont il reprend la forme. Un test
//! shell que rien ne lance serait un test muet.
//!
//! *(Story 31-2, action C3 de la rétrospective de l'Epic 29 : le conteneur de dev
//! n'avait plus les droits du fichier versionné, et rien ne le signalait.)*

use std::path::Path;
use std::process::Command;

#[test]
fn check_dev_grants_script_compares_real_grants_to_versioned_file() {
    let test_script =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/tests/check-dev-grants.test.sh");
    assert!(
        test_script.is_file(),
        "test shell introuvable : {}",
        test_script.display()
    );

    let output = Command::new("bash")
        .arg(&test_script)
        // Le garde ne doit rien hériter de l'appelant (le gate exporte DATABASE_URL,
        // un poste peut porter KESH_DOCKER ou MYSQL_PWD).
        .env_remove("KESH_DOCKER")
        .env_remove("MYSQL_PWD")
        .env_remove("DATABASE_URL")
        .output()
        .expect("impossible de lancer bash");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "le test de check-dev-grants.sh a échoué ({}):\n{stdout}\n{stderr}",
        output.status
    );
    // Garde contre un test muet : le résumé final doit avoir été atteint.
    assert!(
        stdout.contains("aucune assertion en échec"),
        "le test shell n'a pas rendu son résumé :\n{stdout}\n{stderr}"
    );
}
