//! Story 48-11 (#1245, #1430), D6 — **la garde de l'arrondi au centime**.
//!
//! `format!("{:.2}", d)` sur un `Decimal` **coupe** les chiffres au lieu de les arrondir
//! (`10.0050` → `10.00`, `-0.0040` → `-0.00`). Cette garde tient deux listes closes, à égalité
//! stricte **par fichier**, sur `crates/*/src/**/*.rs` (lignes `//` retirées ; ce fichier n'est pas
//! sous `src`, il n'est donc pas balayé) : un fichier absent d'une liste doit compter 0, un compte
//! qui diffère du nombre écrit, dans un sens ou dans l'autre, rougit et nomme le fichier.
//!
//! 1. **Précision de format** : toute précision fixe (`{x:.2}`, `{:.2}`, `{x:>10.2}`), nommée
//!    (`{x:.scale$}`), passée en argument (`{:.*}`) ou positionnelle (`{:.1$}`). Un montant de rapport passe par
//!    `kesh_core::types::format_two_decimals`, qui arrondit **avant** d'écrire ; l'export global
//!    écrit chaque colonne à l'échelle du schéma.
//! 2. **Troncature explicite** : `.trunc()`, `trunc_with_scale(`, `RoundingStrategy::ToZero` et
//!    `RoundDown`.
//!
//! **La garde n'est pas vraie par construction** : la fonction de comptage est d'abord appliquée à
//! un texte écrit ici (sonde positive), les comptes attendus sont écrits à la main, et le
//! balayage doit voir plus de 100 fichiers dont `money.rs`.
//!
//! **Angles morts assumés** :
//! - un `Decimal` écrit par `{}` ou `to_string()` sort à son échelle (quatre décimales) sans rien
//!   couper : c'est le symptôme de #813, non celui-ci ;
//! - une précision construite à l'exécution hors d'un littéral (aucun site) ;
//! - un découpage maison au point décimal (aucun site sur un montant) ;
//! - le frontend (big.js arrondit) ;
//! - l'arrondi au pair `round_dp(` : un autre mode, non une troncature (#1441, #1442, #1443) ;
//! - les **stratégies d'arrondi que la famille 2 ne nomme pas** — `MidpointTowardZero` et
//!   `RoundHalfDown` (le demi vers zéro : `10.0050` → `10.00`, le symptôme même de #1245),
//!   `ToNegativeInfinity`, `.floor()`, `.to_i64()` après une multiplication par 100 : aucun site
//!   aujourd'hui, une forme future n'est pas vue ; la famille 2 n'est pas élargie pour ne pas
//!   confondre une troncature et un arrondi d'un autre mode, que la relecture juge ;
//! - la **liste close compte par fichier** : un échange à compte égal dans un même fichier (un
//!   `{:.3}` de `f32` retiré de `kesh-qrbill/src/pdf.rs`, un `{:.2}` de `Decimal` ajouté au même
//!   fichier) laisse la garde verte — la relecture du diff de tout fichier listé le voit, la garde
//!   non.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

use regex::Regex;

fn racine() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn fichiers_rs(dir: &Path, v: &mut Vec<PathBuf>) {
    let Ok(entrees) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entrees {
        let p = e.unwrap().path();
        if p.is_dir() {
            fichiers_rs(&p, v);
        } else if p.extension().is_some_and(|x| x == "rs") {
            v.push(p);
        }
    }
}

/// Les sources balayées : (chemin relatif à `crates/`, texte), `crates/*/src` seulement.
fn sources() -> Vec<(String, String)> {
    let racine = racine();
    let mut chemins = Vec::new();
    for c in std::fs::read_dir(&racine).unwrap() {
        let c = c.unwrap().path();
        if c.is_dir() {
            fichiers_rs(&c.join("src"), &mut chemins);
        }
    }
    let mut v: Vec<(String, String)> = chemins
        .into_iter()
        .map(|p| {
            let rel = p
                .strip_prefix(&racine)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            (rel, std::fs::read_to_string(&p).unwrap())
        })
        .collect();
    v.sort();
    v
}

fn est_commentaire(ligne: &str) -> bool {
    ligne.trim_start().starts_with("//")
}

/// Le texte sans ses lignes `//` (`///`, `//!` compris).
fn sans_commentaires(texte: &str) -> String {
    texte
        .lines()
        .filter(|l| !est_commentaire(l))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Une expression compilée une fois pour tout le binaire.
fn re(s: &str) -> Regex {
    static CACHE: LazyLock<Mutex<HashMap<String, Regex>>> = LazyLock::new(Default::default);
    CACHE
        .lock()
        .unwrap()
        .entry(s.to_string())
        .or_insert_with(|| Regex::new(s).unwrap())
        .clone()
}

/// Famille 1 : précision fixe, nommée (`x$`) ou passée en argument (`*`), avec ou sans largeur
/// ni alignement.
const PRECISION: &str = r#"\{[A-Za-z_0-9.]*:[^{}"]*\.([0-9]+\$?|[A-Za-z_][A-Za-z_0-9]*\$|\*)\}"#;

/// Famille 2 : troncature explicite.
const TRONCATURE: &str = r"\.trunc\(\)|trunc_with_scale\(|RoundingStrategy::(ToZero|RoundDown)";

fn compte(motif: &str, texte: &str) -> usize {
    re(motif).find_iter(&sans_commentaires(texte)).count()
}

/// Les fichiers qui portent le motif, avec leur nombre d'occurrences.
fn par_fichier(motif: &str, sources: &[(String, String)]) -> BTreeMap<String, usize> {
    sources
        .iter()
        .map(|(f, t)| (f.clone(), compte(motif, t)))
        .filter(|(_, n)| *n > 0)
        .collect()
}

/// Compare le relevé à la liste close, à égalité stricte, et nomme chaque fichier fautif.
fn assert_liste_close(famille: &str, releve: &BTreeMap<String, usize>, attendu: &[(&str, usize)]) {
    let attendu: BTreeMap<String, usize> =
        attendu.iter().map(|(f, n)| (f.to_string(), *n)).collect();
    let mut ecarts = Vec::new();
    for f in releve.keys().chain(attendu.keys()) {
        let lu = releve.get(f).copied().unwrap_or(0);
        let ecrit = attendu.get(f).copied().unwrap_or(0);
        if lu != ecrit {
            let ligne = format!("{f} : {lu} au lieu de {ecrit}");
            if !ecarts.contains(&ligne) {
                ecarts.push(ligne);
            }
        }
    }
    assert!(
        ecarts.is_empty(),
        "garde de l'arrondi au centime, {famille} — écarts à la liste close :\n  {}",
        ecarts.join("\n  ")
    );
}

/// Famille 1, la liste close **après** la story 48-11 : 22 occurrences dans 8 fichiers, une
/// justification par fichier. Nombres écrits à la main.
const PRECISIONS_ADMISES: &[(&str, usize)] = &[
    // La fonction commune : `{:.2}` sur la valeur **déjà arrondie** (`round_for_display`), la
    // précision ne fait que compléter par des zéros.
    ("kesh-core/src/types/money.rs", 1),
    // `fmt_decimal_scale` : `{d:.scale$}`, précision **égale** à l'échelle de la colonne
    // (constantes `DEC_*`) — complète, ne coupe pas ; prouvé par le test de pleine échelle
    // `export_global_chaque_colonne_decimal_a_l_echelle_du_schema`.
    ("kesh-api/src/exports/csv_tables.rs", 1),
    // Messages de test (`{kurs:.20}`), pas un montant.
    ("kesh-core/src/exchange_rates/parse/tests.rs", 2),
    // `format_money` arrondit `MidpointAwayFromZero` juste avant d'écrire.
    ("kesh-i18n/src/formatting.rs", 1),
    // `amt` : `round_dp(2)` avant d'écrire (arrondi au pair, #1441, hors de cette story).
    ("kesh-payment/src/pain001/mod.rs", 1),
    // `format_amount_payload` arrondit `MidpointAwayFromZero` juste avant d'écrire (factures
    // émises, non touché).
    ("kesh-qrbill/src/generator.rs", 1),
    // Treize `f32` de géométrie (`{:.3}mm`, `({:.2}, {:.2})`), dont le module de tests : pas un
    // `Decimal`.
    ("kesh-qrbill/src/pdf.rs", 13),
    // `format_exchange_rate` (`.prec$`, précision jamais inférieure à l'échelle de la valeur :
    // elle complète) et un `f32` de géométrie dans `mod tests`.
    ("kesh-report/src/pdf.rs", 2),
];

/// Famille 2, la liste close : ni l'un ni l'autre n'affiche un montant.
const TRONCATURES_ADMISES: &[(&str, usize)] = &[
    // `n.abs().trunc() >= limit` : borne de la partie entière d'une saisie.
    ("kesh-core/src/bank_imports/manual.rs", 1),
    // `reseau.trunc()` : une adresse IP.
    ("kesh-api/src/routes/mcp_server_settings.rs", 1),
];

/// Le balayage voit des fichiers : un chemin faux rendrait la garde verte à vide.
#[test]
fn le_balayage_voit_les_sources() {
    let s = sources();
    assert!(
        s.len() > 100,
        "balayage : {} fichiers lus, plancher 100 — chemin faux ?",
        s.len()
    );
    assert!(
        s.iter().any(|(f, _)| f == "kesh-core/src/types/money.rs"),
        "balayage : `kesh-core/src/types/money.rs` doit figurer parmi les fichiers lus"
    );
}

/// Sonde positive de la famille 1 : la fonction de comptage voit les sept formes, et seulement
/// elles (la ligne `//` est écartée).
#[test]
fn sonde_positive_famille_1() {
    let sonde = [
        r#"let a = format!("{d:.2}");"#,
        r#"let b = format!("{:.2}", d);"#,
        r#"let c = format!("{d:>10.2}");"#,
        r#"let e = format!("{d:.scale$}");"#,
        r#"let f = format!("{:.*}", 2, d);"#,
        r#"let g = format!("{:.1$}", d, 2);"#,
        "write!(f,\n    \"{:.2}\",\n    d)?;",
        r#"// {:.2} dans un commentaire"#,
    ]
    .join("\n");
    assert_eq!(compte(PRECISION, &sonde), 7);
}

/// Sonde positive de la famille 2.
#[test]
fn sonde_positive_famille_2() {
    let sonde = "let a = d.trunc();\nlet b = RoundingStrategy::ToZero;\n// d.trunc()";
    assert_eq!(compte(TRONCATURE, sonde), 2);
}

/// AC9 — famille 1 : 22 occurrences dans 8 fichiers, tout autre fichier compte 0.
#[test]
fn famille_1_precision_de_format() {
    let releve = par_fichier(PRECISION, &sources());
    assert_liste_close(
        "famille 1 (précision de format)",
        &releve,
        PRECISIONS_ADMISES,
    );
    // Le total écrit à la main, pour que la liste ne soit pas sa propre source.
    assert_eq!(releve.values().sum::<usize>(), 22);
    assert_eq!(releve.len(), 8);
}

/// AC9 — famille 2 : deux troncatures, aucune sur un montant.
#[test]
fn famille_2_troncature_explicite() {
    let releve = par_fichier(TRONCATURE, &sources());
    assert_liste_close("famille 2 (troncature)", &releve, TRONCATURES_ADMISES);
    assert_eq!(releve.values().sum::<usize>(), 2);
}
