//! Test structurel — Story 15-1a-i (#518), R3 et R7 point 4 (T9).
//!
//! ⛔ **UNE seule fonction écrit la marque du lettrage**
//! (`letterings::create_group_in_tx`) **et une seule la retire**
//! (`letterings::dissolve_group_in_tx`). Ce test lit les sources de production
//! — `crates/*/src/**/*.rs`, **hors** blocs `#[cfg(test)]` (les tests de dépôt
//! vivent aussi dans des `mod tests` de `src/`) et **hors**
//! `kesh-db/src/repositories/letterings.rs` — et refuse tout littéral de chaîne
//! qui porte le mot `UPDATE`, `INSERT` ou `REPLACE` (toutes formes : `VALUES`,
//! `SELECT`, quel que soit le blanc qui suit le verbe) et
//! qui nomme `lettering_key` ou `lettering_origin`, ou qui interpole
//! `LINE_COLUMNS` (la liste de colonnes des lignes, qui les porte).
//!
//! Les **exceptions** de R3 n'apparaissent pas ici, et c'est voulu : la
//! migration de rattrapage de la 15-1a2 est un fichier `.sql` ; la
//! restauration d'une sauvegarde (`backup.rs`) nomme ses colonnes
//! **dynamiquement** — aucun littéral ne les porte — et rétablit les marques
//! telles qu'exportées ; les suppressions en bloc (`delete_all_by_company`,
//! `reset_demo`) sont des `DELETE`, qui emportent les lignes avec leurs marques.
//!
//! ⚠️ **Ce que ce test ne voit pas** : une requête assemblée de plusieurs
//! littéraux dont aucun, seul, ne porte à la fois le verbe et la colonne, ou
//! une colonne nommée par une constante autre que `LINE_COLUMNS`. La revue
//! reste la garde de ces formes.
//!
//! Second volet (R7 point 4, F4-4) : chacune des deux primitives appelle
//! `check_rows_affected` sur le résultat de son `UPDATE` — c'est la seule garde
//! contre la mutation « retirer la vérification », qu'aucun montage de test ne
//! peut exercer sans crochet de production.

use std::path::{Path, PathBuf};

/// Un littéral de chaîne et sa position (octet de début) dans la source.
struct Litteral {
    debut: usize,
    texte: String,
}

/// Découpe une source Rust : rend ses littéraux de chaîne (ordinaires et
/// bruts), et une copie « masquée » où le contenu des chaînes et des
/// commentaires est remplacé par des espaces — de quoi compter les accolades
/// sans être trompé par un `"{ids}"`.
fn decouper(source: &str) -> (Vec<Litteral>, String) {
    let b = source.as_bytes();
    let mut masque = b.to_vec();
    let mut litteraux = Vec::new();
    let mut i = 0;
    let effacer = |m: &mut Vec<u8>, de: usize, a: usize| {
        for x in m.iter_mut().take(a).skip(de) {
            if *x != b'\n' {
                *x = b' ';
            }
        }
    };
    while i < b.len() {
        match b[i] {
            b'/' if b.get(i + 1) == Some(&b'/') => {
                let fin = source[i..].find('\n').map_or(b.len(), |n| i + n);
                effacer(&mut masque, i, fin);
                i = fin;
            }
            b'/' if b.get(i + 1) == Some(&b'*') => {
                let fin = source[i + 2..]
                    .find("*/")
                    .map_or(b.len(), |n| i + 2 + n + 2);
                effacer(&mut masque, i, fin);
                i = fin;
            }
            b'r' if matches!(b.get(i + 1), Some(b'"') | Some(b'#'))
                && (i == 0 || !(b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_')) =>
            {
                let mut j = i + 1;
                let mut dieses = 0;
                while b.get(j) == Some(&b'#') {
                    dieses += 1;
                    j += 1;
                }
                if b.get(j) != Some(&b'"') {
                    i += 1;
                    continue;
                }
                let fermeture = format!("\"{}", "#".repeat(dieses));
                let debut = j + 1;
                let fin = source[debut..]
                    .find(&fermeture)
                    .map_or(b.len(), |n| debut + n);
                litteraux.push(Litteral {
                    debut: i,
                    texte: source[debut..fin].to_string(),
                });
                effacer(&mut masque, debut, fin);
                i = fin + fermeture.len();
            }
            b'"' => {
                let debut = i + 1;
                let mut j = debut;
                while j < b.len() && b[j] != b'"' {
                    if b[j] == b'\\' {
                        j += 1;
                    }
                    j += 1;
                }
                let fin = j.min(b.len());
                litteraux.push(Litteral {
                    debut: i,
                    texte: source[debut..fin].to_string(),
                });
                effacer(&mut masque, debut, fin);
                i = fin + 1;
            }
            b'\'' => {
                // Caractère (`'"'`, `'\''`, `'{'`) ou durée de vie (`'a`).
                if b.get(i + 1) == Some(&b'\\') {
                    let fin = source[i + 2..].find('\'').map_or(b.len(), |n| i + 2 + n);
                    effacer(&mut masque, i + 1, fin);
                    i = fin + 1;
                } else if let Some(c) = source[i + 1..].chars().next() {
                    let apres = i + 1 + c.len_utf8();
                    if b.get(apres) == Some(&b'\'') {
                        effacer(&mut masque, i + 1, apres);
                        i = apres + 1;
                    } else {
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }
    (litteraux, String::from_utf8(masque).expect("utf8"))
}

/// Rend l'intervalle `[ouvrante, fermante]` du premier bloc `{ … }` qui suit
/// `depuis` dans la source masquée.
fn bloc_apres(masque: &str, depuis: usize) -> Option<(usize, usize)> {
    let ouvrante = depuis + masque[depuis..].find('{')?;
    let mut profondeur = 0usize;
    for (n, c) in masque[ouvrante..].char_indices() {
        match c {
            '{' => profondeur += 1,
            '}' => {
                profondeur -= 1;
                if profondeur == 0 {
                    return Some((ouvrante, ouvrante + n));
                }
            }
            _ => {}
        }
    }
    None
}

/// Les intervalles des blocs `#[cfg(test)]` (module ou item à accolades).
fn blocs_de_test(masque: &str) -> Vec<(usize, usize)> {
    let mut blocs = Vec::new();
    let mut depuis = 0;
    while let Some(n) = masque[depuis..].find("#[cfg(test)]") {
        let pos = depuis + n;
        if let Some(bloc) = bloc_apres(masque, pos) {
            blocs.push((pos, bloc.1));
            depuis = bloc.1;
        } else {
            break;
        }
    }
    blocs
}

/// Les écritures de la marque hors de la primitive : `(rang du littéral, extrait)`.
fn ecritures_de_la_marque(source: &str) -> Vec<String> {
    let (litteraux, masque) = decouper(source);
    let tests = blocs_de_test(&masque);
    litteraux
        .into_iter()
        .filter(|l| !tests.iter().any(|(de, a)| l.debut >= *de && l.debut <= *a))
        .filter(|l| {
            // Un verbe d'écriture est un MOT du littéral, quel que soit le
            // blanc qui le suit (espace, tabulation, saut de ligne, `\n`
            // échappé) : chercher large (P7). `REPLACE` en est un (revue P1,
            // B6). Seuls `FOR UPDATE` (lecture verrouillante) et `ON UPDATE`
            // (DDL) ne sont pas des écritures ; `ON DUPLICATE KEY UPDATE` en
            // reste une.
            let mots: Vec<&str> = l
                .texte
                .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .filter(|m| !m.is_empty())
                .collect();
            let ecrit = mots.iter().enumerate().any(|(n, mot)| {
                let verbe = ["UPDATE", "INSERT", "REPLACE"]
                    .iter()
                    .any(|v| mot.eq_ignore_ascii_case(v));
                let lecture_ou_ddl = mot.eq_ignore_ascii_case("UPDATE")
                    && n > 0
                    && (mots[n - 1].eq_ignore_ascii_case("FOR")
                        || mots[n - 1].eq_ignore_ascii_case("ON"));
                verbe && !lecture_ou_ddl
            });
            let nomme = l.texte.contains("lettering_key")
                || l.texte.contains("lettering_origin")
                || l.texte.contains("{LINE_COLUMNS}");
            ecrit && nomme
        })
        .map(|l| l.texte.chars().take(120).collect())
        .collect()
}

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entree in std::fs::read_dir(dir).expect("lecture du répertoire") {
        let chemin = entree.expect("entrée").path();
        if chemin.is_dir() {
            sources(&chemin, out);
        } else if chemin.extension().is_some_and(|e| e == "rs") {
            out.push(chemin);
        }
    }
}

fn racine_crates() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .to_path_buf()
}

#[test]
fn no_production_code_writes_the_lettering_mark_outside_the_primitive() {
    let primitive = racine_crates().join("kesh-db/src/repositories/letterings.rs");
    let mut fichiers = Vec::new();
    for crate_dir in std::fs::read_dir(racine_crates()).expect("crates/") {
        let src = crate_dir.expect("crate").path().join("src");
        if src.is_dir() {
            sources(&src, &mut fichiers);
        }
    }
    assert!(
        fichiers.len() > 100,
        "le balayage doit couvrir les sources de production ({} fichiers vus)",
        fichiers.len()
    );
    let mut fautes = Vec::new();
    for f in &fichiers {
        if *f == primitive {
            continue;
        }
        let source = std::fs::read_to_string(f).expect("source");
        for extrait in ecritures_de_la_marque(&source) {
            fautes.push(format!("{} : {extrait}", f.display()));
        }
    }
    assert!(
        fautes.is_empty(),
        "⛔ la marque du lettrage s'écrit hors de `letterings::create_group_in_tx` / \
         `dissolve_group_in_tx` (R3) :\n{}",
        fautes.join("\n")
    );

    // La primitive elle-même est bien vue par le détecteur : sinon, un
    // détecteur muet passerait ce test pour de bon.
    let source = std::fs::read_to_string(&primitive).expect("primitive");
    assert_eq!(
        ecritures_de_la_marque(&source).len(),
        2,
        "la primitive porte exactement deux écritures de la marque (pose, retrait)"
    );
}

#[test]
fn each_primitive_checks_the_rows_its_update_found() {
    let source =
        std::fs::read_to_string(racine_crates().join("kesh-db/src/repositories/letterings.rs"))
            .expect("primitive");
    let (_, masque) = decouper(&source);
    for primitive in [
        "pub async fn create_group_in_tx",
        "pub async fn dissolve_group_in_tx",
    ] {
        let pos = masque
            .find(primitive)
            .unwrap_or_else(|| panic!("{primitive} introuvable"));
        // Le corps : le bloc qui suit la signature (la première `{` après le
        // type de retour).
        let signature_fin = pos + masque[pos..].find(") -> Result").expect("signature");
        let (de, a) = bloc_apres(&masque, signature_fin).expect("corps");
        let corps = &masque[de..=a];
        let update = corps.find("sqlx::query").is_some();
        assert!(update, "{primitive} : aucune requête");
        assert!(
            corps.contains("check_rows_affected("),
            "⛔ {primitive} n'appelle plus `check_rows_affected` sur le résultat de son \
             UPDATE (R7 point 4)"
        );
    }
}

/// Le détecteur, éprouvé sur un source synthétique.
#[test]
fn the_detector_sees_writes_and_only_writes() {
    let source = r##"
        const A: &str = "UPDATE journal_entry_lines SET lettering_key = ? WHERE id = ?";
        const B: &str = "SELECT lettering_key FROM journal_entry_lines";
        // "UPDATE journal_entry_lines SET lettering_key = 1" en commentaire
        const C: &str = r#"INSERT INTO journal_entry_lines (id, lettering_origin) VALUES (1, 'x')"#;
        fn f() { let x = format!("INSERT INTO journal_entry_lines ({LINE_COLUMNS}) SELECT 1"); let c = '"'; let d = '{'; }
        const D: &str = "UPDATE accounts SET name = ?";
        #[cfg(test)]
        mod tests {
            const E: &str = "UPDATE journal_entry_lines SET lettering_key = NULL";
            fn g() { let s = "{ids}"; }
        }
        const F: &str = "update journal_entry_lines set lettering_origin = 'manual'";
        const G: &str = "REPLACE INTO journal_entry_lines (id, lettering_key) VALUES (1, 1)";
        const H: &str = "UPDATE\njournal_entry_lines SET lettering_key = 1";
        const I: &str = "INSERT\tINTO journal_entry_lines (lettering_origin) VALUES ('x')";
        const J: &str = r#"UPDATE
            journal_entry_lines SET lettering_origin = NULL"#;
        const K: &str = "SELECT lettering_key FROM journal_entry_lines WHERE updated_at > ?";
        const L: &str = "SELECT id FROM journal_entry_lines WHERE lettering_key = ? FOR UPDATE";
        const M: &str = "INSERT INTO t (a) VALUES (1) ON DUPLICATE KEY UPDATE lettering_key = 1";
    "##;
    let vus = ecritures_de_la_marque(source);
    assert_eq!(vus.len(), 9, "{vus:#?}");
    assert!(vus[0].starts_with("UPDATE journal_entry_lines SET lettering_key"));
    assert!(vus[1].starts_with("INSERT INTO journal_entry_lines (id, lettering_origin)"));
    assert!(vus[2].contains("{LINE_COLUMNS}"));
    assert!(vus[3].starts_with("update journal_entry_lines set lettering_origin"));
    // Revue P1 (B6) : `REPLACE`, et un verbe suivi d'un autre blanc qu'une espace.
    assert!(vus[4].starts_with("REPLACE INTO"));
    assert!(vus[5].starts_with("UPDATE\\njournal_entry_lines"));
    assert!(vus[6].starts_with("INSERT\\tINTO"));
    assert!(vus[7].starts_with("UPDATE\n"));
    // `FOR UPDATE` n'est pas une écriture ; `ON DUPLICATE KEY UPDATE` en est une.
    assert!(vus[8].contains("ON DUPLICATE KEY UPDATE"));
}
