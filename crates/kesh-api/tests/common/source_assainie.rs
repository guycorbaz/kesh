//! La source de production, **prête à être lue** par une garde lexicale :
//! fichiers de `crates/*/src`, commentaires de ligne retirés, items
//! `#[cfg(test)]` masqués par appariement d'accolades, arguments d'un appel
//! découpés.
//!
//! Écrit à la Story 25-1c-a pour la garde des libellés du journal d'audit
//! (`audit_label_registry.rs`), mis en commun par la Story 35-8a1 (#431, D2 :
//! « pas de parseur neuf ») pour servir aussi la garde des acteurs
//! (`audit_acteur_garde.rs`). Les fonctions passent **sans changement** ; leurs
//! limites, écrites à leurs doc-comments, deviennent celles des deux gardes.
//!
//! Inclus par `#[path = "common/source_assainie.rs"] mod assainissement;` —
//! **pas** par `mod common;`, qui tirerait l'amorce de `common/mod.rs` dans des
//! binaires qui ne l'emploient pas (patron de `common/autour_appel.rs`).

#![allow(dead_code)] // chaque binaire de test n'emploie qu'une partie du module

use std::path::{Path, PathBuf};

/// La racine `crates/` du dépôt.
pub fn racine_crates() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/kesh-api a un parent")
        .to_path_buf()
}

/// Tous les `.rs` de production — `crates/*/src/**`, les répertoires `tests/`
/// exclus par construction.
pub fn fichiers_de_production() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for crate_dir in std::fs::read_dir(racine_crates()).expect("lire crates/") {
        let src = crate_dir.expect("entrée de crates/").path().join("src");
        if src.is_dir() {
            collecter(&src, &mut out);
        }
    }
    out.sort();
    out
}

/// Ajoute à `out` tous les `.rs` sous `dir`, récursivement.
pub fn collecter(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).expect("lire un répertoire source") {
        let p = e.expect("entrée de répertoire").path();
        if p.is_dir() {
            collecter(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// Reprise du décommentage de `admin_pat_denied_e2e.rs` — même limite connue :
/// un `//` **dans une chaîne** tronquerait la ligne. Aucun argument d'audit n'en
/// porte, et le diff bilatéral le signalerait s'il en apparaissait un.
pub fn strip_line_comments(src: &str) -> String {
    src.lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

/// La source d'un fichier, prête à être lue : commentaires masqués et blocs
/// `#[cfg(test)]` **retirés par appariement d'accolades**. **Tous les lecteurs
/// de ce fichier passent par ici** — leur nombre n'est pas écrit : il a changé
/// trois fois, et la phrase était fausse deux fois sur trois.
///
/// ⛔ **On MASQUE le bloc, on ne TRONQUE pas le fichier — et il a fallu deux
/// passes pour y venir.**
///
/// - Première rédaction : `brut.split("#[cfg(test)]")` sur la source brute.
///   `config.rs:406` porte cette chaîne dans un **doc-comment**, bien avant son
///   vrai attribut : **83 %** du fichier sortait du balayage.
/// - Deuxième : coupe sur une **ligne entière**, ce qui fermait la variante
///   « sous-chaîne dans la prose » — et laissait intacte la plus coûteuse. Un
///   `#[cfg(test)]` posé sur une **méthode** (`invoice_email.rs:901`) ou un `mod
///   tests` placé **au milieu** d'un fichier (`version.rs:97`,
///   `entities/user.rs:178`) sont deux arrangements Rust ordinaires : tronquer
///   à la première occurrence perdait **686, 256 et 75 lignes de PRODUCTION**,
///   dont `send_reminder_batch`, `check_downgrade_protection` et `UserUpdate`.
///   ⚠️ Or `invoice_email.rs` **écrit déjà de l'audit**, et la zone perdue
///   appelait cette écriture : le trou était sous le pied du prochain code
///   d'audit.
///
/// *C'est l'appariement d'accolades que l'AC 18 demandait dès l'origine ; je
/// l'avais simplifié en `split`, en comptant sur le diff bilatéral pour
/// rattraper. Il ne rattrape rien pour une action NEUVE, qui n'entre dans aucun
/// ensemble.*
///
/// ⚠️ Les accolades sont comptées **hors chaînes** — sans quoi une accolade
/// isolée dans un littéral (`"Texte { non fermé"`, réel dans
/// `kesh-core/src/email_template_engine.rs`) déséquilibrerait le compte.
///
/// ⚠️ **Limites assumées, toutes écrites** — un doc-comment qui prétend les
/// écrire et en omet est pire qu'un doc-comment muet :
///
/// - un `//` **dans une chaîne** tronque la ligne (hérité de
///   `strip_line_comments`) ;
/// - les commentaires de **bloc** `/* … */` ne sont pas retirés ;
/// - `accolades_hors_chaines` ne comprend pas la sémantique des **chaînes
///   brutes** (`r#"…"#`), où un guillemet interne n'est pas un délimiteur : 27
///   lignes du dépôt en portent dans des blocs masqués, toutes de bilan nul.
///
/// ⚠️ **Les littéraux de caractère, eux, SONT traités** — une rédaction
/// antérieure affirmait ici qu'aucun cas n'existait, « vérifié », et c'était
/// **faux** : `loader.rs:386` en porte un. *Une limite déclarée sans être
/// mesurée vaut moins qu'une limite tue : elle dissuade d'aller voir.*
pub fn source_assainie(brut: &str) -> String {
    let sans_commentaires = strip_line_comments(brut);
    let lignes: Vec<&str> = sans_commentaires.lines().collect();
    let mut gardees: Vec<&str> = Vec::new();
    let mut i = 0;

    while i < lignes.len() {
        if lignes[i].trim() != "#[cfg(test)]" {
            gardees.push(lignes[i]);
            i += 1;
            continue;
        }
        // Bloc de test : sauter l'attribut, puis l'item qu'il garde, jusqu'à ce
        // que ses accolades se referment.
        //
        // ⛔ **Deux sorties, et l'absence de la seconde a coûté une passe.** Un
        // item gardé peut n'ouvrir AUCUNE accolade — `use …;`, `mod fixtures;`,
        // `const … = …;`. Sans sortie propre, la boucle continuait jusqu'à la
        // prochaine accolade du fichier, **quelle qu'elle soit**, avalant tout ce
        // qui se trouvait entre les deux ; et si le fichier n'en portait plus,
        // jusqu'à la fin. `#[cfg(test)] mod fixtures;` est un idiome courant, pas
        // une forme exotique.
        i += 1;
        let mut profondeur = 0i32;
        let mut ouvert = false;
        while i < lignes.len() {
            let ligne = lignes[i];
            let (o, f) = accolades_hors_chaines(ligne);
            profondeur += o - f;
            if o > 0 {
                ouvert = true;
            }
            i += 1;
            // Sortie 1 — le bloc s'est ouvert puis refermé.
            if ouvert && profondeur <= 0 {
                break;
            }
            // Sortie 2 — l'item s'est terminé SANS jamais ouvrir de bloc.
            if !ouvert && profondeur == 0 && ligne.trim_end().ends_with(';') {
                break;
            }
        }
    }
    gardees.join("\n")
}

/// Compte les accolades d'une ligne **en ignorant celles des chaînes et des
/// littéraux de caractère**.
///
/// ⛔ **`'{'` compte pour une accolade si on ne le traite pas — et ce n'était pas
/// théorique.** `kesh-i18n/src/loader.rs:386` porte `ligne[..pos].contains('{')`
/// **à l'intérieur** de son bloc de test : la profondeur n'y retombait jamais à
/// zéro, et ce bloc n'était masqué correctement que **par accident**, parce
/// qu'il court jusqu'à la dernière ligne du fichier. Toute production ajoutée
/// après lui aurait été avalée en silence.
///
/// ⚠️ Une apostrophe n'est pas toujours un littéral : `&'static str` en porte
/// une qui ne ferme jamais. On ne saute donc que la forme complète — `'x'` ou
/// `'\n'` —, et une **lifetime** est laissée telle quelle, sans effet sur le
/// compte.
pub fn accolades_hors_chaines(ligne: &str) -> (i32, i32) {
    let (mut ouvrantes, mut fermantes) = (0, 0);
    let (mut dans_chaine, mut echappe) = (false, false);
    let chars: Vec<char> = ligne.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        if dans_chaine {
            if echappe {
                echappe = false;
            } else if ch == '\\' {
                echappe = true;
            } else if ch == '"' {
                dans_chaine = false;
            }
            i += 1;
            continue;
        }
        match ch {
            '"' => dans_chaine = true,
            '\'' => {
                // `'x'` (3 caractères) ou `'\n'` (4) : un littéral, qu'on saute
                // en entier. Tout le reste est une lifetime.
                if i + 2 < chars.len() && chars[i + 1] != '\\' && chars[i + 2] == '\'' {
                    i += 3;
                    continue;
                }
                if i + 3 < chars.len() && chars[i + 1] == '\\' && chars[i + 3] == '\'' {
                    i += 4;
                    continue;
                }
            }
            '{' => ouvrantes += 1,
            '}' => fermantes += 1,
            _ => {}
        }
        i += 1;
    }
    (ouvrantes, fermantes)
}

/// Découpe les arguments d'un appel — `src` commence **après** la parenthèse
/// ouvrante. Rend `None` si l'appel n'est pas refermé.
pub fn args_de(src: &str) -> Option<Vec<String>> {
    let (mut depth, mut cur, mut out) = (0usize, String::new(), Vec::new());
    let (mut in_str, mut echappe) = (false, false);
    for ch in src.chars() {
        if in_str {
            cur.push(ch);
            if echappe {
                echappe = false;
            } else if ch == '\\' {
                echappe = true;
            } else if ch == '"' {
                in_str = false;
            }
            continue;
        }
        match ch {
            '"' => {
                in_str = true;
                cur.push(ch);
            }
            '(' | '[' | '{' => {
                depth += 1;
                cur.push(ch);
            }
            ')' | ']' | '}' => {
                if depth == 0 {
                    out.push(cur);
                    return Some(out);
                }
                depth -= 1;
                cur.push(ch);
            }
            ',' if depth == 0 => {
                out.push(std::mem::take(&mut cur));
            }
            _ => cur.push(ch),
        }
    }
    None
}
