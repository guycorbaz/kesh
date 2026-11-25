//! Story 53-3 (#513) — garde lexicale : le contrôle du type des comptes d'une
//! écriture d'ouverture se fait **sous verrou, dans `create_opening_entry`**, et
//! le handler `POST /opening-balances` ne le lit plus.
//!
//! L'analyse est celle de `tests/lexique/mod.rs` : sur le **texte masqué**
//! (commentaires, chaînes et blocs de test exclus), sans quoi le doc-comment de
//! `create_opening_entry`, qui nomme la fonction, ferait passer la garde à vide
//! après suppression de l'appel.
//!
//! **Limite écrite** : lexical, un renommage en alias le contourne ; la garde
//! protège le geste, non l'intention. L'ordre de verrouillage (`ORDER BY id`) et
//! l'index (`FORCE INDEX (PRIMARY)`) ne sont observés par aucun test de
//! comportement (le plan de la base de test reste `PRIMARY`) : cette garde en
//! exige les motifs dans les **littéraux** de la requête.

mod lexique;
use lexique::{blocs_de_test, decouper, fonction_englobante, fonctions, racine_crates};

const LECTURE: &str = "lock_types_by_ids_in_tx";
const ANCIEN_NOM: &str = "find_types_by_ids_in_tx";

fn lire(relatif: &str) -> String {
    let chemin = racine_crates().join(relatif);
    std::fs::read_to_string(&chemin)
        .unwrap_or_else(|e| panic!("source illisible {}: {e}", chemin.display()))
}

/// Les fonctions de production (blocs de test exclus) qui **appellent** `nom(`.
fn appelants(source: &str, nom: &str) -> Vec<String> {
    let (_, masque) = decouper(source);
    let tests = blocs_de_test(&masque);
    let fns = fonctions(&masque);
    let motif = format!("{nom}(");
    let mut out = Vec::new();
    let mut depuis = 0;
    while let Some(n) = masque[depuis..].find(&motif) {
        let pos = depuis + n;
        depuis = pos + motif.len();
        let avant = masque[..pos].chars().last();
        if avant.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_') {
            continue;
        }
        if masque[..pos].trim_end().ends_with("fn") {
            continue;
        }
        if tests.iter().any(|(de, a)| pos >= *de && pos <= *a) {
            continue;
        }
        if let Some((f, _, _)) = fonction_englobante(&fns, pos) {
            out.push(f.clone());
        }
    }
    out
}

/// Les littéraux de chaîne du corps de la fonction `nom`, concaténés.
fn litteraux_de(source: &str, nom: &str) -> String {
    let (litteraux, masque) = decouper(source);
    let (_, de, a) = fonctions(&masque)
        .into_iter()
        .find(|(n, _, _)| n == nom)
        .unwrap_or_else(|| panic!("fonction {nom} introuvable"));
    litteraux
        .iter()
        .filter(|l| l.debut >= de && l.debut <= a)
        .map(|l| l.texte.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

const JOURNAL: &str = "kesh-db/src/repositories/journal_entries.rs";
const ACCOUNTS: &str = "kesh-db/src/repositories/accounts.rs";
const HANDLER: &str = "kesh-api/src/routes/opening_balances.rs";

#[test]
fn create_opening_entry_lit_les_types_sous_verrou() {
    let appelants = appelants(&lire(JOURNAL), LECTURE);
    assert!(
        appelants.iter().any(|f| f == "create_opening_entry"),
        "create_opening_entry doit appeler {LECTURE} (appelants vus : {appelants:?})"
    );
}

#[test]
fn le_handler_ne_lit_plus_le_type() {
    let source = lire(HANDLER);
    for nom in [LECTURE, ANCIEN_NOM] {
        let vus = appelants(&source, nom);
        assert!(
            vus.is_empty(),
            "le handler ne doit pas appeler {nom} : {vus:?}"
        );
    }
}

/// Sonde positive : les sources sont lues et le détecteur voit les fonctions.
#[test]
fn sonde_positive_les_sources_sont_lues() {
    let (_, masque) = decouper(&lire(JOURNAL));
    assert!(
        fonctions(&masque)
            .iter()
            .any(|(n, _, _)| n == "create_opening_entry"),
        "create_opening_entry introuvable : le détecteur est aveugle"
    );
    let (_, masque) = decouper(&lire(HANDLER));
    assert!(
        fonctions(&masque)
            .iter()
            .any(|(n, _, _)| n == "generate_opening_balances"),
        "le handler est introuvable : le détecteur est aveugle"
    );
}

/// Sonde négative : un appel retiré (le doc-comment, lui, nomme toujours la
/// fonction) fait rougir la garde.
#[test]
fn sonde_negative_un_appel_retire_rougit() {
    let source = lire(JOURNAL);
    let sans_appel = source.replace(&format!("{LECTURE}(&mut tx"), "types_retires(&mut tx");
    assert_ne!(source, sans_appel, "la sonde n'a rien retiré");
    assert!(
        sans_appel.contains(LECTURE),
        "le doc-comment doit encore nommer la fonction (sinon la sonde ne prouve rien)"
    );
    assert!(
        !appelants(&sans_appel, LECTURE)
            .iter()
            .any(|f| f == "create_opening_entry"),
        "la garde ne doit plus passer sans l'appel"
    );
}

/// M11 : la requête épingle l'index et l'ordre de verrouillage, et prend le
/// verrou partagé.
#[test]
fn la_requete_epingle_index_ordre_et_verrou() {
    let requete = litteraux_de(&lire(ACCOUNTS), LECTURE);
    for motif in ["FORCE INDEX (PRIMARY)", "ORDER BY id", "LOCK IN SHARE MODE"] {
        assert!(
            requete.contains(motif),
            "motif absent de la requête : {motif}"
        );
    }
    assert!(
        !requete.contains("FOR SHARE"),
        "`FOR SHARE` est une erreur de syntaxe sur MariaDB 10.11"
    );
}

/// Sonde négative de M11 : sans `FORCE INDEX`, la garde rougit.
#[test]
fn sonde_negative_sans_force_index_rougit() {
    let source = lire(ACCOUNTS).replace("FORCE INDEX (PRIMARY)", "");
    let requete = litteraux_de(&source, LECTURE);
    assert!(!requete.contains("FORCE INDEX (PRIMARY)"));
}
