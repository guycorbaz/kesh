//! Story 37-3a — deux gardes lexicales sur les sources de production
//! (`crates/*/src/**/*.rs`, hors blocs `#[cfg(test)]`) :
//!
//! 1. le constructeur de test `kesh_db::test_fixtures::chf_line` n'est appelé
//!    que depuis `test_fixtures.rs` : en production, chaque construction d'une
//!    `NewJournalEntryLine` dit ce qu'elle pose (`foreign` compris, sans
//!    `Default`) ;
//! 2. la variante `journal_entries::create_opening_in_tx` — qui exempte
//!    l'écriture de la règle d'une seule devise étrangère (AC-4, rang 2 bis) —
//!    n'a que ses deux appelants : `create_opening_entry` et le complément
//!    d'ouverture (`opening_complement.rs`).
//!
//! L'analyse (littéraux, commentaires et blocs de test masqués) est celle de
//! `tests/lexique/mod.rs`.

mod lexique;
use lexique::{blocs_de_test, decouper, fonction_englobante, fonctions, racine_crates, sources};

/// Les appels de `nom(` dans le code de production (commentaires, chaînes et
/// blocs de test exclus) : `(fichier relatif, fonction englobante)`.
fn appels(nom: &str) -> Vec<(String, String)> {
    let racine = racine_crates();
    let mut fichiers = Vec::new();
    for entree in std::fs::read_dir(&racine).unwrap() {
        let src = entree.unwrap().path().join("src");
        if src.is_dir() {
            sources(&src, &mut fichiers);
        }
    }
    let motif = format!("{nom}(");
    let mut out = Vec::new();
    for f in fichiers {
        let source = std::fs::read_to_string(&f).unwrap();
        let (_, masque) = decouper(&source);
        let tests = blocs_de_test(&masque);
        let fns = fonctions(&masque);
        let mut depuis = 0;
        while let Some(n) = masque[depuis..].find(&motif) {
            let pos = depuis + n;
            depuis = pos + motif.len();
            let avant = masque[..pos].chars().last();
            if avant.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_') {
                continue;
            }
            if masque[..pos].trim_end().ends_with("fn") {
                continue; // la définition elle-même
            }
            if tests.iter().any(|(de, a)| pos >= *de && pos <= *a) {
                continue;
            }
            let englobante = fonction_englobante(&fns, pos)
                .map(|(n, _, _)| n.clone())
                .unwrap_or_default();
            let relatif = f
                .strip_prefix(&racine)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            out.push((relatif, englobante));
        }
    }
    out.sort();
    out
}

#[test]
fn chf_line_n_est_jamais_appele_en_production() {
    let hors_fixtures: Vec<_> = appels("chf_line")
        .into_iter()
        .filter(|(f, _)| f != "kesh-db/src/test_fixtures.rs")
        .collect();
    assert!(
        hors_fixtures.is_empty(),
        "chf_line est un constructeur de test : {hors_fixtures:?}"
    );
}

#[test]
fn create_opening_in_tx_n_a_que_ses_deux_appelants() {
    assert_eq!(
        appels("create_opening_in_tx"),
        vec![
            (
                "kesh-db/src/repositories/journal_entries.rs".to_string(),
                "create_opening_entry".to_string()
            ),
            (
                "kesh-db/src/repositories/opening_complement.rs".to_string(),
                "create_opening_complement_in_tx".to_string()
            ),
        ],
        "la variante d'ouverture exempte de la règle d'une devise : ses appelants sont clos"
    );
}

/// Anti-muet : le détecteur trouve bien un appel connu.
#[test]
fn le_detecteur_voit_un_appel_connu() {
    let vus = appels("validate_lines_accounts_in_tx");
    assert_eq!(vus.len(), 2, "{vus:?}");
}
