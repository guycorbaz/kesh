//! Story 63-1 (#1752, #1753, #1754, AC-10) — garde lexicale : en production,
//! **une seule** annulation construit encore `CancellationDating::Today(…)` —
//! `journal_entries::reverse` (`POST /journal-entries/{id}/reverse`, R10 de la
//! fiche), la correction d'une écriture non possédée depuis le présent. Toute
//! autre annulation (dé-rapprochement, gestes depuis la fiche d'une facture,
//! d'un paiement, d'une facture fournisseur) est datée de l'écriture d'origine
//! (`CancellationDating::OriginDate`). Un site neuf qui daterait du jour rougit
//! ce test.
//!
//! **Forme (validation P2 de la fiche)** : la garde ne distingue pas
//! construction et motif par la syntaxe (un bras `X => CancellationDating::Today(j),`
//! construit, et porte un `=>`). Elle tient l'**inventaire clos de toutes les
//! occurrences** du jeton `CancellationDating::Today` dans le texte masqué de
//! `tests/lexique/mod.rs` (commentaires, chaînes et blocs `#[cfg(test)]`
//! exclus), chacune rattachée à sa fonction englobante, en **égalité stricte**
//! avec [`ATTENDU`] : une construction (R10) et deux motifs (la conversion
//! `From<CancellationDating> for ReversalDating` et le bras `Today` de la queue
//! commune, conservés par D7 de la fiche ; issue #1759 pour leur retrait).
//!
//! **Contournements fermés** : un alias de l'énumération (`CancellationDating as D`,
//! `type D = CancellationDating;`), une importation de ses variantes
//! (`CancellationDating::{…}`, `CancellationDating::*`, `use …CancellationDating::Today`,
//! cette dernière comptée comme occurrence hors de toute fonction), un bloc
//! `impl CancellationDating` (où `Self::Today` échapperait au jeton) — aucun
//! n'est admis en production. Les espaces autour de `::` sont tolérés.
//!
//! **Angles morts écrits** :
//! - un `#[cfg(test)] mod x;` **sans accolades** fait masquer par
//!   `blocs_de_test` le code de production jusqu'à la fin du premier bloc qui
//!   suit (`bloc_apres` prend la première `{`) — défaut de l'outil commun,
//!   issue **#1760** ; à la date de la story, `kesh-report/src/lib.rs` masque
//!   ainsi des `pub mod`/`pub use`, sans jeton ;
//! - une macro qui engendrerait le jeton (`concat_idents!`, macro procédurale) ;
//! - une annulation **future** qui créerait son inverse par `create_in_tx`, daté
//!   du jour, **sans** `CancellationDating` : aucun jeton, donc aucun relevé.
//!   Aucun site de ce genre aujourd'hui — `reverses_entry_id` n'est posé que par
//!   `reverse_in_tx_inner` — mais rien ne l'interdit par la syntaxe (revue P1
//!   de la 63-1, L6) ;
//! - un chemin qui ne nomme pas l'énumération (`use journal_entries as j;` puis
//!   `j::CancellationDating::Today` reste vu ; un `pub use` qui la renommerait
//!   est fermé par la règle de l'alias).

mod lexique;
use lexique::{blocs_de_test, decouper, fonction_englobante, fonctions, racine_crates, sources};

const ENUM: &str = "CancellationDating";

/// L'inventaire clos : `(fichier relatif à crates/, fonction englobante,
/// nombre d'occurrences)`.
const ATTENDU: &[(&str, &str, usize)] = &[
    // R10 — `POST /journal-entries/{id}/reverse` : la seule construction.
    ("kesh-db/src/repositories/journal_entries.rs", "reverse", 1),
    // Motif : `From<CancellationDating> for ReversalDating`.
    ("kesh-db/src/repositories/journal_entries.rs", "from", 1),
    // Motif : bras `Today` de la queue commune (rang 5, inatteignable, D7).
    (
        "kesh-db/src/repositories/settlement_cancellation.rs",
        "settlement_entry_cancel_blocker",
        1,
    ),
];

fn est_ident(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Ce qu'une source dit de `CancellationDating` : les fonctions englobantes des
/// occurrences de `CancellationDating::Today` (production seule ; `"<hors
/// fonction>"` hors de tout corps), et les contournements vus.
fn analyser(source: &str) -> (Vec<String>, Vec<String>) {
    let (_, masque) = decouper(source);
    let tests = blocs_de_test(&masque);
    let fns = fonctions(&masque);
    let mut occurrences = Vec::new();
    let mut contournements = Vec::new();
    let mut depuis = 0;
    while let Some(n) = masque[depuis..].find(ENUM) {
        let pos = depuis + n;
        depuis = pos + ENUM.len();
        let avant = masque[..pos].chars().next_back();
        let apres = masque[pos + ENUM.len()..].chars().next();
        if avant.is_some_and(est_ident) || apres.is_some_and(est_ident) {
            continue;
        }
        if tests.iter().any(|(de, a)| pos >= *de && pos <= *a) {
            continue;
        }
        // L'instruction qui précède l'occurrence, depuis le dernier `;`, `{`
        // ou `}` : de quoi reconnaître `type D = …` et `impl … {`.
        let debut_instr = masque[..pos]
            .rfind([';', '{', '}'])
            .map(|i| i + 1)
            .unwrap_or(0);
        let instr = &masque[debut_instr..pos];
        let mots: Vec<&str> = instr.split(|c: char| !est_ident(c)).collect();
        let suite = masque[pos + ENUM.len()..].trim_start();
        if suite.starts_with("as") && !suite[2..].starts_with(est_ident) {
            contournements.push(format!("alias `{ENUM} as …` : {}", extrait(&masque, pos)));
            continue;
        }
        if mots.contains(&"type") {
            contournements.push(format!(
                "alias `type … = {ENUM}` : {}",
                extrait(&masque, pos)
            ));
            continue;
        }
        // `impl CancellationDating {`, `impl Trait for CancellationDating {` :
        // l'énumération est la cible du bloc (suivie de `{` ou de ses
        // paramètres) ; `impl From<CancellationDating> for …` ne l'est pas.
        if mots.contains(&"impl") && (suite.starts_with('{') || suite.starts_with('<')) {
            contournements.push(format!("bloc `impl {ENUM}` : {}", extrait(&masque, pos)));
            continue;
        }
        let Some(chemin) = suite.strip_prefix("::") else {
            continue;
        };
        let chemin = chemin.trim_start();
        if chemin.starts_with('{') || chemin.starts_with('*') {
            contournements.push(format!(
                "importation des variantes `{ENUM}::{{…}}` ou `::*` : {}",
                extrait(&masque, pos)
            ));
            continue;
        }
        let variante: String = chemin.chars().take_while(|c| est_ident(*c)).collect();
        if variante != "Today" {
            continue;
        }
        let f = fonction_englobante(&fns, pos)
            .map(|(f, _, _)| f.clone())
            .unwrap_or_else(|| "<hors fonction>".into());
        occurrences.push(f);
    }
    (occurrences, contournements)
}

fn extrait(masque: &str, pos: usize) -> String {
    let de = masque[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let a = masque[pos..]
        .find('\n')
        .map(|i| pos + i)
        .unwrap_or(masque.len());
    masque[de..a].trim().to_string()
}

#[test]
fn cancellation_dating_today_n_est_construit_que_par_reverse() {
    let racine = racine_crates();
    let mut fichiers = Vec::new();
    for entree in std::fs::read_dir(&racine).expect("crates/") {
        let src = entree.expect("entrée").path().join("src");
        if src.is_dir() {
            sources(&src, &mut fichiers);
        }
    }
    fichiers.sort();
    assert!(
        fichiers.len() > 100,
        "sonde : les sources de production sont lues ({} fichiers)",
        fichiers.len()
    );

    let mut vu: Vec<(String, String, usize)> = Vec::new();
    let mut contournements = Vec::new();
    for chemin in &fichiers {
        let source = std::fs::read_to_string(chemin).expect("source lisible");
        let relatif = chemin
            .strip_prefix(&racine)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let (occurrences, c) = analyser(&source);
        contournements.extend(c.into_iter().map(|c| format!("{relatif} : {c}")));
        for f in occurrences {
            match vu.iter_mut().find(|(r, g, _)| *r == relatif && *g == f) {
                Some(e) => e.2 += 1,
                None => vu.push((relatif.clone(), f, 1)),
            }
        }
    }
    vu.sort();
    let mut attendu: Vec<(String, String, usize)> = ATTENDU
        .iter()
        .map(|(r, f, n)| (r.to_string(), f.to_string(), *n))
        .collect();
    attendu.sort();

    // Sonde positive : R10 DOIT être trouvé — sans quoi la garde passerait à
    // vide (troncature, chemin faux, masquage excessif).
    assert!(
        vu.iter()
            .any(|(r, f, n)| r.ends_with("journal_entries.rs") && f == "reverse" && *n == 1),
        "sonde : la construction de `reverse` (R10) n'est pas vue — la garde lit-elle les sources ? {vu:?}"
    );
    assert!(
        contournements.is_empty(),
        "contournement de la garde en production : {contournements:#?}"
    );
    assert_eq!(
        vu, attendu,
        "inventaire de `CancellationDating::Today` en production : seul \
         `journal_entries::reverse` (R10) construit ; toute autre annulation est \
         datée de l'origine (Story 63-1, #1752, #1753, #1754)"
    );
}

/// Échantillons synthétiques : chacun doit être vu par l'analyse (sinon la
/// garde serait aveugle à cette forme).
#[test]
fn l_analyse_voit_les_formes_qui_construisent_et_les_contournements() {
    // Un argument sur plusieurs lignes, chemin qualifié, espaces autour de `::`.
    let s = "fn geste() {\n    reverse_owned_in_tx(\n        tx,\n        \
             super::journal_entries::CancellationDating :: Today(\n            aujourd_hui,\n        ),\n    );\n}\n";
    assert_eq!(analyser(s), (vec!["geste".to_string()], vec![]));

    // Un bras `=>` qui construit.
    let s = "fn choisir(x: bool, j: NaiveDate) -> CancellationDating {\n    match x {\n        \
             true => CancellationDating::Today(j),\n        false => CancellationDating::OriginDate,\n    }\n}\n";
    assert_eq!(analyser(s), (vec!["choisir".to_string()], vec![]));

    // Un alias : `use … as D; D::Today(j)`.
    let s = "use super::journal_entries::CancellationDating as D;\nfn g(j: NaiveDate) { h(D::Today(j)); }\n";
    let (occ, c) = analyser(s);
    assert!(occ.is_empty() && c.len() == 1, "{occ:?} {c:?}");

    // `type D = CancellationDating;`
    let s = "type D = super::journal_entries::CancellationDating;\nfn g(j: NaiveDate) { h(D::Today(j)); }\n";
    assert_eq!(analyser(s).1.len(), 1);

    // Importation des variantes.
    let s = "use super::journal_entries::CancellationDating::{Today, OriginDate};\nfn g(j: NaiveDate) { h(Today(j)); }\n";
    assert_eq!(analyser(s).1.len(), 1);
    let s = "use super::journal_entries::CancellationDating::*;\n";
    assert_eq!(analyser(s).1.len(), 1);

    // `use …CancellationDating::Today;` : une occurrence hors fonction.
    let s = "use super::journal_entries::CancellationDating::Today;\nfn g(j: NaiveDate) { h(Today(j)); }\n";
    assert_eq!(analyser(s), (vec!["<hors fonction>".to_string()], vec![]));

    // `impl CancellationDating { … Self::Today(…) }`.
    let s =
        "impl CancellationDating {\n    fn du_jour(j: NaiveDate) -> Self { Self::Today(j) }\n}\n";
    assert_eq!(analyser(s).1.len(), 1);

    // Ce qui ne compte pas : un commentaire, une chaîne, un bloc de test, un
    // autre identifiant qui contient le nom, la conversion `From<…> for …`.
    let s = "// CancellationDating::Today(j)\n/// CancellationDating::Today(j)\n\
             fn a() { let _ = \"CancellationDating::Today(j)\"; }\n\
             #[cfg(test)]\nmod tests {\n    fn t() { CancellationDating::Today(j); }\n}\n\
             fn b() { MyCancellationDating::Today(j); }\n\
             impl From<CancellationDating> for ReversalDating {\n    fn from(d: CancellationDating) -> Self { match d { _ => Self::OriginDate } }\n}\n";
    assert_eq!(analyser(s), (vec![], vec![]));
}
