//! Test structurel — Story 15-1a-i (#518), R3 et R7 point 4 (T9).
//!
//! ⛔ **UNE seule fonction écrit la marque du lettrage**
//! (`letterings::create_group_inner`) **et une seule la retire**
//! (`letterings::dissolve_group_inner`) — atteintes par les deux primitives et
//! par la synchronisation des pièces seules (Story 15-1a2-i). Ce test lit les sources de production
//! — `crates/*/src/**/*.rs`, **hors** blocs `#[cfg(test)]` (les tests de dépôt
//! vivent aussi dans des `mod tests` de `src/`) et **hors**
//! `kesh-db/src/repositories/letterings.rs` — et refuse tout littéral de chaîne
//! qui porte le mot `UPDATE`, `INSERT` ou `REPLACE` (toutes formes : `VALUES`,
//! `SELECT`, quel que soit le blanc qui suit le verbe) et
//! qui nomme `lettering_key` ou `lettering_origin`, ou qui interpole
//! `LINE_COLUMNS` (la liste de colonnes des lignes, qui les porte).
//!
//! Les **exceptions** de R3 n'apparaissent pas ici, et c'est voulu : les deux
//! migrations de rattrapage de la 15-1a2-ii
//! (`20261010000001_lettering_documents_backfill.sql`,
//! `20261010000002_lettering_reversal_pairs_backfill.sql`) sont des fichiers
//! `.sql` — et la première, rejouée à l'import (`post_restore.rs`), y est
//! embarquée par `include_str!`, sans littéral Rust ; la
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
//! Second volet (R7 point 4, F4-4) : chacun des deux corps qui écrivent la
//! marque (`create_group_inner`, `dissolve_group_inner`) appelle
//! `check_rows_affected` sur le résultat de son `UPDATE` — c'est la seule garde
//! contre la mutation « retirer la vérification », qu'aucun montage de test ne
//! peut exercer sans crochet de production.
//!
//! Troisième volet (Story 15-1a2-i, AC8 part i) : l'**inventaire fermé** des
//! écrivains des pièces clientes — tout `INSERT`/`DELETE` de
//! `invoice_settlements` vit dans son module, toute fonction qui crée un
//! règlement synchronise le lettrage **après**, l'unique `INSERT INTO
//! credit_notes` aussi, et l'annulation dissout **avant** de contre-passer.
//! ⚠️ Ce que ce volet ne voit pas : un appel par un chemin importé
//! (`use invoice_settlements::create_in_tx;` puis `create_in_tx(`) — le volet
//! (a), qui borne l'`INSERT` à son module, en limite la portée.
//!
//! Quatrième volet (Story 15-1a2-ii, AC8 part ii) : les trois `UPDATE
//! supplier_invoices … settlement_journal_entry_id` de production — le paiement
//! synchronise **après** son `UPDATE`, les deux annulations dissolvent **avant**
//! de contre-passer ; un site neuf rougit en se nommant.

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
/// pourquoi `ecritures_de_la_marque` cherche le verbe dans le texte neutralisé
/// **et** dans le texte brut, la barre oblique y servant de séparateur : l'union
/// des deux lectures ne rate aucune des deux formes (P7 : chercher large).
fn neutraliser_echappements(texte: &str) -> String {
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

/// Les écritures de la marque hors de la primitive : `(rang du littéral, extrait)`.
fn ecritures_de_la_marque(source: &str) -> Vec<String> {
    let (litteraux, masque) = decouper(source);
    let tests = blocs_de_test(&masque);
    litteraux
        .into_iter()
        .filter(|l| !tests.iter().any(|(de, a)| l.debut >= *de && l.debut <= *a))
        .filter(|l| {
            // Un verbe d'écriture est un MOT du littéral, quel que soit le
            // blanc qui le précède ou le suit (espace, tabulation, saut de
            // ligne, `\n` / `\t` / `\r` / `\0` échappés, continuation de
            // ligne) : chercher large (P7). `REPLACE` en est un (revue P1,
            // B6). Seuls `FOR UPDATE` (lecture verrouillante) et `ON UPDATE`
            // (DDL) ne sont pas des écritures ; `ON DUPLICATE KEY UPDATE` en
            // reste une.
            // Deux lectures, et leur union : le texte neutralisé (`\nUPDATE`
            // → `UPDATE`) et le texte brut, `\` y faisant séparateur
            // (`\UPDATE` d'une chaîne brute → `UPDATE`). Revue P3 ciblée, F1.
            let ecrit_dans = |texte: &str| {
                let mots: Vec<&str> = texte
                    .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                    .filter(|m| !m.is_empty())
                    .collect();
                mots.iter().enumerate().any(|(n, mot)| {
                    let verbe = ["UPDATE", "INSERT", "REPLACE"]
                        .iter()
                        .any(|v| mot.eq_ignore_ascii_case(v));
                    let lecture_ou_ddl = mot.eq_ignore_ascii_case("UPDATE")
                        && n > 0
                        && (mots[n - 1].eq_ignore_ascii_case("FOR")
                            || mots[n - 1].eq_ignore_ascii_case("ON"));
                    verbe && !lecture_ou_ddl
                })
            };
            let ecrit = ecrit_dans(&neutraliser_echappements(&l.texte)) || ecrit_dans(&l.texte);
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
        "⛔ la marque du lettrage s'écrit hors de `letterings::create_group_inner` / \
         `dissolve_group_inner` (R3) :\n{}",
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
    // Story 15-1a2-i : les deux primitives publiques délèguent à ces corps
    // privés, qui portent l'`UPDATE` — la vérification y est exigée.
    for primitive in [
        "async fn create_group_inner",
        "async fn dissolve_group_inner",
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
        const N: &str = "SELECT 1;\nUPDATE journal_entry_lines SET lettering_key = 1";
        const O: &str = "x\tINSERT INTO journal_entry_lines (lettering_origin) VALUES ('x')";
        const P: &str = "x\rREPLACE INTO journal_entry_lines (id, lettering_key) VALUES (1, 1)";
        const Q: &str = "x\0UPDATE journal_entry_lines SET lettering_origin = NULL";
        const R: &str = "SELECT 1; \
            UPDATE journal_entry_lines SET lettering_key = NULL";
        const S: &str = r#"SELECT 1;\UPDATE journal_entry_lines SET lettering_key = NULL"#;
        const T: &str = r#"x\\\INSERT INTO journal_entry_lines (lettering_origin) VALUES ('x')"#;
    "##;
    let vus = ecritures_de_la_marque(source);
    assert_eq!(vus.len(), 16, "{vus:#?}");
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
    // Revue de code P2 (B2-1 / E2-1) : un verbe PRÉCÉDÉ d'un échappement
    // (`\nUPDATE` se découpait en `nUPDATE`), et la continuation de ligne.
    assert!(vus[9].starts_with("SELECT 1;\\nUPDATE"));
    assert!(vus[10].starts_with("x\\tINSERT"));
    assert!(vus[11].starts_with("x\\rREPLACE"));
    assert!(vus[12].starts_with("x\\0UPDATE"));
    assert!(vus[13].starts_with("SELECT 1; \\\n"));
    // Revue P3 ciblée, F1 : une barre oblique d'une chaîne brute n'échappe rien,
    // le verbe qui la suit reste un verbe.
    assert!(vus[14].starts_with("SELECT 1;\\UPDATE"));
    assert!(vus[15].starts_with("x\\\\\\INSERT"));
}

// ===========================================================================
// Story 15-1a2-i (#518) — AC8 part i : l'inventaire fermé des écrivains des
// pièces clientes.
// ===========================================================================

/// Les fonctions d'une source masquée : `(nom, début du corps, fin du corps)`.
/// Une déclaration sans corps (`fn x();`) est ignorée.
fn fonctions(masque: &str) -> Vec<(String, usize, usize)> {
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
fn fonction_englobante(
    fns: &[(String, usize, usize)],
    p: usize,
) -> Option<&(String, usize, usize)> {
    fns.iter()
        .filter(|(_, de, a)| *de <= p && p <= *a)
        .min_by_key(|(_, de, a)| a - de)
}

/// Pour chaque appel `appel` du code (hors chaînes, commentaires et blocs
/// `#[cfg(test)]`), la fonction qui le porte et si elle appelle `suite`
/// **après lui, dans son propre corps** : `(nom, suivi)`.
fn appels_et_suite(source: &str, appel: &str, suite: &str) -> Vec<(String, bool)> {
    let (_, masque) = decouper(source);
    let tests = blocs_de_test(&masque);
    let fns = fonctions(&masque);
    let mut out = Vec::new();
    let mut depuis = 0;
    while let Some(n) = masque[depuis..].find(appel) {
        let p = depuis + n;
        depuis = p + appel.len();
        if tests.iter().any(|(de, a)| p >= *de && p <= *a) {
            continue;
        }
        let Some((nom, _, fin)) = fonction_englobante(&fns, p) else {
            out.push(("<hors fonction>".into(), false));
            continue;
        };
        let suivi = masque[p + appel.len()..*fin].contains(suite);
        out.push((nom.clone(), suivi));
    }
    out
}

/// Les littéraux de production (hors blocs `#[cfg(test)]`) qui écrivent la
/// table `table` par l'un des `verbes` (`"INSERT INTO"`, `"DELETE FROM"`) —
/// blancs normalisés, insensible à la casse, nom de table entier :
/// `(position, extrait)`.
fn ecritures_de_table(source: &str, verbes: &[&str], table: &str) -> Vec<(usize, String)> {
    let (litteraux, masque) = decouper(source);
    let tests = blocs_de_test(&masque);
    litteraux
        .into_iter()
        .filter(|l| !tests.iter().any(|(de, a)| l.debut >= *de && l.debut <= *a))
        .filter(|l| {
            let mots: Vec<String> = neutraliser_echappements(&l.texte)
                .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .filter(|m| !m.is_empty())
                .map(|m| m.to_ascii_uppercase())
                .collect();
            verbes.iter().any(|v| {
                let v: Vec<String> = v.split_whitespace().map(str::to_string).collect();
                mots.windows(v.len() + 1)
                    .any(|w| w[..v.len()] == v[..] && w[v.len()].eq_ignore_ascii_case(table))
            })
        })
        .map(|l| (l.debut, l.texte.chars().take(80).collect()))
        .collect()
}

/// AC8 (c) — pour chaque `INSERT INTO <table>` de production, la fonction qui le
/// porte et si elle appelle `suite` **après lui**, dans son corps : `(nom, suivi)`.
/// Un `INSERT` hors de toute fonction est rendu sous le nom `<hors fonction>`,
/// non suivi.
fn ecritures_et_suite(source: &str, table: &str, suite: &str) -> Vec<(String, bool)> {
    let (_, masque) = decouper(source);
    let fns = fonctions(&masque);
    ecritures_de_table(source, &["INSERT INTO"], table)
        .into_iter()
        .map(|(p, _)| match fonction_englobante(&fns, p) {
            Some((nom, _, fin)) => (nom.clone(), masque[p..*fin].contains(suite)),
            None => ("<hors fonction>".to_string(), false),
        })
        .collect()
}

/// AC8 (d) — le corps de la fonction `fonction` appelle `premier` **avant**
/// `second` (code seul : chaînes et commentaires masqués).
fn appelle_avant(source: &str, fonction: &str, premier: &str, second: &str) -> Result<(), String> {
    let (_, masque) = decouper(source);
    let (_, de, a) = fonctions(&masque)
        .into_iter()
        .find(|(nom, _, _)| nom == fonction)
        .ok_or_else(|| format!("fonction `{fonction}` introuvable"))?;
    let corps = &masque[de..=a];
    match (corps.find(premier), corps.find(second)) {
        (Some(p), Some(s)) if p < s => Ok(()),
        autre => Err(format!(
            "fonction `{fonction}` : `{premier}` doit précéder `{second}` (positions {autre:?})"
        )),
    }
}

/// Les sources de production : `(chemin relatif à crates/, contenu)`.
fn sources_de_production() -> Vec<(String, String)> {
    let mut fichiers = Vec::new();
    for crate_dir in std::fs::read_dir(racine_crates()).expect("crates/") {
        let src = crate_dir.expect("crate").path().join("src");
        if src.is_dir() {
            sources(&src, &mut fichiers);
        }
    }
    assert!(
        fichiers.len() > 100,
        "balayage trop court ({})",
        fichiers.len()
    );
    let racine = racine_crates();
    fichiers.sort();
    fichiers
        .into_iter()
        .map(|f| {
            let rel = f
                .strip_prefix(&racine)
                .expect("sous crates/")
                .display()
                .to_string();
            (rel, std::fs::read_to_string(&f).expect("source"))
        })
        .collect()
}

/// AC8 (a), (b) — tout `INSERT INTO` / `DELETE FROM invoice_settlements` de
/// production vit dans son module ; toute fonction de production qui crée un
/// règlement (`invoice_settlements::create_in_tx(`) appelle
/// `sync_invoice_in_tx(` **après lui**, dans son propre corps.
#[test]
fn invoice_settlement_writers_stay_in_their_module_and_sync() {
    let modules = [
        "kesh-db/src/repositories/invoice_settlements.rs",
        "kesh-db/src/repositories/invoice_settlements_write.rs",
    ];
    let mut fautes = Vec::new();
    let mut ecrivains = 0;
    let mut createurs = Vec::new();
    for (fichier, source) in sources_de_production() {
        for (_, extrait) in ecritures_de_table(
            &source,
            &["INSERT INTO", "DELETE FROM"],
            "invoice_settlements",
        ) {
            ecrivains += 1;
            if !modules.contains(&fichier.as_str()) {
                fautes.push(format!("(a) {fichier} : {extrait}"));
            }
        }
        for (fonction, suivi) in appels_et_suite(
            &source,
            "invoice_settlements::create_in_tx(",
            "sync_invoice_in_tx(",
        ) {
            createurs.push(format!("{fichier}::{fonction}"));
            if !suivi {
                fautes.push(format!(
                    "(b) {fichier}, fonction `{fonction}` : crée un règlement sans appeler \
                     `sync_invoice_in_tx(` après lui"
                ));
            }
        }
    }
    assert!(
        fautes.is_empty(),
        "⛔ inventaire des écrivains de `invoice_settlements` (Story 15-1a2-i, AC8) :\n{}",
        fautes.join("\n")
    );
    // Anti-muet : l'`INSERT` et le `DELETE` connus, et les trois créateurs.
    assert_eq!(ecrivains, 2, "un INSERT et un DELETE de production");
    createurs.sort();
    assert_eq!(
        createurs,
        [
            "kesh-api/src/routes/reconciliation.rs::accept_one_invoice",
            "kesh-db/src/repositories/invoice_settlements_write.rs::settle_invoice",
            "kesh-db/src/repositories/invoice_settlements_write.rs::write_off_invoice",
        ],
        "les créateurs de règlement connus"
    );
}

/// AC8 (c), (d) — l'unique `INSERT INTO credit_notes` de production est dans
/// `create_credit_note`, qui synchronise **après** lui ; l'annulation d'un
/// règlement client dissout le groupe **avant** de contre-passer.
#[test]
fn credit_note_insert_is_followed_by_sync_and_cancel_dissolves_first() {
    let mut fautes = Vec::new();
    let mut inserts = Vec::new();
    for (fichier, source) in sources_de_production() {
        for (nom, suivi) in ecritures_et_suite(&source, "credit_notes", "sync_invoice_in_tx(") {
            if !suivi {
                fautes.push(format!(
                    "(c) {fichier}, fonction `{nom}` : INSERT INTO credit_notes sans \
                     `sync_invoice_in_tx(` après lui"
                ));
            }
            inserts.push(format!("{fichier}::{nom}"));
        }
    }
    assert_eq!(
        inserts,
        ["kesh-db/src/repositories/credit_notes.rs::create_credit_note"],
        "(c) l'unique INSERT INTO credit_notes de production"
    );

    let source = std::fs::read_to_string(
        racine_crates().join("kesh-db/src/repositories/invoice_settlements_write.rs"),
    )
    .expect("source");
    if let Err(e) = appelle_avant(
        &source,
        "cancel_settlement_in_tx",
        "dissolve_invoice_document_group_in_tx(",
        "reverse_owned_in_tx(",
    ) {
        fautes.push(format!("(d) invoice_settlements_write.rs : {e}"));
    }
    assert!(
        fautes.is_empty(),
        "⛔ inventaire des écrivains des pièces clientes (Story 15-1a2-i, AC8) :\n{}",
        fautes.join("\n")
    );
}

/// Les détecteurs d'AC8, éprouvés sur un source synthétique.
#[test]
fn the_function_body_detector_sees_calls_and_order() {
    let source = r##"
        // invoice_settlements::create_in_tx( en commentaire : ignoré
        async fn bonne() {
            invoice_settlements::create_in_tx(&mut tx, x).await?;
            let f = |y| { y + 1 };
            letterings::sync_invoice_in_tx(&mut tx, 1).await?;
        }
        async fn sans_sync() -> Result<(), E> {
            super::invoice_settlements::create_in_tx(&mut tx, x).await?;
        }
        async fn sync_avant() {
            sync_invoice_in_tx(&mut tx).await?;
            invoice_settlements::create_in_tx(&mut tx, x).await?;
        }
        fn imbriquee() {
            fn interne() { invoice_settlements::create_in_tx(a); }
            sync_invoice_in_tx(b);
        }
        fn chaine() { let s = "invoice_settlements::create_in_tx("; }
        const A: &str = "INSERT INTO invoice_settlements (a) VALUES (1)";
        const B: &str = "delete\n from invoice_settlements where id = ?";
        const C: &str = "INSERT INTO invoice_settlements_archive (a) VALUES (1)";
        const D: &str = "SELECT * FROM invoice_settlements";
        const E: &str = "INSERT INTO credit_note_lines (a) VALUES (1)";
        #[cfg(test)]
        mod tests {
            const F: &str = "INSERT INTO invoice_settlements (a) VALUES (1)";
            fn t() { invoice_settlements::create_in_tx(a); }
        }
    "##;
    let vus = appels_et_suite(
        source,
        "invoice_settlements::create_in_tx(",
        "sync_invoice_in_tx(",
    );
    assert_eq!(
        vus,
        vec![
            ("bonne".to_string(), true),
            ("sans_sync".to_string(), false),
            ("sync_avant".to_string(), false),
            ("interne".to_string(), false),
        ],
        "appels vus, fonction la plus intérieure, ordre exigé ; chaîne, commentaire et \
         bloc de test ignorés"
    );
    let ecritures: Vec<String> = ecritures_de_table(
        source,
        &["INSERT INTO", "DELETE FROM"],
        "invoice_settlements",
    )
    .into_iter()
    .map(|(_, e)| e)
    .collect();
    assert_eq!(ecritures.len(), 2, "{ecritures:#?}");
    assert!(ecritures[0].starts_with("INSERT INTO invoice_settlements (a)"));
    assert!(ecritures[1].starts_with("delete"));
    assert!(ecritures_de_table(source, &["INSERT INTO"], "credit_notes").is_empty());

    // Volets (c) et (d) (revue P1, A-5) : l'INSERT d'avoir suivi — ou non — de
    // la synchronisation dans sa fonction, et l'ordre dissolution → contre-passation.
    let avoirs = r##"
        async fn suivi() {
            sqlx::query("INSERT INTO credit_notes (a) VALUES (1)").execute(t).await?;
            letterings::sync_invoice_in_tx(t).await?;
        }
        async fn orphelin() {
            sqlx::query("INSERT INTO credit_notes (a) VALUES (1)").execute(t).await?;
        }
        async fn annule_bien() {
            dissolve_invoice_document_group_in_tx(t).await?;
            reverse_owned_in_tx(t).await?;
        }
        async fn annule_mal() {
            reverse_owned_in_tx(t).await?;
            // dissolve_invoice_document_group_in_tx( en commentaire : ignoré
            let s = "dissolve_invoice_document_group_in_tx(";
        }
    "##;
    assert_eq!(
        ecritures_et_suite(avoirs, "credit_notes", "sync_invoice_in_tx("),
        vec![("suivi".to_string(), true), ("orphelin".to_string(), false)]
    );
    let ordre = |f: &str| {
        appelle_avant(
            avoirs,
            f,
            "dissolve_invoice_document_group_in_tx(",
            "reverse_owned_in_tx(",
        )
    };
    assert!(ordre("annule_bien").is_ok());
    assert!(
        ordre("annule_mal").is_err(),
        "absente du code : chaîne et commentaire masqués"
    );
    assert!(ordre("inconnue").is_err());
}

// ===========================================================================
// Story 15-1a2-ii (#518) — AC8 part ii : les écrivains du règlement fournisseur.
// ===========================================================================

/// Pour chaque `UPDATE supplier_invoices` de production qui nomme
/// `settlement_journal_entry_id` : la fonction qui le porte, si elle appelle
/// `sync_supplier_invoice_in_tx(` **après** lui, et si elle appelle
/// `dissolve_supplier_invoice_document_group_in_tx(` **avant**
/// `reverse_owned_in_tx(` — `(nom, synchronise_après, dissout_avant)`.
fn ecrivains_du_reglement_fournisseur(source: &str) -> Vec<(String, bool, bool)> {
    let (litteraux, masque) = decouper(source);
    let fns = fonctions(&masque);
    ecritures_de_table(source, &["UPDATE"], "supplier_invoices")
        .into_iter()
        .filter(|(p, _)| {
            litteraux
                .iter()
                .any(|l| l.debut == *p && l.texte.contains("settlement_journal_entry_id"))
        })
        .map(|(p, _)| match fonction_englobante(&fns, p) {
            Some((nom, de, fin)) => {
                let corps = &masque[*de..=*fin];
                let synchronise = masque[p..*fin].contains("sync_supplier_invoice_in_tx(");
                let dissout = matches!(
                    (
                        corps.find("dissolve_supplier_invoice_document_group_in_tx("),
                        corps.find("reverse_owned_in_tx("),
                    ),
                    (Some(d), Some(r)) if d < r
                );
                (nom.clone(), synchronise, dissout)
            }
            None => ("<hors fonction>".to_string(), false, false),
        })
        .collect()
}

/// AC8 (part ii) — chacun des trois `UPDATE supplier_invoices …
/// settlement_journal_entry_id` de production est dans une fonction qui
/// synchronise le lettrage après lui (`pay_in_tx`) ou qui le dissout avant de
/// contre-passer (les deux annulations). Un site neuf rougit en se nommant.
#[test]
fn supplier_settlement_writers_sync_or_dissolve() {
    let mut fautes = Vec::new();
    let mut ecrivains = Vec::new();
    for (fichier, source) in sources_de_production() {
        for (nom, synchronise, dissout) in ecrivains_du_reglement_fournisseur(&source) {
            let attendu = if nom == "pay_in_tx" {
                synchronise
            } else {
                dissout
            };
            if !attendu {
                fautes.push(format!(
                    "{fichier}, fonction `{nom}` : écrit settlement_journal_entry_id sans \
                     `sync_supplier_invoice_in_tx(` après (paiement) ni \
                     `dissolve_supplier_invoice_document_group_in_tx(` avant \
                     `reverse_owned_in_tx(` (annulation)"
                ));
            }
            ecrivains.push(format!("{fichier}::{nom}"));
        }
    }
    assert!(
        fautes.is_empty(),
        "⛔ écrivains du règlement fournisseur (Story 15-1a2-ii, AC8) :\n{}",
        fautes.join("\n")
    );
    ecrivains.sort();
    assert_eq!(
        ecrivains,
        [
            "kesh-db/src/repositories/supplier_invoices.rs::cancel_in_tx",
            "kesh-db/src/repositories/supplier_invoices.rs::cancel_settlement_in_tx",
            "kesh-db/src/repositories/supplier_invoices.rs::pay_in_tx",
        ],
        "les trois écrivains connus — un site neuf se nomme ici"
    );

    // Le détecteur, éprouvé sur un source synthétique (anti-muet).
    let synthetique = r##"
        async fn pay_in_tx() {
            sqlx::query("UPDATE supplier_invoices SET settlement_journal_entry_id = ?").execute(t).await?;
            letterings::sync_supplier_invoice_in_tx(t).await?;
        }
        async fn sync_avant() {
            letterings::sync_supplier_invoice_in_tx(t).await?;
            sqlx::query("UPDATE supplier_invoices SET settlement_journal_entry_id = ?").execute(t).await?;
        }
        async fn annule_bien() {
            dissolve_supplier_invoice_document_group_in_tx(t).await?;
            reverse_owned_in_tx(t).await?;
            sqlx::query("update supplier_invoices set settlement_journal_entry_id = NULL").execute(t).await?;
        }
        async fn annule_mal() {
            reverse_owned_in_tx(t).await?;
            dissolve_supplier_invoice_document_group_in_tx(t).await?;
            sqlx::query("UPDATE supplier_invoices SET settlement_journal_entry_id = NULL").execute(t).await?;
        }
        async fn autre_colonne() {
            sqlx::query("UPDATE supplier_invoices SET version = version + 1").execute(t).await?;
        }
        #[cfg(test)]
        mod tests {
            const F: &str = "UPDATE supplier_invoices SET settlement_journal_entry_id = 1";
        }
    "##;
    assert_eq!(
        ecrivains_du_reglement_fournisseur(synthetique),
        vec![
            ("pay_in_tx".to_string(), true, false),
            ("sync_avant".to_string(), false, false),
            ("annule_bien".to_string(), false, true),
            ("annule_mal".to_string(), false, false),
        ],
        "écrivains vus, ordre exigé, autre colonne et bloc de test ignorés"
    );
}
