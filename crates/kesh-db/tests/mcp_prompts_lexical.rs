//! Story 32-10a-i (refs #723) — gardes lexicales des écritures des prompts du
//! serveur MCP (AC-i14) et liste fermée des créateurs de société (AC-i11).
//!
//! Le classeur est celui de P7, réemployé : `kesh_db::sql_lexical`
//! (`split_statements`, le premier mot-clé, les tables nommées). Pour un `.rs`,
//! la garde extrait d'abord les **littéraux de chaîne**, dans cet ordre (AC-i14,
//! validation P3 F4) : (1) commentaires `//` retirés (un guillemet en prose
//! ouvrirait un faux littéral) ; (2) littéraux extraits (`"…"`, `r"…"`,
//! `r#"…"#`), continuations `\`+fin de ligne résolues ; (3) commentaires de bloc
//! `/* … */` remplacés par une espace, puis `--` retirés ; (4) découpage sur
//! `;`. Chaque garde a un **témoin rouge** et un **témoin vert**, sur source
//! factice par une fonction pure, et des témoins **existants** lus au dépôt.
//!
//! ⚠️ **Angles morts écrits** : un nom de table **composé** à l'exécution
//! (`format!("… {table}")` : la restauration générique `backup.rs`, le vidage
//! de test `test_fixtures.rs`, la remise à zéro `kesh-seed`), une macro qui
//! fabriquerait un nom, une écriture par procédure stockée (`CALL`), et un
//! appelant neuf d'un créateur de société dans un point d'entrée qui n'amorce
//! pas (la liste fermée dit où regarder ; la revue le voit).

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use kesh_db::sql_lexical::{
    colonnes_du_set, est_ecriture, premier_mot_cle, retirer_commentaires_de_bloc,
    split_statements, tables_nommees,
};

// ---------------------------------------------------------------------------
// Outillage : sources, littéraux, statements
// ---------------------------------------------------------------------------

fn racine_crates() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .to_path_buf()
}

/// Les fichiers de `dir` (récursivement) d'extension `ext`, hors `target`.
fn fichiers(dir: &Path, ext: &str, sortie: &mut Vec<PathBuf>) {
    let Ok(entrees) = fs::read_dir(dir) else {
        return;
    };
    for e in entrees.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "target" || n == "node_modules") {
                continue;
            }
            fichiers(&p, ext, sortie);
        } else if p.extension().is_some_and(|x| x == ext) {
            sortie.push(p);
        }
    }
}

fn relatif(p: &Path) -> String {
    p.strip_prefix(racine_crates())
        .unwrap_or(p)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Les littéraux de chaîne d'une source Rust, avec la ligne (1-based) où
/// chacun commence. Commentaires `//` sautés jusqu'à la fin de ligne ;
/// littéraux de caractère sautés (un `'"'` n'ouvre rien) ; échappements
/// résolus, continuation `\`+fin de ligne comprise (elle mange les blancs
/// qui suivent).
fn litteraux_rust(src: &str) -> Vec<(usize, String)> {
    let c: Vec<char> = src.chars().collect();
    let mut sortie = Vec::new();
    let mut ligne = 1usize;
    let mut i = 0usize;
    let est_ident = |x: char| x.is_alphanumeric() || x == '_';
    while i < c.len() {
        let ch = c[i];
        if ch == '\n' {
            ligne += 1;
            i += 1;
        } else if ch == '/' && c.get(i + 1) == Some(&'/') {
            while i < c.len() && c[i] != '\n' {
                i += 1;
            }
        } else if ch == '/' && c.get(i + 1) == Some(&'*') {
            i += 2;
            while i + 1 < c.len() && !(c[i] == '*' && c[i + 1] == '/') {
                if c[i] == '\n' {
                    ligne += 1;
                }
                i += 1;
            }
            i += 2;
        } else if ch == '\'' {
            // Caractère `'x'`, `'\n'`, `'\u{..}'` — ou durée de vie `'a`.
            if c.get(i + 1) == Some(&'\\') {
                i += 2;
                while i < c.len() && c[i] != '\'' {
                    i += 1;
                }
                i += 1;
            } else if c.get(i + 2) == Some(&'\'') {
                i += 3;
            } else {
                i += 1;
            }
        } else if ch == 'r'
            && (i == 0 || !est_ident(c[i - 1]))
            && matches!(c.get(i + 1), Some('"') | Some('#'))
        {
            // Littéral brut : r"…", r#"…"#.
            let mut j = i + 1;
            let mut dieses = 0;
            while c.get(j) == Some(&'#') {
                dieses += 1;
                j += 1;
            }
            if c.get(j) != Some(&'"') {
                i += 1;
                continue;
            }
            let debut_ligne = ligne;
            j += 1;
            let mut s = String::new();
            loop {
                if j >= c.len() {
                    break;
                }
                if c[j] == '"' && (0..dieses).all(|k| c.get(j + 1 + k) == Some(&'#')) {
                    j += 1 + dieses;
                    break;
                }
                if c[j] == '\n' {
                    ligne += 1;
                }
                s.push(c[j]);
                j += 1;
            }
            sortie.push((debut_ligne, s));
            i = j;
        } else if ch == '"' {
            let debut_ligne = ligne;
            let mut j = i + 1;
            let mut s = String::new();
            while j < c.len() && c[j] != '"' {
                if c[j] == '\\' {
                    match c.get(j + 1) {
                        Some('\n') => {
                            ligne += 1;
                            j += 2;
                            while j < c.len() && c[j].is_whitespace() {
                                if c[j] == '\n' {
                                    ligne += 1;
                                }
                                j += 1;
                            }
                            continue;
                        }
                        Some('n') => s.push('\n'),
                        Some('t') => s.push('\t'),
                        Some('r') => s.push('\r'),
                        Some('0') => s.push('\0'),
                        Some(&x) => s.push(x),
                        None => {}
                    }
                    j += 2;
                    continue;
                }
                if c[j] == '\n' {
                    ligne += 1;
                }
                s.push(c[j]);
                j += 1;
            }
            sortie.push((debut_ligne, s));
            i = j + 1;
        } else {
            i += 1;
        }
    }
    sortie
}

/// Les statements SQL d'un texte (littéral ou fichier `.sql`).
fn statements(sql: &str) -> Vec<String> {
    split_statements(&retirer_commentaires_de_bloc(sql))
}

/// Les statements des littéraux d'une source Rust, avec leur ligne.
fn statements_rs(src: &str) -> Vec<(usize, String)> {
    litteraux_rust(src)
        .into_iter()
        .flat_map(|(l, lit)| statements(&lit).into_iter().map(move |s| (l, s)))
        .collect()
}

/// Le **code de production** d'une source : jusqu'à sa première ligne
/// `#[cfg(test)]` suivie — après d'éventuels autres attributs — d'un
/// `mod <nom> {` **à corps**. Une ligne `#[cfg(test)]` qui précède une simple
/// déclaration (`mod x;`) n'est pas une frontière (validation P4, H1).
fn code_de_production(src: &str) -> &str {
    let lignes: Vec<&str> = src.split_inclusive('\n').collect();
    let mut offset = 0usize;
    for (i, l) in lignes.iter().enumerate() {
        if l.trim() == "#[cfg(test)]" {
            let suivante = lignes[i + 1..]
                .iter()
                .map(|x| x.trim())
                .find(|x| !x.is_empty() && !x.starts_with("#["));
            if let Some(s) = suivante {
                let s = s.strip_prefix("pub ").unwrap_or(s);
                if s.starts_with("mod ") && s.contains('{') {
                    return &src[..offset];
                }
            }
        }
        offset += l.len();
    }
    src
}

/// Le nom de la fonction qui englobe la ligne `ligne` (la dernière ligne
/// `fn <nom>` qui la précède).
fn fonction_englobante(src: &str, ligne: usize) -> String {
    src.lines()
        .take(ligne)
        .filter_map(|l| {
            let pos = l.find("fn ")?;
            if pos > 0 && !l[..pos].ends_with(|c: char| c.is_whitespace()) {
                return None;
            }
            let nom: String = l[pos + 3..]
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            (!nom.is_empty()).then_some(nom)
        })
        .last()
        .unwrap_or_default()
}

const TABLES_DES_PROMPTS: [&str; 2] = ["mcp_prompts", "mcp_prompt_versions"];

/// Les écritures d'un ensemble de statements qui nomment une des `tables`.
fn ecritures_sur<'a>(
    stmts: impl IntoIterator<Item = &'a (usize, String)>,
    tables: &[&str],
) -> Vec<(usize, String)> {
    stmts
        .into_iter()
        .filter(|(_, s)| est_ecriture(s))
        .filter(|(_, s)| tables_nommees(s).iter().any(|t| tables.contains(&t.as_str())))
        .cloned()
        .collect()
}

/// Toutes les écritures nommant `tables` dans `crates/` (`.rs` et `.sql`),
/// par fichier relatif.
fn sites_d_ecriture(tables: &[&str]) -> BTreeSet<String> {
    let mut sources = Vec::new();
    fichiers(&racine_crates(), "rs", &mut sources);
    let mut sql = Vec::new();
    fichiers(&racine_crates(), "sql", &mut sql);
    let mut sites = BTreeSet::new();
    for f in sources {
        let src = fs::read_to_string(&f).unwrap();
        if !ecritures_sur(&statements_rs(&src), tables).is_empty() {
            sites.insert(relatif(&f));
        }
    }
    for f in sql {
        let src = fs::read_to_string(&f).unwrap();
        let stmts: Vec<_> = statements(&src).into_iter().map(|s| (0, s)).collect();
        if !ecritures_sur(&stmts, tables).is_empty() {
            sites.insert(relatif(&f));
        }
    }
    sites
}

/// Lit un fichier de `crates/` et rend le statement de ses littéraux qui
/// contient `marqueur` (témoin existant).
fn statement_existant(fichier: &str, marqueur: &str) -> String {
    let src = fs::read_to_string(racine_crates().join(fichier)).unwrap();
    let trouves: Vec<_> = statements_rs(&src)
        .into_iter()
        .map(|(_, s)| s)
        .filter(|s| s.contains(marqueur))
        .collect();
    assert!(!trouves.is_empty(), "témoin introuvable : {fichier} « {marqueur} »");
    trouves[0].clone()
}

// ---------------------------------------------------------------------------
// L'outillage, éprouvé
// ---------------------------------------------------------------------------

#[test]
fn extraction_des_litteraux() {
    let src = "// \"INSERT INTO mcp_prompts\" en prose\n\
               let c = '\"';\n\
               let a = \"UPDATE t \\\n        SET x = 1\";\n\
               let b = r#\"DELETE FROM \"u\" WHERE 1\"#;\n\
               /* \"REPLACE INTO v\" */\n";
    let l = litteraux_rust(src);
    assert_eq!(
        l,
        vec![
            (3, "UPDATE t SET x = 1".to_string()),
            (5, "DELETE FROM \"u\" WHERE 1".to_string()),
        ]
    );
    // Les commentaires de bloc dans un littéral, puis `--`.
    assert_eq!(
        statements("UPDATE /* ; */ t SET a = 1 -- ; fin\n; SELECT 1"),
        vec!["UPDATE   t SET a = 1".to_string(), "SELECT 1".to_string()]
    );
}

#[test]
fn frontiere_du_code_de_production() {
    // Une déclaration `mod x;` après `#[cfg(test)]` n'est pas une frontière.
    let src = "#[cfg(test)]\n#[path = \"x.rs\"]\nmod x;\nfn f() { q(\"INSERT INTO companies (a) VALUES (1)\"); }\n\
               #[cfg(test)]\nmod tests {\n    fn g() { q(\"INSERT INTO companies (b) VALUES (2)\"); }\n}\n";
    let prod = code_de_production(src);
    assert!(prod.contains("fn f()"));
    assert!(!prod.contains("fn g()"));
    assert_eq!(creations_de_societe(prod), vec![(String::new(), "f".to_string())]);
    // Témoins sur le vrai fichier : `bootstrap.rs:47-49` n'est pas une
    // frontière (le code d'`ensure_admin_user` reste de production),
    // `mod tests {` en est une.
    let bootstrap =
        fs::read_to_string(racine_crates().join("kesh-api/src/auth/bootstrap.rs")).unwrap();
    assert!(bootstrap.contains("mod installations_atteintes;"));
    let prod = code_de_production(&bootstrap);
    assert!(prod.contains("pub async fn ensure_admin_user"));
    assert!(!prod.contains("mod tests {"));
    assert!(bootstrap.contains("mod tests {"));
}

// ---------------------------------------------------------------------------
// AC-i14 — les tables des prompts n'ont qu'un écrivain
// ---------------------------------------------------------------------------

/// Les fichiers autorisés à écrire les tables des prompts : leur dépôt, et
/// les tests (`crates/*/tests/`).
fn autorise(fichier: &str) -> bool {
    fichier == "kesh-db/src/repositories/mcp_prompts.rs" || fichier.contains("/tests/")
}

#[test]
fn tables_des_prompts_ecrites_par_leur_depot() {
    let sites = sites_d_ecriture(&TABLES_DES_PROMPTS);
    let hors: Vec<_> = sites.iter().filter(|f| !autorise(f)).collect();
    assert!(
        hors.is_empty(),
        "écriture des tables des prompts hors de leur dépôt : {hors:?}"
    );
    // Témoin : la garde voit le dépôt lui-même (elle ne passe pas à vide).
    assert!(
        sites.contains("kesh-db/src/repositories/mcp_prompts.rs"),
        "{sites:?}"
    );
    // Mesure sur l'analogue existant (`mcp_keys`), même classeur : hors des
    // tests, ses écritures ne vivent qu'en `repositories/mcp_keys.rs`.
    let analogue: Vec<_> = sites_d_ecriture(&["mcp_keys"])
        .into_iter()
        .filter(|f| !f.contains("/tests/"))
        .collect();
    assert_eq!(analogue, ["kesh-db/src/repositories/mcp_keys.rs"]);
}

#[test]
fn tables_des_prompts_temoins() {
    // Rouges factices, chacun comme dans une source de `routes/`.
    let rouges = [
        "q(\"INSERT LOW_PRIORITY INTO mcp_prompts (a) VALUES (1)\")",
        "q(\"INSERT mcp_prompts (a) VALUES (1)\")",
        "q(\"REPLACE mcp_prompt_versions (a) VALUES (1)\")",
        "q(\"DELETE v FROM mcp_prompt_versions v WHERE v.id = 1\")",
        "q(\"TRUNCATE TABLE mcp_prompts\")",
        "q(\"UPDATE mcp_prompts p JOIN mcp_prompt_versions v ON v.prompt_id = p.id SET v.body = ?\")",
        "q(\"INSERT INTO \\\n    mcp_prompts (a) VALUES (1)\")",
        "q(\"UPDATE IGNORE mcp_prompts SET version = 1\")",
        "q(\"DELETE QUICK FROM mcp_prompt_versions\")",
        "q(\"INSERT INTO mcp_prompts (a) VALUES (1) ON DUPLICATE KEY UPDATE a = 2\")",
        "q(r#\"WITH x AS (SELECT 1) DELETE FROM mcp_prompts\"#)",
    ];
    for r in rouges {
        assert_eq!(
            ecritures_sur(&statements_rs(r), &TABLES_DES_PROMPTS).len(),
            1,
            "rouge non vu : {r}"
        );
    }
    // Verts factices : lecture, DDL, prose.
    for v in [
        "q(\"SELECT * FROM mcp_prompts\")",
        "// INSERT INTO mcp_prompts en prose\nq(\"SELECT 1\")",
        "q(\"CREATE TABLE x (p BIGINT REFERENCES mcp_prompts(id) ON DELETE RESTRICT)\")",
    ] {
        assert!(
            ecritures_sur(&statements_rs(v), &TABLES_DES_PROMPTS).is_empty(),
            "vert pris : {v}"
        );
    }
    // Verts existants : le `CREATE TABLE … ON DELETE RESTRICT` de la migration
    // des clés n'est pas une écriture ; l'`INSERT … ON DUPLICATE KEY UPDATE` de
    // `journal_entries.rs` en est une, sur `fiscal_years` seulement.
    let migration = fs::read_to_string(
        racine_crates().join("kesh-db/migrations/20261025000001_mcp_keys_and_server_settings.sql"),
    )
    .unwrap();
    let ddl: Vec<_> = statements(&migration);
    assert!(ddl.iter().any(|s| s.contains("ON DELETE RESTRICT")));
    assert!(ddl.iter().all(|s| !est_ecriture(s)));
    let s = statement_existant(
        "kesh-db/src/repositories/journal_entries.rs",
        "ON DUPLICATE KEY UPDATE status = 'Closed'",
    );
    assert!(est_ecriture(&s));
    assert_eq!(tables_nommees(&s), ["fiscal_years"]);
    // Rouges existants, sur leur table : multi-tables à alias, nom composé.
    let s = statement_existant("kesh-db/tests/open_items.rs", "UPDATE journal_entry_lines jel");
    assert_eq!(tables_nommees(&s), ["journal_entry_lines", "journal_entries"]);
    let s = statement_existant("kesh-db/src/test_fixtures.rs", "TRUNCATE TABLE");
    assert!(est_ecriture(&s));
    assert_eq!(tables_nommees(&s), ["{table}"]);
    let s = statement_existant("kesh-db/src/repositories/companies.rs", "t SET t.company_id");
    assert!(est_ecriture(&s));
    assert_eq!(tables_nommees(&s), ["{table}"]);
    // `WITH` de lecture existants : pas des écritures.
    for (f, m) in [
        ("kesh-db/tests/letterings.rs", "WITH pieces AS"),
        ("kesh-db/tests/support/lettering_documents.rs", "WITH ancre AS"),
    ] {
        let s = statement_existant(f, m);
        assert_eq!(premier_mot_cle(&s), "WITH");
        assert!(!est_ecriture(&s), "{f}");
    }
}

// ---------------------------------------------------------------------------
// AC-i14 — les versions ne se réécrivent jamais
// ---------------------------------------------------------------------------

const COLONNES_DE_DECISION: [&str; 3] = ["state", "decided_by_user_id", "decided_at"];

/// Les écritures de `mcp_prompt_versions` d'une source qui ne sont ni un
/// `INSERT` sans `ON DUPLICATE KEY UPDATE`, ni un `UPDATE` des seules colonnes
/// de décision.
fn reecritures(src: &str) -> Vec<String> {
    ecritures_sur(&statements_rs(src), &["mcp_prompt_versions"])
        .into_iter()
        .map(|(_, s)| s)
        .filter(|s| {
            let haut = s.to_ascii_uppercase();
            match premier_mot_cle(s).as_str() {
                "INSERT" => haut.contains("DUPLICATE"),
                "UPDATE" => !colonnes_du_set(s)
                    .unwrap_or_default()
                    .iter()
                    .all(|c| COLONNES_DE_DECISION.contains(&c.as_str())),
                _ => true,
            }
        })
        .collect()
}

#[test]
fn versions_jamais_reecrites() {
    let src = fs::read_to_string(racine_crates().join("kesh-db/src/repositories/mcp_prompts.rs"))
        .unwrap();
    assert_eq!(reecritures(&src), Vec::<String>::new());
    // Témoin : le dépôt écrit bien cette table (un `INSERT`, un `UPDATE`).
    let ecritures = ecritures_sur(&statements_rs(&src), &["mcp_prompt_versions"]);
    let verbes: BTreeSet<_> = ecritures.iter().map(|(_, s)| premier_mot_cle(s)).collect();
    assert_eq!(verbes, BTreeSet::from(["INSERT".to_string(), "UPDATE".to_string()]));
    // Rouges factices.
    for r in [
        "q(\"UPDATE mcp_prompt_versions SET state = ?, body = ? WHERE id = ?\")",
        "q(\"UPDATE mcp_prompts p JOIN mcp_prompt_versions v ON v.prompt_id = p.id SET v.body = ?\")",
        "q(\"INSERT INTO mcp_prompt_versions (a) VALUES (1) ON DUPLICATE KEY UPDATE body = VALUES(body)\")",
        "q(\"REPLACE INTO mcp_prompt_versions (a) VALUES (1)\")",
        "q(\"DELETE FROM mcp_prompt_versions WHERE id = 1\")",
    ] {
        assert_eq!(reecritures(r).len(), 1, "rouge non vu : {r}");
    }
    // Verts factices : la décision, l'insertion.
    for v in [
        "q(\"UPDATE mcp_prompt_versions SET state = ?, decided_by_user_id = ?, decided_at = NOW(3) WHERE id = ?\")",
        "q(\"INSERT INTO mcp_prompt_versions (a) VALUES (1)\")",
    ] {
        assert!(reecritures(v).is_empty(), "vert pris : {v}");
    }
    // Verts existants de l'extracteur du `SET` (multi-ligne, alias retiré).
    let s = statement_existant("kesh-db/src/repositories/mcp_keys.rs", "SET revoked_at = NOW(3)");
    assert_eq!(
        colonnes_du_set(&s),
        Some(vec!["revoked_at".to_string(), "version".to_string()])
    );
    let s = statement_existant("kesh-db/src/repositories/companies.rs", "t SET t.company_id");
    assert_eq!(colonnes_du_set(&s), Some(vec!["company_id".to_string()]));
}

// ---------------------------------------------------------------------------
// AC-i11 — la liste fermée des créateurs de société
// ---------------------------------------------------------------------------

/// Les `(fichier, fonction)` d'une source de production dont une instruction
/// `INSERT`, `REPLACE` ou `LOAD` nomme `companies`. `UPDATE`, `DELETE` et
/// `TRUNCATE` ne créent pas de société (validation P4, H2).
fn creations_de_societe(prod: &str) -> Vec<(String, String)> {
    statements_rs(prod)
        .into_iter()
        .filter(|(_, s)| matches!(premier_mot_cle(s).as_str(), "INSERT" | "REPLACE" | "LOAD"))
        .filter(|(_, s)| tables_nommees(s).iter().any(|t| t == "companies"))
        .map(|(l, _)| (String::new(), fonction_englobante(prod, l)))
        .collect()
}

#[test]
fn creations_de_societe_liste_fermee() {
    let mut sources = Vec::new();
    fichiers(&racine_crates(), "rs", &mut sources);
    let mut trouvees = BTreeSet::new();
    for f in sources {
        let rel = relatif(&f);
        // Code de production seulement : `crates/*/src/`.
        if !rel.split('/').nth(1).is_some_and(|d| d == "src") {
            continue;
        }
        let src = fs::read_to_string(&f).unwrap();
        for (_, fonction) in creations_de_societe(code_de_production(&src)) {
            trouvees.insert((rel.clone(), fonction));
        }
    }
    let attendues: BTreeSet<(String, String)> = [
        ("kesh-db/src/repositories/companies.rs", "create"),
        ("kesh-db/src/repositories/companies.rs", "insert_stub"),
        ("kesh-db/src/test_fixtures.rs", "seed_accounting_company"),
        ("kesh-db/src/test_fixtures.rs", "seed_accounting_company_no_fy"),
        ("kesh-db/src/test_fixtures.rs", "seed_changeme_user_only"),
        ("kesh-db/src/test_fixtures.rs", "seed_stub_company_only"),
    ]
    .into_iter()
    .map(|(f, n)| (f.to_string(), n.to_string()))
    .collect();
    assert_eq!(
        trouvees, attendues,
        "une fonction neuve crée une société : elle doit amorcer les prompts standard \
         (AC-i11) — ajouter son point d'entrée à la liste et un appel d'amorçage"
    );
}

#[test]
fn creations_de_societe_temoins() {
    // Rouge factice : une fonction de production qui crée une société.
    let rouge = "pub async fn neuve(pool: &P) {\n    q(\"INSERT INTO companies (name) VALUES (?)\");\n}\n";
    assert_eq!(creations_de_societe(rouge), vec![(String::new(), "neuve".to_string())]);
    for forme in [
        "INSERT IGNORE INTO `companies` (a) VALUES (1)",
        "REPLACE companies (a) VALUES (1)",
        "INSERT companies (a) VALUES (1)",
    ] {
        let src = format!("fn f() {{ q(\"{forme}\"); }}");
        assert_eq!(creations_de_societe(&src).len(), 1, "{forme}");
    }
    // Verts existants : un message dont le premier mot n'est pas un verbe
    // SQL, et un `UPDATE companies`, ne sont pas comptés.
    let companies =
        fs::read_to_string(racine_crates().join("kesh-db/src/repositories/companies.rs")).unwrap();
    assert!(companies.contains("last_insert_id invalide après INSERT companies"));
    assert!(companies.contains("\"UPDATE companies SET"));
    let comptees = creations_de_societe(code_de_production(&companies));
    assert_eq!(
        comptees,
        vec![
            (String::new(), "create".to_string()),
            (String::new(), "insert_stub".to_string())
        ]
    );
    // Les sept `INSERT INTO companies` du module de tests de `bootstrap.rs` ne
    // sont pas comptés (frontière `mod tests {`).
    let bootstrap =
        fs::read_to_string(racine_crates().join("kesh-api/src/auth/bootstrap.rs")).unwrap();
    assert!(creations_de_societe(&bootstrap).len() >= 7, "témoin : le fichier entier les porte");
    assert!(creations_de_societe(code_de_production(&bootstrap)).is_empty());
}
