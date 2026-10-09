//! Lecture des sources LaTeX des manuels — Story 15-14b (factorisé pour les
//! gardes de `configuration_transmise.rs` et de `textes_coherents.rs`).
//!
//! Inclus par `#[path = "common/manuel.rs"] mod manuel;` — **pas** par
//! `mod common;`, qui tirerait l'amorce de base de `common/mod.rs` dans des
//! binaires de test qui ne l'emploient pas.
//!
//! Deux outils, sans aucune assertion de garde (chaque appelant pose la
//! sienne) :
//!
//! - [`normaliser`] — la normalisation `N` de la fiche 15-14b (AC 3), celle
//!   de la commande d'inventaire `perl` : une phrase coupée sur deux lignes,
//!   mise en gras en son milieu ou accentuée par une macro se lit d'un tenant ;
//! - [`sections_synology`] — les bornes, sur le source **brut**, des deux
//!   sections du manuel d'administration qui décrivent Synology.

#![allow(dead_code)] // chaque binaire de test n'emploie qu'une partie du module

use regex::Regex;

/// Normalisation `N` (fiche 15-14b, AC 3 — même ordre que la commande `perl`
/// d'inventaire, pour que le test et l'inventaire comptent la même chose) :
///
/// 1. `\textbf`, `\emph`, `\texttt`, `\textit`, `\keshcommand`, `\keshpath`
///    dépliés **jusqu'à stabilité** (imbrications comprises) ;
/// 2. accents écrits en macros : `\'e` → `é`, `` \`e `` → `è`, `\^e` → `ê`,
///    `\^a` → `â`, `` \`a `` → `à` (accolades admises : `\'{e}`) ;
/// 3. `\_` → `_` (les noms du compose s'écrivent échappés en LaTeX) ;
/// 4. `~` et tout blanc (sauts de ligne compris) → une espace.
///
/// ⚠️ Les sauts de ligne disparaissent : les **bornes** d'une section se
/// prennent sur le source brut ([`sections_synology`]), avant normalisation.
pub fn normaliser(texte: &str) -> String {
    let commande =
        Regex::new(r"\\(?:textbf|emph|texttt|textit|keshcommand|keshpath)\{([^{}]*)\}").unwrap();
    let mut courant = texte.to_string();
    loop {
        let suivant = commande.replace_all(&courant, "$1").into_owned();
        if suivant == courant {
            break;
        }
        courant = suivant;
    }
    for (motif, accent) in [
        (r"\\'\{?e\}?", "é"),
        (r"\\`\{?e\}?", "è"),
        (r"\\\^\{?e\}?", "ê"),
        (r"\\\^\{?a\}?", "â"),
        (r"\\`\{?a\}?", "à"),
    ] {
        courant = Regex::new(motif)
            .unwrap()
            .replace_all(&courant, accent)
            .into_owned();
    }
    let courant = desechapper(&courant).replace('~', " ");
    Regex::new(r"\s+")
        .unwrap()
        .replace_all(&courant, " ")
        .into_owned()
}

/// `\_` → `_` seul : pour comparer un nom cité par le manuel à un nom du
/// compose sans perdre les sauts de ligne du source.
pub fn desechapper(texte: &str) -> String {
    texte.replace("\\_", "_")
}

/// Labels des deux sections Synology du manuel d'administration.
pub const LABELS_SYNOLOGY: [&str; 2] = ["sec:synology", "sec:backup-dsm"];

/// Bornes `[début, fin)` (octets du source **brut**) de la section qui porte
/// `\label{<label>}` : du `\subsection{` qui porte le label — **titre compris**,
/// le titre de `sec:backup-dsm` nommant Hyper Backup avant son `\label` — au
/// premier `\subsection` ou `\section` qui suit (après `sec:synology` vient une
/// `\section`, d'un niveau supérieur : « même niveau » seul ne bornerait pas).
/// `\subsubsection` ne borne pas. Panique si le label manque, s'il n'est pas
/// sur la ligne d'un `\subsection{`, ou si rien ne suit : une borne non
/// trouvée ferait passer une garde à vide.
pub fn section(source: &str, label: &str) -> (usize, usize) {
    let marque = format!("\\label{{{label}}}");
    let pos = source
        .find(&marque)
        .unwrap_or_else(|| panic!("{marque} absent du manuel"));
    let debut_ligne = source[..pos].rfind('\n').map_or(0, |i| i + 1);
    let debut = source[debut_ligne..pos]
        .find("\\subsection{")
        .map(|i| debut_ligne + i)
        .unwrap_or_else(|| panic!("{marque} n'est pas sur la ligne d'un \\subsection{{"));
    let suite = Regex::new(r"\\(?:sub)?section\*?\{").unwrap();
    let apres = pos + marque.len();
    let fin = suite
        .find(&source[apres..])
        .map(|m| apres + m.start())
        .unwrap_or_else(|| panic!("aucune \\subsection ni \\section après {marque}"));
    (debut, fin)
}

/// Les deux sections Synology (`sec:synology`, `sec:backup-dsm`), dans l'ordre.
pub fn sections_synology(source: &str) -> Vec<(usize, usize)> {
    LABELS_SYNOLOGY.iter().map(|l| section(source, l)).collect()
}

/// Le source privé des sections données (remplacées par un saut de ligne, pour
/// ne pas souder deux phrases).
pub fn hors_sections(source: &str, sections: &[(usize, usize)]) -> String {
    let mut out = String::with_capacity(source.len());
    let mut curseur = 0;
    let mut triees = sections.to_vec();
    triees.sort();
    for (d, f) in triees {
        out.push_str(&source[curseur..d]);
        out.push('\n');
        curseur = f;
    }
    out.push_str(&source[curseur..]);
    out
}

/// Contenus des `lstlisting` d'un extrait de source (options `[…]` exclues).
pub fn listings(extrait: &str) -> Vec<String> {
    Regex::new(r"(?s)\\begin\{lstlisting\}(?:\[[^\]]*\])?(.*?)\\end\{lstlisting\}")
        .unwrap()
        .captures_iter(extrait)
        .map(|c| c[1].to_string())
        .collect()
}

/// Le source privé de ses **commentaires LaTeX** : de tout `%` non échappé
/// (`\%` reste) jusqu'à la fin de la ligne, **hors** des `lstlisting`, où `%`
/// est littéral (`date +%Y%m%d`). Sans cela, une phrase exigée par une garde
/// pourrait n'exister qu'en commentaire, absente du PDF, et la garde passerait
/// (revue de code P1 de la 15-14b, B10). La commande `perl` d'inventaire de
/// l'AC 3 ne retire pas les commentaires : les deux ne diffèrent que sur une
/// ligne commentée qui porterait le motif (aucune au 2026-10-09).
///
/// **Angles morts écrits** (revue de code P2, E2-10) : `\\%` (saut de ligne LaTeX
/// suivi d'un commentaire) est pris pour un `\%` échappé, et le commentaire reste
/// lu ; un `% \begin{lstlisting}` commenté fait passer en mode listing jusqu'au
/// `\end{lstlisting}` suivant. Aucun des deux dans les manuels au 2026-10-09.
pub fn sans_commentaires(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut dans_listing = false;
    for ligne in source.split_inclusive('\n') {
        if dans_listing {
            out.push_str(ligne);
            if ligne.contains("\\end{lstlisting}") {
                dans_listing = false;
            }
            continue;
        }
        let mut coupe = None;
        let octets = ligne.as_bytes();
        for (i, b) in octets.iter().enumerate() {
            if *b == b'%' && (i == 0 || octets[i - 1] != b'\\') {
                coupe = Some(i);
                break;
            }
        }
        match coupe {
            Some(i) => {
                out.push_str(&ligne[..i]);
                if ligne.ends_with('\n') {
                    out.push('\n');
                }
            }
            None => out.push_str(ligne),
        }
        if ligne.contains("\\begin{lstlisting}") && !ligne.contains("\\end{lstlisting}") {
            dans_listing = true;
        }
    }
    out
}
