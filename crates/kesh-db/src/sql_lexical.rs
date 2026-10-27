//! Analyse lexicale **naïve** du SQL du dépôt — le classeur partagé par le
//! triage P7 des migrations (`post_restore.rs`) et par les gardes des
//! écritures des prompts MCP (`tests/mcp_prompts_lexical.rs`, Story 32-10a-i,
//! AC-i14).
//!
//! **Le principe est celui de P7** : classer un statement sur son **premier
//! mot-clé**, ce qui donne de la largeur sans énumération à maintenir (une
//! forme imprévue d'`INSERT` reste un `INSERT`). Seul [`split_statements`] est
//! partagé avec `post_restore.rs` ; le prédicat des gardes, [`est_ecriture`],
//! est **distinct** du prédicat du triage P7 (`writes_data`, qui reste privé à
//! `post_restore.rs` sous `#[cfg(test)]`, validation P4 H8).
//!
//! Module **outil de garde**, `#[doc(hidden)] pub` : il n'est employé en
//! production que par `post_restore.rs`.

/// Retire les commentaires `--` (en ligne entière comme en fin de ligne) et
/// découpe le SQL en statements sur les `;`.
///
/// **Le découpage doit être multi-ligne** : plusieurs backfills du dépôt portent
/// leur `SELECT` sur la ligne suivant l'`INSERT`, et un détecteur mono-ligne les
/// rate — c'est ainsi qu'une migration exposée a échappé au triage manuel.
///
/// **Le retrait des commentaires n'est pas cosmétique** : les migrations sont
/// très commentées et plusieurs commentaires contiennent le mot `UPDATE` en
/// prose. Attention en particulier à `ON UPDATE CURRENT_TIMESTAMP`, présent dans
/// une vingtaine de migrations — c'est du DDL, pas un statement `UPDATE`.
///
/// # Deux limites, et elles sont OUTILLÉES et non simplement déclarées
///
/// Cet analyseur est **volontairement naïf** : il ne connaît ni les commentaires
/// de bloc `/* */`, ni les littéraux entre apostrophes. Les deux limites sont
/// tenues par un test chacune (`post_restore.rs`), parce qu'une hypothèse sur
/// le contenu du dépôt qui n'échoue pas bruyamment quand elle cesse d'être
/// vraie est exactement le « garde-fou muet » que ce module existe pour
/// empêcher.
///
/// 1. **`/* */` — risque de FAUX NÉGATIF au triage.** Un `UPDATE` enfermé dans un
///    commentaire de bloc, ou un `;` à l'intérieur d'un tel bloc, décale le
///    découpage et peut faire rendre `false` au prédicat du triage P7
///    (`writes_data`) : la migration échapperait alors au triage **en
///    silence**. Tenu par `migrations_contain_no_block_comment`. Les gardes des
///    prompts, elles, passent d'abord [`retirer_commentaires_de_bloc`].
/// 2. **`--` ou `;` dans un littéral — risque de SQL CORROMPU à l'exécution.** La
///    troncature au premier `--` et la découpe au premier `;` ignorent les
///    apostrophes ; un `SET label = 'poste -- 4000'` partirait tronqué à MariaDB.
///    Le dépôt contient **déjà** un `;` littéral (`20260505000001_bank_profiles`,
///    `IN (',', ';', '\t')`), inoffensif parce que ce fichier n'est pas rejoué —
///    ce qui montre que le risque n'est pas théorique. Tenu, **sur le SQL
///    réellement exécuté** et lui seul, par `registry_sql_has_no_literal_hazard`.
pub fn split_statements(sql: &str) -> Vec<String> {
    let stripped: String = sql
        .lines()
        .map(|line| match line.find("--") {
            Some(idx) => &line[..idx],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n");

    stripped
        .split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Remplace chaque commentaire de bloc `/* … */` par une espace (un bloc non
/// fermé court jusqu'à la fin). Passe propre aux gardes des prompts, que le
/// triage P7 n'emploie pas.
pub fn retirer_commentaires_de_bloc(sql: &str) -> String {
    let mut sortie = String::with_capacity(sql.len());
    let mut reste = sql;
    while let Some(debut) = reste.find("/*") {
        sortie.push_str(&reste[..debut]);
        sortie.push(' ');
        match reste[debut + 2..].find("*/") {
            Some(fin) => reste = &reste[debut + 2 + fin + 2..],
            None => return sortie,
        }
    }
    sortie.push_str(reste);
    sortie
}

/// Un jeton du classeur : un mot (identifiant, mot-clé, `{nom}` composé), un
/// littéral entre apostrophes, ou une ponctuation.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Jeton {
    Mot(String),
    Litteral,
    Ponct(char),
}

fn est_car_de_mot(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

/// Découpe un statement en jetons. Les identifiants entre accents graves
/// perdent leurs accents ; un nom qualifié `a.b` devient `b` ; un `{…}` (nom
/// composé par `format!`) est un mot.
fn jetons(statement: &str) -> Vec<Jeton> {
    let chars: Vec<char> = statement.chars().collect();
    let mut sortie: Vec<Jeton> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
        } else if c == '\'' || c == '"' {
            // Littéral : jusqu'à la même apostrophe non doublée.
            let q = c;
            i += 1;
            while i < chars.len() {
                if chars[i] == '\\' {
                    i += 2;
                    continue;
                }
                if chars[i] == q {
                    if chars.get(i + 1) == Some(&q) {
                        i += 2;
                        continue;
                    }
                    i += 1;
                    break;
                }
                i += 1;
            }
            sortie.push(Jeton::Litteral);
        } else if c == '`' || c == '{' {
            let fin = if c == '`' { '`' } else { '}' };
            let debut = i;
            i += 1;
            while i < chars.len() && chars[i] != fin {
                i += 1;
            }
            let mot: String = if c == '`' {
                chars[debut + 1..i.min(chars.len())].iter().collect()
            } else {
                chars[debut..(i + 1).min(chars.len())].iter().collect()
            };
            i += 1;
            sortie.push(Jeton::Mot(mot));
        } else if est_car_de_mot(c) {
            let debut = i;
            while i < chars.len() && est_car_de_mot(chars[i]) {
                i += 1;
            }
            sortie.push(Jeton::Mot(chars[debut..i].iter().collect()));
        } else {
            sortie.push(Jeton::Ponct(c));
            i += 1;
        }
        // Un nom qualifié `a.b` : ne garder que `b`.
        if let [.., Jeton::Mot(_), Jeton::Ponct('.'), Jeton::Mot(_)] = sortie.as_slice() {
            let dernier = sortie.pop();
            sortie.pop();
            sortie.pop();
            if let Some(d) = dernier {
                sortie.push(d);
            }
        }
    }
    sortie
}

/// Le premier mot-clé d'un statement, en capitales (`""` s'il n'en a pas).
pub fn premier_mot_cle(statement: &str) -> String {
    match jetons(statement).into_iter().next() {
        Some(Jeton::Mot(m)) => m.to_ascii_uppercase(),
        _ => String::new(),
    }
}

const VERBES_ECRITURE: &[&str] = &["INSERT", "REPLACE", "UPDATE", "DELETE", "TRUNCATE", "LOAD"];

/// Vrai si le statement **écrit** : premier mot-clé `INSERT`, `REPLACE`,
/// `UPDATE`, `DELETE`, `TRUNCATE` ou `LOAD` (toutes formes) ; ou premier mot-clé
/// `WITH` (CTE) suivi, **hors parenthèses**, d'un de ces verbes avant tout
/// `SELECT`. Un `CREATE TABLE … ON DELETE RESTRICT` ou `ON UPDATE
/// CURRENT_TIMESTAMP(3)` a pour premier mot-clé `CREATE` : ce n'en est pas une.
/// Une écriture par procédure stockée (`CALL`) est un angle mort écrit.
pub fn est_ecriture(statement: &str) -> bool {
    let premier = premier_mot_cle(statement);
    if VERBES_ECRITURE.contains(&premier.as_str()) {
        return true;
    }
    if premier != "WITH" {
        return false;
    }
    let mut profondeur = 0i32;
    for j in jetons(statement).into_iter().skip(1) {
        match j {
            Jeton::Ponct('(') => profondeur += 1,
            Jeton::Ponct(')') => profondeur -= 1,
            Jeton::Mot(m) if profondeur == 0 => {
                let m = m.to_ascii_uppercase();
                if m == "SELECT" {
                    return false;
                }
                if VERBES_ECRITURE.contains(&m.as_str()) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// Mots qui ouvrent une position de table.
const OUVRE_TABLE: &[&str] = &[
    "INSERT", "REPLACE", "UPDATE", "DELETE", "TRUNCATE", "LOAD", "INTO", "FROM", "JOIN", "TABLE",
];
/// Mots qui terminent la liste des tables.
const FIN_TABLES: &[&str] = &[
    "SET", "VALUES", "VALUE", "SELECT", "WHERE", "ORDER", "LIMIT",
];
/// Mots sautés sans changer d'état.
const NEUTRES: &[&str] = &[
    "LOW_PRIORITY",
    "HIGH_PRIORITY",
    "DELAYED",
    "IGNORE",
    "QUICK",
    "DATA",
    "LOCAL",
    "INFILE",
    "CONCURRENT",
    "LEFT",
    "RIGHT",
    "INNER",
    "OUTER",
    "CROSS",
    "NATURAL",
    "STRAIGHT_JOIN",
    "AS",
];

/// Les tables qu'un statement **nomme** : les identifiants qui suivent le
/// verbe, `INTO`, `FROM`, `JOIN`, `TABLE` et les virgules d'une liste
/// multi-tables, jusqu'au premier `SET`, `VALUES`, `SELECT` ou `WHERE` hors
/// parenthèses ; accents graves, qualificatifs et **alias** retirés, en
/// minuscules. Un nom composé par `format!` (`{table}`) est rendu tel quel.
/// Une parenthèse (liste de colonnes, sous-requête) est sautée.
pub fn tables_nommees(statement: &str) -> Vec<String> {
    #[derive(PartialEq)]
    enum Pos {
        Table,
        ApresTable,
        ApresAlias,
        Autre,
    }
    let mut pos = Pos::Autre;
    let mut profondeur = 0i32;
    let mut tables = Vec::new();
    for j in jetons(statement) {
        match j {
            Jeton::Ponct('(') => {
                profondeur += 1;
                if profondeur == 1 && pos != Pos::Autre {
                    pos = Pos::Autre;
                }
            }
            Jeton::Ponct(')') => profondeur -= 1,
            _ if profondeur > 0 => {}
            Jeton::Ponct(',') => {
                if matches!(pos, Pos::ApresTable | Pos::ApresAlias) {
                    pos = Pos::Table;
                }
            }
            Jeton::Mot(m) => {
                let haut = m.to_ascii_uppercase();
                if FIN_TABLES.contains(&haut.as_str()) {
                    break;
                }
                if OUVRE_TABLE.contains(&haut.as_str()) {
                    pos = Pos::Table;
                } else if haut == "ON" || haut == "USING" {
                    pos = Pos::Autre;
                } else if NEUTRES.contains(&haut.as_str()) {
                } else {
                    match pos {
                        Pos::Table => {
                            tables.push(m.to_ascii_lowercase());
                            pos = Pos::ApresTable;
                        }
                        Pos::ApresTable => pos = Pos::ApresAlias,
                        Pos::ApresAlias | Pos::Autre => {}
                    }
                }
            }
            _ => {}
        }
    }
    tables
}

/// Les colonnes affectées par le premier `SET` d'un statement (hors
/// parenthèses), alias retirés et en minuscules ; `None` sans `SET`. Les
/// virgules à l'intérieur de parenthèses (`COALESCE(a, b)`) ne séparent pas
/// deux affectations.
pub fn colonnes_du_set(statement: &str) -> Option<Vec<String>> {
    let jetons = jetons(statement);
    let mut profondeur = 0i32;
    let mut debut = None;
    for (i, j) in jetons.iter().enumerate() {
        match j {
            Jeton::Ponct('(') => profondeur += 1,
            Jeton::Ponct(')') => profondeur -= 1,
            Jeton::Mot(m) if profondeur == 0 && m.eq_ignore_ascii_case("SET") => {
                debut = Some(i + 1);
                break;
            }
            _ => {}
        }
    }
    let debut = debut?;
    let mut colonnes = Vec::new();
    let mut attend_colonne = true;
    profondeur = 0;
    for j in &jetons[debut..] {
        match j {
            Jeton::Ponct('(') => profondeur += 1,
            Jeton::Ponct(')') => profondeur -= 1,
            _ if profondeur > 0 => {}
            Jeton::Ponct(',') => attend_colonne = true,
            Jeton::Mot(m) => {
                let haut = m.to_ascii_uppercase();
                if ["WHERE", "ORDER", "LIMIT"].contains(&haut.as_str()) {
                    break;
                }
                if attend_colonne {
                    colonnes.push(m.to_ascii_lowercase());
                    attend_colonne = false;
                }
            }
            _ => {}
        }
    }
    Some(colonnes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn premier_mot_et_ecriture() {
        assert_eq!(
            premier_mot_cle("  insert ignore into t values (1)"),
            "INSERT"
        );
        for e in [
            "INSERT INTO t VALUES (1)",
            "insert low_priority into t values (1)",
            "REPLACE t (a) VALUES (1)",
            "UPDATE t SET a = 1",
            "DELETE QUICK FROM t",
            "TRUNCATE TABLE t",
            "LOAD DATA INFILE 'x' INTO TABLE t",
            "WITH c AS (SELECT 1) UPDATE t SET a = 1",
            "WITH c AS (SELECT 1), d AS (SELECT 2) DELETE FROM t",
        ] {
            assert!(est_ecriture(e), "{e}");
        }
        for l in [
            "SELECT 1",
            "WITH pieces AS (SELECT 1) SELECT * FROM pieces",
            "CREATE TABLE t (a INT, FOREIGN KEY (a) REFERENCES u(id) ON DELETE RESTRICT)",
            "CREATE TABLE t (d DATETIME ON UPDATE CURRENT_TIMESTAMP(3))",
            "CALL p()",
            "last_insert_id invalide après INSERT companies",
        ] {
            assert!(!est_ecriture(l), "{l}");
        }
    }

    #[test]
    fn tables() {
        let cas: &[(&str, &[&str])] = &[
            ("INSERT INTO prompts_x (a, b) VALUES (?, ?)", &["prompts_x"]),
            ("INSERT prompts_x (a) VALUES (1)", &["prompts_x"]),
            (
                "INSERT LOW_PRIORITY INTO `prompts_x` VALUES (1)",
                &["prompts_x"],
            ),
            ("REPLACE INTO kesh.versions_x SELECT 1", &["versions_x"]),
            (
                "UPDATE prompts_x p JOIN versions_x v ON v.prompt_id = p.id SET v.body = ?",
                &["prompts_x", "versions_x"],
            ),
            ("UPDATE users t SET t.company_id = ?", &["users"]),
            (
                "DELETE v FROM versions_x v WHERE v.id = 1",
                &["v", "versions_x"],
            ),
            ("TRUNCATE TABLE {table}", &["{table}"]),
            ("UPDATE a, b AS bb SET a.x = 1", &["a", "b"]),
            (
                "UPDATE journal_entry_lines jel JOIN journal_entries je ON je.id = jel.entry_id SET jel.k = 1",
                &["journal_entry_lines", "journal_entries"],
            ),
        ];
        for (sql, attendu) in cas {
            assert_eq!(tables_nommees(sql), *attendu, "{sql}");
        }
    }

    #[test]
    fn set() {
        assert_eq!(
            colonnes_du_set(
                "UPDATE cles_x SET revoked_at = NOW(3), version = version + 1 WHERE id = ?"
            ),
            Some(vec!["revoked_at".into(), "version".into()])
        );
        assert_eq!(
            colonnes_du_set("UPDATE users t SET t.company_id = ? WHERE t.company_id = ?"),
            Some(vec!["company_id".into()])
        );
        assert_eq!(
            colonnes_du_set("UPDATE x SET a = COALESCE(b, c), d = 1"),
            Some(vec!["a".into(), "d".into()])
        );
        assert_eq!(colonnes_du_set("INSERT INTO t VALUES (1)"), None);
    }

    #[test]
    fn blocs() {
        assert_eq!(retirer_commentaires_de_bloc("a /* b; */ c"), "a   c");
        assert_eq!(retirer_commentaires_de_bloc("a /* b"), "a  ");
        assert_eq!(retirer_commentaires_de_bloc("abc"), "abc");
    }
}
