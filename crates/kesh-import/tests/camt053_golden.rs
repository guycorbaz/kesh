//! Caractérisation de la sortie du parseur CAMT.053 (Story v017-1, #789, AC9).
//!
//! **Pourquoi ce test existe.** La montée de `quick-xml` (0.39 → 0.42) ne doit
//! changer le sens d'aucun relevé déjà importé en 0.16 : la clé de doublon de
//! ligne (`DuplicateKey`, `kesh-core`) dérive de la sortie du parseur — date,
//! montant, références. Les références `fixtures/camt053_golden/*.json` ont été
//! **produites sous `quick-xml` 0.39.4**, dans un commit qui précède celui de la
//! montée et ne touche aucun code de production ; ce test exige ensuite l'égalité
//! stricte, valeur par valeur.
//!
//! **Ce qui est comparé.**
//! - Un `Ok(stmts)` : `serde_json::to_string_pretty` de la sortie **entière**.
//! - Un `Err` : le nom du variant et sa charge entière — sauf les
//!   `MalformedXml` qui relaient le message d'une erreur de `quick-xml`
//!   (`e.to_string()`), comparés au variant seul : ce message change d'une
//!   version à l'autre sans qu'aucune donnée ne change, et il n'est ni stocké ni
//!   montré tel quel (`BANK_IMPORT_MALFORMED_XML`). Les messages rédigés par Kesh
//!   se reconnaissent à [`KESH_MESSAGES`] ; le garde
//!   [`inventory_of_kesh_messages_is_closed`] tient cette liste close.
//!
//! **Une seule divergence admise** (C-v017-1-11) : `v04_latin1_hors_texte.xml`,
//! un fichier non UTF-8 dont les octets fautifs sont hors du texte lu. Sous 0.39
//! il se lit (référence `v04_latin1_hors_texte.0.39.json`) ; sous toute autre
//! version, le lecteur valide l'UTF-8 de toute l'entrée et le refuse en
//! `MalformedXml`. L'exception est **nominative** et **conditionnée par la
//! version de `quick-xml` lue au `Cargo.lock`**.
//!
//! **Régénération** (`KESH_GOLDEN_REGEN=1`) : verrouillée. Elle panique si la
//! variable `CI` est définie, ou si le `Cargo.lock` ne résout pas `quick-xml` en
//! `0.39.x` — une référence ne se produit jamais sous la version qu'elle juge.
//! Hors de ce mode, une référence manquante fait rougir le test.

use std::path::{Path, PathBuf};

use kesh_import::{CamtError, ImportedStatement, parse_camt053};
use regex::Regex;
use serde_json::{Value, json};

/// Fixtures du crate (`tests/fixtures/camt053/*.xml`).
const CRATE_FIXTURES: &str = "tests/fixtures/camt053";
/// Fixtures E2E du frontend (`frontend/tests/e2e/fixtures/camt053_*.xml`), relatives au crate.
const FRONTEND_FIXTURES: &str = "../../frontend/tests/e2e/fixtures";
/// Dossier des références — frère de `camt053/`, pour ne pas fausser le
/// décompte `ls camt053/` de `camt053_tests.rs` et du `README.md`.
const GOLDEN_DIR: &str = "tests/fixtures/camt053_golden";

/// La seule fixture dont la lecture dépend de la version (C-v017-1-11).
const UTF8_EXCEPTION: &str = "v04_latin1_hors_texte";
/// Sa référence, produite sous 0.39 et nommée comme telle.
const UTF8_EXCEPTION_REFERENCE: &str = "v04_latin1_hors_texte.0.39.json";

/// Nombres minimaux de fixtures attendues : 14 + 5 neuves (Story v017-1) dans
/// le crate, 4 au frontend. Garde contre un test qui passerait à vide (dossier
/// déplacé, motif de fichier faux).
const MIN_CRATE_FIXTURES: usize = 19;
const MIN_FRONTEND_FIXTURES: usize = 4;

/// Les messages `MalformedXml` **rédigés par Kesh** (`src/camt053/mod.rs`),
/// comparés entiers ; tout autre `MalformedXml` relaie une erreur de
/// bibliothèque et se compare au variant seul. Liste close : le garde
/// [`inventory_of_kesh_messages_is_closed`] compte les sites du source.
const KESH_MESSAGES: [&str; 5] = [
    "élément racine <Document> introuvable",
    "préfixe namespace non lié : ",
    "fin de fichier inattendue, éléments non fermés : ",
    ".CdtDbtInd inattendu : ",
    "nom de tag non UTF-8 : ",
];

/// Version de `quick-xml` résolue par le `Cargo.lock` du workspace.
fn locked_quick_xml_version() -> String {
    let lock_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.lock");
    let lock = std::fs::read_to_string(&lock_path)
        .unwrap_or_else(|e| panic!("Cargo.lock illisible ({}) : {e}", lock_path.display()));
    let mut lines = lock.lines();
    let mut versions = Vec::new();
    while let Some(line) = lines.next() {
        if line.trim() == "name = \"quick-xml\"" {
            let v = lines
                .next()
                .and_then(|l| l.trim().strip_prefix("version = \""))
                .and_then(|l| l.strip_suffix('"'))
                .expect("ligne version après name = \"quick-xml\"");
            versions.push(v.to_string());
        }
    }
    assert_eq!(
        versions.len(),
        1,
        "le Cargo.lock doit porter une seule version de quick-xml : {versions:?}"
    );
    versions.remove(0)
}

fn is_quick_xml_039() -> bool {
    locked_quick_xml_version().starts_with("0.39.")
}

/// Mode de régénération, verrouillé (voir l'en-tête du module).
fn regen_mode() -> bool {
    if std::env::var("KESH_GOLDEN_REGEN").as_deref() != Ok("1") {
        return false;
    }
    assert!(
        std::env::var_os("CI").is_none(),
        "KESH_GOLDEN_REGEN interdit en CI : une référence ne se régénère pas en intégration continue"
    );
    let v = locked_quick_xml_version();
    assert!(
        v.starts_with("0.39."),
        "KESH_GOLDEN_REGEN refusé : les références se produisent sous quick-xml 0.39.x, le Cargo.lock résout {v}"
    );
    true
}

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Les fichiers d'entrée, triés : `(nom de la référence sans extension, chemin)`.
fn inputs() -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    let mut crate_count = 0;
    let mut frontend_count = 0;
    for (dir, prefix, counter) in [
        (CRATE_FIXTURES, "", &mut crate_count),
        (FRONTEND_FIXTURES, "camt053_", &mut frontend_count),
    ] {
        let dir = crate_dir().join(dir);
        let mut found: Vec<PathBuf> = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("dossier {} illisible : {e}", dir.display()))
            .map(|e| e.expect("entrée de dossier").path())
            .filter(|p| {
                p.extension().is_some_and(|x| x == "xml")
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.starts_with(prefix))
            })
            .collect();
        found.sort();
        *counter = found.len();
        for p in found {
            let stem = p.file_stem().unwrap().to_str().unwrap().to_string();
            out.push((stem, p));
        }
    }
    assert!(
        crate_count >= MIN_CRATE_FIXTURES,
        "{crate_count} fixtures CAMT.053 au crate, {MIN_CRATE_FIXTURES} au moins attendues"
    );
    assert!(
        frontend_count >= MIN_FRONTEND_FIXTURES,
        "{frontend_count} fixtures CAMT.053 au frontend, {MIN_FRONTEND_FIXTURES} au moins attendues"
    );
    let stems: std::collections::HashSet<&String> = out.iter().map(|(s, _)| s).collect();
    assert_eq!(stems.len(), out.len(), "deux fixtures portent le même nom");
    out
}

/// Vrai si le message est l'un de ceux que Kesh rédige.
fn is_kesh_message(msg: &str) -> bool {
    KESH_MESSAGES.iter().any(|m| msg.contains(m))
}

/// Représentation comparée d'un résultat du parseur.
fn characterize(result: &Result<Vec<ImportedStatement>, CamtError>) -> Value {
    match result {
        Ok(stmts) => json!({ "ok": stmts }),
        Err(e) => {
            let (variant, payload) = match e {
                CamtError::MalformedXml(m) if is_kesh_message(m) => ("MalformedXml", Some(m)),
                // Relais d'une erreur de bibliothèque : variant seul.
                CamtError::MalformedXml(_) => ("MalformedXml", None),
                CamtError::UnsupportedVersion(m) => ("UnsupportedVersion", Some(m)),
                CamtError::MissingRequiredField(m) => ("MissingRequiredField", Some(m)),
                CamtError::InvalidAmount(m) => ("InvalidAmount", Some(m)),
                CamtError::InvalidDate(m) => ("InvalidDate", Some(m)),
            };
            json!({ "err": { "variant": variant, "payload": payload } })
        }
    }
}

fn render(v: &Value) -> String {
    let mut s = serde_json::to_string_pretty(v).expect("sérialisation JSON");
    s.push('\n');
    s
}

#[test]
fn parser_output_matches_references_produced_under_quick_xml_039() {
    let regen = regen_mode();
    let golden_dir = crate_dir().join(GOLDEN_DIR);
    if regen {
        std::fs::create_dir_all(&golden_dir).expect("création du dossier des références");
    }
    let is_039 = is_quick_xml_039();
    let mut failures = Vec::new();
    let mut checked = 0;

    for (stem, path) in inputs() {
        let bytes = std::fs::read(&path).expect("lecture de la fixture");
        let result = parse_camt053(&bytes);
        let actual = render(&characterize(&result));

        let reference_name = if stem == UTF8_EXCEPTION {
            UTF8_EXCEPTION_REFERENCE.to_string()
        } else {
            format!("{stem}.json")
        };
        let reference_path = golden_dir.join(&reference_name);

        if regen {
            std::fs::write(&reference_path, &actual).expect("écriture de la référence");
            checked += 1;
            continue;
        }

        if stem == UTF8_EXCEPTION && !is_039 {
            // Divergence admise (C-v017-1-11) : refus strict, rien d'autre.
            if !matches!(result, Err(CamtError::MalformedXml(_))) {
                failures.push(format!(
                    "{stem} : Err(MalformedXml) attendu hors de quick-xml 0.39, obtenu {result:?}"
                ));
            }
            checked += 1;
            continue;
        }

        let expected = std::fs::read_to_string(&reference_path).unwrap_or_else(|e| {
            panic!(
                "référence {} absente ({e}) : elle se produit sous quick-xml 0.39 (KESH_GOLDEN_REGEN=1), jamais après la montée",
                reference_path.display()
            )
        });
        if expected != actual {
            failures.push(format!(
                "{stem} : la sortie du parseur diffère de sa référence {reference_name}\n--- attendu\n{expected}--- obtenu\n{actual}"
            ));
        }
        checked += 1;
    }

    assert!(
        checked >= MIN_CRATE_FIXTURES + MIN_FRONTEND_FIXTURES,
        "{checked} fichiers comparés"
    );
    assert!(
        failures.is_empty(),
        "{} divergence(s) — classe « interdite » ou « à rapporter » (fiche v017-1, AC9) ; \
         ne JAMAIS régénérer une référence pour la faire disparaître :\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Chaque référence correspond à une entrée : une référence orpheline (fixture
/// renommée ou retirée) se voit au lieu de dormir.
#[test]
fn every_reference_has_an_input() {
    let stems: Vec<String> = inputs().into_iter().map(|(s, _)| s).collect();
    let golden_dir = crate_dir().join(GOLDEN_DIR);
    for entry in std::fs::read_dir(&golden_dir).expect("dossier des références") {
        let name = entry.unwrap().file_name().into_string().unwrap();
        let stem = if name == UTF8_EXCEPTION_REFERENCE {
            UTF8_EXCEPTION.to_string()
        } else {
            name.strip_suffix(".json")
                .unwrap_or_else(|| panic!("fichier inattendu dans camt053_golden : {name}"))
                .to_string()
        };
        assert!(stems.contains(&stem), "référence orpheline : {name}");
    }
}

fn load_crate_fixture(name: &str) -> Vec<u8> {
    std::fs::read(crate_dir().join(CRATE_FIXTURES).join(name)).expect("fixture")
}

/// Les formes à figer atteignent bien la sortie : marqueurs lus **dans les
/// champs** (non par recherche de sous-chaîne — `CHF` est aussi la devise du
/// relevé). Sans eux, une fixture qui ne traverserait pas le parseur jusqu'à ces
/// valeurs ferait une caractérisation vide.
#[test]
fn characterization_fixture_reaches_the_forms_it_freezes() {
    let stmts = parse_camt053(&load_crate_fixture("v04_characterization.xml"))
        .expect("v04_characterization se lit");
    assert_eq!(stmts.len(), 1);
    let txs = &stmts[0].transactions;
    assert_eq!(
        txs.len(),
        2,
        "deux transactions : Ntry n° 1, TxDtls du Ntry n° 2"
    );
    // Ntry n° 1 : `Ccy="C&#72;F"` — référence de caractère dans une valeur d'attribut.
    assert_eq!(txs[0].currency, "CHF");
    // Ntry n° 2 : `Ccy` du TxDtls/Amt à tabulation littérale, NON normalisée.
    assert_eq!(txs[1].currency, "C\tF");
    // Saut de ligne CRLF intérieur à `Ustrd`, NON normalisé.
    assert!(
        txs[1].details.contains("\r\n"),
        "CRLF intérieur attendu dans details : {:?}",
        txs[1].details
    );
    // Les sorties fautives des entités et du CDATA (comportement de 0.16, défaut
    // tracé par #819 ; ne pas corriger ici) sont figées par la référence JSON.
}

/// Un nom accentué encodé en Latin-1 **dans un texte lu** est refusé, sous les
/// deux versions (aucune fonctionnalité `encoding` n'est activée).
#[test]
fn latin1_in_read_text_is_malformed() {
    let r = parse_camt053(&load_crate_fixture("v04_latin1.xml"));
    assert!(
        matches!(r, Err(CamtError::MalformedXml(_))),
        "Err(MalformedXml) attendu, obtenu {r:?}"
    );
}

/// Retire les commentaires `//…` et `/* … */` d'un source Rust, en laissant les
/// chaînes intactes (un `//` dans une chaîne n'ouvre pas de commentaire).
fn strip_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;
    let mut in_string = false;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        if in_string {
            out.push(c);
            if c == '\\' {
                if let Some(n) = next {
                    out.push(n);
                    i += 2;
                    continue;
                }
            } else if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match (c, next) {
            ('"', _) => {
                in_string = true;
                out.push(c);
                i += 1;
            }
            ('/', Some('/')) => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            ('/', Some('*')) => {
                i += 2;
                while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                i += 2;
            }
            _ => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

/// Arguments de chaque `CamtError::MalformedXml(` du source, jusqu'à la
/// parenthèse fermante appariée.
fn malformed_xml_arguments(src: &str) -> Vec<String> {
    const SITE: &str = "CamtError::MalformedXml(";
    let mut args = Vec::new();
    let mut rest = src;
    while let Some(pos) = rest.find(SITE) {
        let after = &rest[pos + SITE.len()..];
        let mut depth = 1usize;
        let mut end = None;
        for (j, c) in after.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(j);
                        break;
                    }
                }
                _ => {}
            }
        }
        let end = end.expect("parenthèse fermante appariée");
        args.push(after[..end].to_string());
        rest = &after[end..];
    }
    args
}

/// Garde d'inventaire (AC9, C-v017-1-3) : les sites `CamtError::MalformedXml(`
/// de `src/camt053/mod.rs` qui ne sont pas un relais `<var>.to_string()` sont
/// exactement les messages de Kesh de [`KESH_MESSAGES`]. Un sixième message
/// rédigé fait rougir ce garde au lieu de passer, en silence, en « variant
/// seul ». Un site en commentaire ne compte pas.
#[test]
fn inventory_of_kesh_messages_is_closed() {
    let src = strip_comments(include_str!("../src/camt053/mod.rs"));
    let relay = Regex::new(r"^\s*\w+\s*\.\s*to_string\(\)\s*,?\s*$").unwrap();
    let args = malformed_xml_arguments(&src);
    let kesh_sites: Vec<&String> = args.iter().filter(|a| !relay.is_match(a)).collect();
    assert!(
        args.len() > kesh_sites.len(),
        "aucun relais reconnu : l'expression ne voit plus rien ({args:?})"
    );
    assert_eq!(
        kesh_sites.len(),
        KESH_MESSAGES.len(),
        "sites MalformedXml rédigés par Kesh : {kesh_sites:#?} — mettre KESH_MESSAGES à jour"
    );
    for m in KESH_MESSAGES {
        assert!(
            kesh_sites.iter().any(|a| a.contains(m)),
            "le message {m:?} n'est plus écrit dans mod.rs"
        );
    }
}
