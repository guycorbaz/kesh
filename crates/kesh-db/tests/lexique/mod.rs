//! Analyse lexicale des sources Rust de production, partagée par les tests
//! structurels — extraite de `letterings_lexical.rs` (Story 15-1a-i) par la
//! Story 28-4a (#642), qui en a besoin pour son registre des écrivains
//! (`crates/kesh-api/tests/bank_account_closed_inventory.rs`, qui l'inclut par
//! `#[path]`). Le code est celui d'origine, déplacé sans changement de sens.
//!
//! Ce que l'analyse sait faire : séparer les littéraux de chaîne (ordinaires et
//! bruts) du code, masquer chaînes et commentaires pour compter les accolades,
//! repérer les blocs `#[cfg(test)]` et les fonctions (nom, corps), et lister
//! les sources `crates/*/src/**/*.rs`.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// Un littéral de chaîne et sa position (octet de début) dans la source.
pub struct Litteral {
    pub debut: usize,
    pub texte: String,
}

/// Découpe une source Rust : rend ses littéraux de chaîne (ordinaires et
/// bruts), et une copie « masquée » où le contenu des chaînes et des
/// commentaires est remplacé par des espaces — de quoi compter les accolades
/// sans être trompé par un `"{ids}"`.
pub fn decouper(source: &str) -> (Vec<Litteral>, String) {
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
pub fn bloc_apres(masque: &str, depuis: usize) -> Option<(usize, usize)> {
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
pub fn blocs_de_test(masque: &str) -> Vec<(usize, usize)> {
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

/// Le texte d'un littéral avec chaque séquence d'échappement — la barre
/// oblique inverse ET le caractère qui la suit — remplacée par deux espaces.
///
/// Le texte d'un littéral est la source **brute** (`decouper` ne décode
/// rien) : sans cette neutralisation, `"…;\nUPDATE …"` se découpe en
/// `nUPDATE`, qui n'est aucun verbe — idem `\t`, `\r`, `\0` (revue de code
/// P2, B2-1 / E2-1, régression née de B6). Couper sur la paire entière couvre
/// aussi `\\` (la paire est consommée d'un bloc, le mot suivant reste
/// entier) et la continuation de ligne (`\` suivi du saut de ligne).
///
/// ⚠️ Seule, elle **rétrécit** le détecteur : dans une chaîne brute, où `\`
/// n'échappe rien, `\UPDATE` devient `  PDATE` (revue P3 ciblée, F1). C'est
/// pourquoi un détecteur cherche le verbe dans le texte neutralisé **et** dans
/// le texte brut, la barre oblique y servant de séparateur : l'union des deux
/// lectures ne rate aucune des deux formes (P7 : chercher large).
pub fn neutraliser_echappements(texte: &str) -> String {
    let mut sortie = String::with_capacity(texte.len());
    let mut caracteres = texte.chars();
    while let Some(c) = caracteres.next() {
        if c == '\\' {
            sortie.push(' ');
            if caracteres.next().is_some() {
                sortie.push(' ');
            }
        } else {
            sortie.push(c);
        }
    }
    sortie
}

/// Les fonctions d'une source masquée : `(nom, début du corps, fin du corps)`.
/// Une déclaration sans corps (`fn x();`) est ignorée.
pub fn fonctions(masque: &str) -> Vec<(String, usize, usize)> {
    let b = masque.as_bytes();
    let mut out = Vec::new();
    let mut depuis = 0;
    while let Some(n) = masque[depuis..].find("fn ") {
        let pos = depuis + n;
        depuis = pos + 3;
        if pos > 0 && (b[pos - 1].is_ascii_alphanumeric() || b[pos - 1] == b'_') {
            continue;
        }
        let nom: String = masque[pos + 3..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if nom.is_empty() {
            continue;
        }
        let Some((de, a)) = bloc_apres(masque, pos) else {
            continue;
        };
        if masque[pos..de].contains(';') {
            continue;
        }
        out.push((nom, de, a));
    }
    out
}

/// La fonction la plus intérieure qui contient la position `p`.
pub fn fonction_englobante(
    fns: &[(String, usize, usize)],
    p: usize,
) -> Option<&(String, usize, usize)> {
    fns.iter()
        .filter(|(_, de, a)| *de <= p && p <= *a)
        .min_by_key(|(_, de, a)| a - de)
}

/// Les fichiers `.rs` sous `dir`, récursivement.
pub fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entree in std::fs::read_dir(dir).expect("lecture du répertoire") {
        let chemin = entree.expect("entrée").path();
        if chemin.is_dir() {
            sources(&chemin, out);
        } else if chemin.extension().is_some_and(|e| e == "rs") {
            out.push(chemin);
        }
    }
}

/// Le répertoire `crates/` du dépôt (parent de la crate du test).
pub fn racine_crates() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .to_path_buf()
}
