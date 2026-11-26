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

/// Sonde de **portée** de M11 : l'extracteur est borné à la fonction nommée. Une
/// fonction voisine du même fichier qui prend `LOCK IN SHARE MODE` **sans**
/// `FORCE INDEX` ne rend pas le motif de `lock_types_by_ids_in_tx` : si
/// l'extracteur rendait les littéraux de tout le fichier, elle le rendrait.
/// (Un `replace` global du motif dans le fichier serait vrai par construction.)
#[test]
fn sonde_negative_l_extracteur_est_borne_a_la_fonction() {
    let source = lire(ACCOUNTS);
    let voisine = litteraux_de(&source, "type_in_share_mode");
    assert!(
        voisine.contains("LOCK IN SHARE MODE"),
        "la sonde doit viser une fonction qui verrouille en partage"
    );
    assert!(
        !voisine.contains("FORCE INDEX (PRIMARY)"),
        "l'extracteur déborde de la fonction nommée"
    );
    let requete = litteraux_de(&source, LECTURE);
    assert!(requete.contains("FORCE INDEX (PRIMARY)"));
}

/// Le corps **masqué** (commentaires et chaînes vides) de la fonction `nom`.
fn corps_masque(source: &str, nom: &str) -> String {
    let (_, masque) = decouper(source);
    let (_, de, a) = fonctions(&masque)
        .into_iter()
        .find(|(n, _, _)| n == nom)
        .unwrap_or_else(|| panic!("fonction {nom} introuvable"));
    masque[de..a].to_string()
}

/// `(position de la lecture des types, position de count_by_company)` dans le
/// corps masqué de `create_opening_entry`, ou `None` si l'un manque.
fn ordre_lecture_et_recensement(source: &str) -> Option<(usize, usize)> {
    let corps = corps_masque(source, "create_opening_entry");
    Some((
        corps.find(&format!("{LECTURE}("))?,
        corps.find("count_by_company(")?,
    ))
}

/// **M10** — la lecture verrouillante des types précède `count_by_company`
/// (première lecture cohérente de la transaction : lue après, elle s'exécuterait
/// vue ouverte, et rendrait `ER_CHECKREAD` (1020) sous `innodb_snapshot_isolation`,
/// MariaDB >= 11.6, #669 — sans effet visible sous 10.11, d'où la garde lexicale).
///
/// ⚠️ **Limite** : elle ne connaît que les **noms** des deux appels. Une autre
/// lecture ordinaire ajoutée avant, sous un autre nom, lui échappe : c'est à la
/// revue de code de la voir.
#[test]
fn la_lecture_des_types_precede_le_recensement() {
    let (lecture, recensement) = ordre_lecture_et_recensement(&lire(JOURNAL))
        .expect("create_opening_entry appelle la lecture et le recensement");
    assert!(
        lecture < recensement,
        "la lecture des types suit count_by_company : la vue naîtrait avant le verrou (#669)"
    );
}

/// Sonde négative de M10 : sur une copie du texte réel où l'appel est **déplacé**
/// après `count_by_company` (et non supprimé : il est toujours présent), la
/// garde voit l'ordre fautif.
#[test]
fn sonde_negative_l_appel_deplace_apres_le_recensement_rougit() {
    let source = lire(JOURNAL);
    let appel = format!(
        "    let types = super::accounts::{LECTURE}(&mut tx, company_id, &account_ids).await?;\n"
    );
    assert_eq!(
        source.matches(&appel).count(),
        1,
        "appel introuvable ou ambigu"
    );
    let ancre = "    // Étape 4 : statut vivant du premier exercice";
    assert_eq!(
        source.matches(ancre).count(),
        1,
        "ancre introuvable ou ambiguë"
    );
    let deplacee = source
        .replacen(&appel, "", 1)
        .replacen(ancre, &format!("{appel}{ancre}"), 1);
    assert_ne!(source, deplacee, "la sonde n'a rien déplacé");
    let (lecture, recensement) = ordre_lecture_et_recensement(&deplacee)
        .expect("l'appel est toujours présent dans le corps, seule sa place change");
    assert!(
        lecture > recensement,
        "la copie déplacée doit montrer l'ordre fautif"
    );
}
