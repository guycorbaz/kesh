//! Story 32-10a-i (refs #723) — garde `amorcage_au_demarrage` (AC-i11) : le
//! démarrage installe les prompts standard du serveur MCP, **après** le
//! bootstrap qui pose la société provisoire d'une base vide, et **avant** que
//! le serveur ne réponde.
//!
//! C'est le seul point d'entrée qu'aucun test n'exécute (`main.rs` n'est monté
//! par aucun test : `build_router` l'est) ; les autres — `set_language`, la
//! remise à zéro, `_test/seed`, `_test/reset` — sont couverts par des tests
//! d'exécution (`onboarding_audit_e2e.rs`, `test_endpoints_e2e.rs`). Patron
//! de `auto_backup::tests::lancement_par_main_seulement`.
//!
//! ⚠️ **Limite assumée** de [`code`] : lecture ligne à ligne ; seules les
//! lignes entières commençant par `//` sont écartées, et la lecture s'arrête à
//! la première ligne `#[cfg(test)]` — `main.rs` n'en porte aucune (assertion
//! ci-dessous), la troncature y est donc sans effet.

const MAIN: &str = include_str!("../src/main.rs");

/// Lignes de code d'un fichier, commentaires `//` écartés, tronqué au premier
/// `#[cfg(test)]`.
fn code(source: &str) -> String {
    source
        .lines()
        .take_while(|l| l.trim() != "#[cfg(test)]")
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// La position de l'unique occurrence de `jeton` dans `texte`, ou un échec
/// qui dit combien il y en a.
fn unique(texte: &str, jeton: &str) -> usize {
    let n = texte.matches(jeton).count();
    assert_eq!(
        n, 1,
        "« {jeton} » doit apparaître une fois dans main.rs, vu {n}"
    );
    texte.find(jeton).unwrap()
}

/// Vrai si `appel` est **strictement entre** `avant` et `apres` (ancres
/// uniques) dans `texte`.
fn entre(texte: &str, avant: &str, appel: &str, apres: &str) -> bool {
    let (a, x, b) = (
        unique(texte, avant),
        unique(texte, appel),
        unique(texte, apres),
    );
    a < x && x < b
}

#[test]
fn amorcage_au_demarrage() {
    assert_eq!(
        MAIN.lines().filter(|l| l.trim() == "#[cfg(test)]").count(),
        0
    );
    let main = code(MAIN);
    let pos = unique(&main, "amorcer_standards(");
    // L'appel porte les standards du code, pas une liste de test.
    let fin = main[pos..].find(')').unwrap();
    assert!(
        main[pos..pos + fin].contains("STANDARDS"),
        "l'amorçage du démarrage doit porter STANDARDS"
    );
    assert!(
        entre(
            &main,
            "ensure_admin_user(",
            "amorcer_standards(",
            "build_router("
        ),
        "l'amorçage doit suivre le bootstrap et précéder le routeur"
    );
}

#[test]
fn amorcage_au_demarrage_temoins() {
    // Témoin vert existant de la fonction d'ordre : la sauvegarde automatique
    // est lancée entre le bootstrap et le routeur.
    assert!(entre(
        &code(MAIN),
        "ensure_admin_user(",
        "auto_backup::spawn(",
        "build_router("
    ));
    // Témoin rouge factice : un appel placé avant le bootstrap.
    let factice = "fn main() {\n    amorcer_standards(&pool, STANDARDS);\n    \
                   ensure_admin_user(&pool);\n    build_router(s);\n}";
    assert!(!entre(
        &code(factice),
        "ensure_admin_user(",
        "amorcer_standards(",
        "build_router("
    ));
    // Témoin de la troncature des commentaires, factice dit tel.
    assert_eq!(
        code("// amorcer_standards(&pool, STANDARDS)\nfn f() {}")
            .matches("amorcer_standards(")
            .count(),
        0
    );
}
