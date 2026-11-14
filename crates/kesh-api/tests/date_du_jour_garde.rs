//! Story 35-5a0 (#929, #1022, #907), D5 — **la garde de la date du jour**.
//!
//! Deux familles de contrôles, en **listes closes à égalité stricte par fichier** sur
//! `crates/*/src/**/*.rs` et `crates/*/tests/**/*.rs` (ce fichier exclu nommément) : un fichier
//! absent d'une liste doit compter 0, un compte qui diffère du nombre écrit, dans un sens ou dans
//! l'autre, rougit.
//!
//! 1. **L'inventaire des sites non résolus** : les formes qui tirent la date du jour d'ailleurs que
//!    de l'horloge (`.date_naive(`, `UTC_DATE(`, `CURDATE(`…). L'unité est l'**occurrence**, après
//!    retrait des lignes `//` et réduction des blancs autour des `.` (une forme coupée sur
//!    plusieurs lignes est vue). La liste se ramène au compte réel à chaque conversion, comme un
//!    cliquet : la 35-5a, la 35-5b, la 35-5c l'éditent, dans cet ordre.
//! 2. **Le durcissement de la plomberie de test** : `KESH_TEST_INSTANT`, sa constante,
//!    `instant_de_test`, `horloge_depuis`, `horloge_de_test`, `new_for_tests`, `HorlogeFixe` ne
//!    sont nommés qu'aux endroits prévus — si bien que `main.rs` ne peut ni lire la variable ni
//!    fixer l'horloge —, `HorlogeSysteme` n'est construit qu'à deux endroits, et aucun helper de
//!    jour ne mémorise l'horloge.
//! 3. **L'horloge réelle d'un test** (remédiation de la revue P1 de la 35-5c, B2/E6) : l'heure
//!    réelle passée à `jour_suisse` ou à `HorlogeFixe` (`jour_suisse(Utc::now())`,
//!    `HorlogeFixe(chrono::Utc::now())`) rend un jour juste mais **sourd à `KESH_TEST_INSTANT`**,
//!    forme qu'aucune des deux familles ci-dessus ne voit. Liste close, à égalité stricte, sur le
//!    texte entier sans ses lignes `//` (une forme coupée est vue).
//!
//! **Angle mort assumé** (inventaire) : un jour tiré d'une variable d'instant
//! (`let n = Utc::now().naive_utc(); n.date()`) n'est pas relevé — `.date()` sur tout récepteur
//! rendrait des centaines de lignes légitimes. Ne sont pas relevées non plus, faute d'usage au
//! dépôt le 2026-11-13 (revue P1, B3 : `git grep` sans résultat utile) : `Utc::now().date()`
//! (API dépréciée), `NaiveDate::from(… naive_utc())`, et en SQL `CAST(NOW() AS DATE)`,
//! `SYSDATE()`, `DATE_FORMAT(NOW(), …)`, une comparaison `… < NOW()` ; la première qui
//! apparaîtrait s'ajoute à `FORMES`. **Angle mort assumé** (durcissement) : un autre nom
//! de variable d'environnement, lu en production et converti à la main en instant passé à une
//! horloge écrite pour l'occasion ; la revue de `main.rs` et de `lib.rs` le voit. **Angle mort
//! assumé** (horloge réelle d'un test) : l'instant passé par une variable
//! (`let n = Utc::now(); jour_suisse(n)`), un alias (`use chrono::Utc as U;`,
//! `use kesh_core::horloge::jour_suisse as js;`), une fonction ou une macro enveloppe,
//! `SystemTime::now()` converti, une `impl Horloge` écrite pour l'occasion.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

use regex::Regex;

/// Ce fichier, exclu du balayage : ses motifs et ses sondes contiennent les formes.
const CE_FICHIER: &str = "kesh-api/tests/date_du_jour_garde.rs";

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

/// Les sources balayées : (chemin relatif à `crates/`, texte), ce fichier exclu.
fn sources() -> Vec<(String, String)> {
    let racine = racine();
    let mut chemins = Vec::new();
    for c in std::fs::read_dir(&racine).unwrap() {
        let c = c.unwrap().path();
        if c.is_dir() {
            fichiers_rs(&c.join("src"), &mut chemins);
            fichiers_rs(&c.join("tests"), &mut chemins);
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
        .filter(|(f, _)| f != CE_FICHIER)
        .collect();
    v.sort();
    assert!(
        v.len() >= 500,
        "balayage : {} fichiers lus, plancher 500 — chemin faux ?",
        v.len()
    );
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

/// Une expression compilée une fois pour tout le binaire (les sondes rebalaient les sources).
fn re(s: &str) -> Regex {
    static CACHE: LazyLock<Mutex<HashMap<String, Regex>>> = LazyLock::new(Default::default);
    CACHE
        .lock()
        .unwrap()
        .entry(s.to_string())
        .or_insert_with(|| Regex::new(s).unwrap())
        .clone()
}

// ---------------------------------------------------------------------------
// 1. L'inventaire des sites non résolus
// ---------------------------------------------------------------------------

/// Les formes relevées, en expressions bornées : Rust (casse respectée, tout récepteur), puis
/// SQL (casse ignorée).
const FORMES: &[&str] = &[
    r"\.date_naive\s*\(",
    r"\.naive_utc\s*\(\s*\)\s*\.\s*date\s*\(",
    r"\.naive_local\s*\(",
    r"\bUtc::today\b",
    r"\bLocal::",
    r"\bUtc::now\s*\(\s*\)\s*\.\s*(year|month|day|format)\s*\(",
    r"(?i)\bUTC_DATE\s*\(",
    r"(?i)\bCURDATE\s*\(",
    r"(?i)\bCURRENT_DATE\b",
    r"(?i)\bDATE\s*\(\s*(NOW|UTC_TIMESTAMP|CURRENT_TIMESTAMP)\b",
];

/// Le texte lu par l'inventaire : lignes `//` écartées, puis blancs réduits avant et après les
/// `.` — une chaîne de méthodes coupée sur plusieurs lignes se lit d'un tenant.
fn texte_inventaire(texte: &str) -> String {
    re(r"\s*\.\s*")
        .replace_all(&sans_commentaires(texte), ".")
        .into_owned()
}

fn occurrences_inventaire(texte: &str) -> usize {
    let t = texte_inventaire(texte);
    FORMES.iter().map(|f| re(f).find_iter(&t).count()).sum()
}

/// Les fichiers qui portent encore une forme, avec leur nombre d'occurrences.
fn inventaire(sources: &[(String, String)]) -> BTreeMap<String, usize> {
    sources
        .iter()
        .map(|(f, t)| (f.clone(), occurrences_inventaire(t)))
        .filter(|(_, n)| *n > 0)
        .collect()
}

/// `(fichier, nombre, story, motif)` — la liste close de l'inventaire.
type Exemption = (&'static str, usize, &'static str, &'static str);

/// La liste close, comptée **depuis l'exécution** de la garde (Story 35-5a0).
/// Entrées permanentes : `kesh-core/src/horloge.rs` (le `.date_naive()` du jour) et, dans
/// `admin.rs`, l'horodatage du nom de sauvegarde (un instant, non un jour). Le reste était la
/// dette que la 35-5a, la 35-5b et la 35-5c ont soldée (verdicts de la fiche 35-5a) : depuis
/// la 35-5c (D2), la liste ne porte plus que ces deux entrées, et tout autre fichier de `src`
/// et de `tests` compte 0.
const EXEMPTIONS: &[Exemption] = &[
    // --- Production (`src`)
    (
        "kesh-core/src/horloge.rs",
        1,
        "permanent",
        "le jour de l'instant, seul site admis",
    ),
    (
        "kesh-api/src/routes/admin.rs",
        // 2 → 1 : Story 35-5b, le nom daté de la sauvegarde lit l'horloge (site 9).
        1,
        "permanent",
        "horodatage `%Y%m%dT%H%M%S%3f` du nom de sauvegarde, un instant (#669)",
    ),
    // Story 35-5b (D5) : plus aucune entrée de code de production hors les deux permanentes.
    // Sortis de la liste, leur date venant de l'horloge de l'état ou de la `Lecture`, ou d'un
    // paramètre — `routes/audit_log.rs`, `bank_accounts.rs`, `exports.rs`,
    // `invoice_email.rs`, `invoices.rs` (sites 10 à 16), `services/postes_ouverts.rs`,
    // `rapports.rs` (sites 18 à 21) ; `kesh-db` `dunning_eligibility.rs` (site 22) et
    // `invoices.rs` (sites 23-25 ; son module de tests en date fixe, D9) ; `kesh-seed`
    // `lib.rs` (site 26) et `pieges.rs` (sites 27-28).
    // Story 35-5c (D1, D2) : sortis de la liste — le module de tests de `kesh-db`
    // `journal_entries.rs` (50 : `setup()` lit le jour suisse une fois, ses tests et
    // `lettrer_dans` le réemploient) ; `kesh-api` `accounts_e2e.rs` (2),
    // `admin_full_import_e2e.rs` (3), `credit_card_e2e.rs` (3), `invoice_delete_e2e.rs` (2),
    // `invoice_unvalidate_e2e.rs` (6), `mcp_enveloppe_e2e.rs` (1),
    // `reconciliation_link_e2e.rs` (1), leurs montages lisant le jour de l'horloge de test.
    // Remédiation de la revue P3 de la 35-5a (exécution sous instant décalé) :
    // `journal_entry_reversal_e2e.rs` et `period_lock_e2e.rs` sortis de la liste.
    // Story 35-5b : sortis de la liste, leurs jours lisant l'horloge de test —
    // `bank_accounts_closed_e2e.rs` (`closedOn`), `dunning_reminders_e2e.rs`,
    // `fiscal_years_e2e.rs` (`seed_demo`), `homepage_summary_e2e.rs`, `mcp_outils_e2e.rs`,
    // `onboarding_audit_e2e.rs`, `onboarding_e2e.rs`, `open_items_e2e.rs`,
    // `services_motif.rs`, et `kesh-db` `dunning_eligibility.rs`.
    // --- Tests de `kesh-db`
    // Story 35-5a (AC9) : sortis de la liste, leurs jours lisant l'horloge de test —
    // `bank_account_opening_balance_e2e.rs`, `dates_bornes_e2e.rs`, `filet_bilan_clos_e2e.rs`,
    // `letterings_e2e.rs`, `opening_balances_e2e.rs`, `reconciliation_rules_e2e.rs`,
    // `rejeu_interblocage_e2e.rs`, `fiscal_years_repository.rs` ; `fiscal_years_e2e.rs` 2 → 1.
    // Remédiation de la revue P3 (D4) : `supplier_invoices_draft.rs` sorti de la liste,
    // son `aujourdhui()` lisant le jour de l'horloge de test.
    // Remédiation de la revue P2 (D4, inventaire parti du symptôme) : sortis de la liste,
    // leurs montages lisant le jour de l'horloge de test — `reconciliation_e2e.rs`,
    // `filet_bilan_clos.rs`, `invoice_settlement.rs`, `letterings.rs`, `open_items.rs`,
    // `reconciliation_cancel.rs`, `supplier_invoices_repository.rs`,
    // `support/lettering_documents.rs` (`jours_avant`), `kesh-report` `open_items_invariant.rs`.
];

fn ecarts(trouve: &BTreeMap<String, usize>, attendu: &[Exemption]) -> Vec<String> {
    let mut attendu_map = BTreeMap::new();
    for (f, n, _, _) in attendu {
        assert!(
            attendu_map.insert(f.to_string(), *n).is_none(),
            "{f} : deux entrées"
        );
    }
    let mut v = Vec::new();
    for (f, n) in trouve {
        let a = attendu_map.get(f).copied().unwrap_or(0);
        if a != *n {
            v.push(format!("{f} : {n} occurrence(s), {a} écrite(s)"));
        }
    }
    for (f, a) in &attendu_map {
        if !trouve.contains_key(f) {
            v.push(format!("{f} : 0 occurrence, {a} écrite(s)"));
        }
    }
    v
}

#[test]
fn inventaire_des_dates_du_jour() {
    let e = ecarts(&inventaire(&sources()), EXEMPTIONS);
    assert!(
        e.is_empty(),
        "inventaire de la date du jour (liste close, égalité stricte) :\n{}",
        e.join("\n")
    );
}

// ---------------------------------------------------------------------------
// 2. Le durcissement de la plomberie de test
// ---------------------------------------------------------------------------

/// Où une garde de durcissement balaie.
#[derive(Clone, Copy)]
enum Portee {
    /// `crates/*/src` et `crates/*/tests`.
    SrcEtTests,
    /// `crates/*/src` seulement.
    Src,
}

fn dans(portee: Portee, f: &str) -> bool {
    match portee {
        Portee::SrcEtTests => true,
        Portee::Src => f.split('/').nth(1) == Some("src"),
    }
}

/// Une garde de durcissement : un motif compté, `use` compris, sur le texte sans commentaires.
struct Durcissement {
    nom: &'static str,
    motif: &'static str,
    portee: Portee,
    /// Retirer les instructions `use` avant de compter (le seul compte de `HorlogeSysteme`).
    sans_use: bool,
    /// Lignes qui ne comptent pas (définition et en-tête d'`impl`), après retrait des `use`.
    lignes_ecartees: &'static [&'static str],
    attendu: &'static [(&'static str, usize)],
}

/// Retire les instructions `use`, `pub use`, `pub(…) use` : de leur début de ligne au premier `;`.
fn sans_instructions_use(texte: &str) -> String {
    let debut = re(r"^\s*(pub(\s*\([^)]*\))?\s+)?use\s");
    let mut v = Vec::new();
    let mut dans_use = false;
    for l in texte.lines() {
        if !dans_use && debut.is_match(l) {
            dans_use = true;
        }
        if dans_use {
            if l.contains(';') {
                dans_use = false;
            }
            continue;
        }
        v.push(l);
    }
    v.join("\n")
}

fn compte_durcissement(d: &Durcissement, texte: &str) -> usize {
    let mut t = sans_commentaires(texte);
    if d.sans_use {
        t = sans_instructions_use(&t);
    }
    let ecartees: Vec<Regex> = d.lignes_ecartees.iter().map(|m| re(m)).collect();
    let motif = re(d.motif);
    t.lines()
        .filter(|l| !ecartees.iter().any(|e| e.is_match(l)))
        .map(|l| motif.find_iter(l).count())
        .sum()
}

fn ecarts_durcissement(d: &Durcissement, sources: &[(String, String)]) -> Vec<String> {
    let trouve: BTreeMap<String, usize> = sources
        .iter()
        .filter(|(f, _)| dans(d.portee, f))
        .map(|(f, t)| (f.clone(), compte_durcissement(d, t)))
        .filter(|(_, n)| *n > 0)
        .collect();
    let attendu: Vec<Exemption> = d.attendu.iter().map(|(f, n)| (*f, *n, "", "")).collect();
    ecarts(&trouve, &attendu)
        .into_iter()
        .map(|e| format!("[{}] {e}", d.nom))
        .collect()
}

const DURCISSEMENTS: &[Durcissement] = &[
    Durcissement {
        nom: "littéral KESH_TEST_INSTANT",
        motif: r#""KESH_TEST_INSTANT""#,
        portee: Portee::SrcEtTests,
        sans_use: false,
        lignes_ecartees: &[],
        // La constante, et l'attribut `should_panic` de son test (qui exige un littéral).
        attendu: &[("kesh-core/src/horloge.rs", 2)],
    },
    Durcissement {
        nom: "NOM_VARIABLE_INSTANT",
        motif: r"\bNOM_VARIABLE_INSTANT\b",
        portee: Portee::SrcEtTests,
        sans_use: false,
        lignes_ecartees: &[],
        // 0 ailleurs, `main.rs` et `lib.rs` compris. `configuration_transmise.rs` : la
        // lecture admise de `horloge_de_test` (forme exacte) et la liste des variables de
        // tests, lue sur la constante (C-e355a0-7).
        attendu: &[
            ("kesh-core/src/horloge.rs", 3),
            ("kesh-api/src/horloge.rs", 2),
            ("kesh-api/tests/configuration_transmise.rs", 2),
            ("kesh-db/tests/support/jour.rs", 2),
            ("kesh-report/tests/support/jour.rs", 2),
        ],
    },
    Durcissement {
        nom: "instant_de_test",
        motif: r"\binstant_de_test\b",
        portee: Portee::SrcEtTests,
        sans_use: false,
        lignes_ecartees: &[],
        // `kesh-core` : 6 → 9 à la remédiation P1 (E2, E5 : blanc, décalage).
        attendu: &[
            ("kesh-core/src/horloge.rs", 9),
            ("kesh-api/src/horloge.rs", 2),
            ("kesh-db/tests/support/jour.rs", 2),
            ("kesh-report/tests/support/jour.rs", 2),
        ],
    },
    Durcissement {
        nom: "horloge_depuis",
        motif: r"\bhorloge_depuis\b",
        portee: Portee::Src,
        sans_use: false,
        lignes_ecartees: &[],
        // Définition, appel par `horloge_de_test`, deux tests du module.
        attendu: &[("kesh-api/src/horloge.rs", 4)],
    },
    Durcissement {
        nom: "new_for_tests",
        motif: r"\bnew_for_tests\b",
        portee: Portee::Src,
        sans_use: false,
        lignes_ecartees: &[],
        // Sa définition seule : aucun appel de production, ni de `mod tests` de `src`.
        attendu: &[("kesh-api/src/lib.rs", 1)],
    },
    Durcissement {
        nom: "horloge_de_test",
        motif: r"\bhorloge_de_test\b",
        portee: Portee::Src,
        sans_use: false,
        lignes_ecartees: &[],
        // Sa définition ; son appel par `new_for_tests`.
        attendu: &[("kesh-api/src/horloge.rs", 1), ("kesh-api/src/lib.rs", 1)],
    },
    Durcissement {
        nom: "HorlogeSysteme construit",
        motif: r"\bHorlogeSysteme\b",
        portee: Portee::SrcEtTests,
        sans_use: true,
        lignes_ecartees: &[
            r"^\s*pub\s+struct\s+HorlogeSysteme\b",
            r"^\s*impl\b.*\bfor\s+HorlogeSysteme\b",
        ],
        // L'horloge de production (`main.rs`) ; la branche réelle de `horloge_depuis`.
        attendu: &[("kesh-api/src/main.rs", 1), ("kesh-api/src/horloge.rs", 1)],
    },
    Durcissement {
        nom: "HorlogeFixe",
        motif: r"\bHorlogeFixe\b",
        portee: Portee::Src,
        sans_use: false,
        lignes_ecartees: &[],
        // Définition et tests ; import et branche fixe de `horloge_depuis` ; `test_state()`
        // d'un `mod tests`. 0 dans `main.rs`, `lib.rs` et tout autre fichier de `src` :
        // une `HorlogeFixe(Utc::now())` de production figerait le jour au démarrage.
        // `kesh-core` : 5 → 6 à la remédiation P1 (E5).
        attendu: &[
            ("kesh-core/src/horloge.rs", 6),
            ("kesh-api/src/horloge.rs", 2),
            ("kesh-api/src/middleware/auth.rs", 1),
        ],
    },
];

/// Les fichiers des helpers de jour, où aucune mémoire n'est admise.
const SANS_MEMOIRE: &[&str] = &[
    "kesh-core/src/horloge.rs",
    "kesh-api/src/horloge.rs",
    "kesh-api/tests/common/jour.rs",
    "kesh-db/tests/support/jour.rs",
    "kesh-report/tests/support/jour.rs",
];

const MEMOIRES: &str = r"\bstatic\b|\bOnceLock\b|\bLazyLock\b|\blazy_static!|\bthread_local!";

fn ecarts_memoire(sources: &[(String, String)]) -> Vec<String> {
    let motif = re(MEMOIRES);
    let mut v = Vec::new();
    for f in SANS_MEMOIRE {
        match sources.iter().find(|(g, _)| g == f) {
            None => v.push(format!("[mémoire] {f} introuvable")),
            Some((_, t)) => {
                let n = motif.find_iter(&sans_commentaires(t)).count();
                if n > 0 {
                    v.push(format!("[mémoire] {f} : {n} mémoire(s)"));
                }
            }
        }
    }
    v
}

// ---------------------------------------------------------------------------
// 3. L'horloge réelle d'un test
// ---------------------------------------------------------------------------

/// L'heure réelle (`Utc::now()`, chemin qualifié ou non) passée **directement** à `jour_suisse`
/// ou à `HorlogeFixe`. Appliqué au texte entier sans ses lignes `//` : `\s` couvre les retours à
/// la ligne, une forme coupée est vue. Portée : `crates/*/src` (production et `mod tests`) et
/// `crates/*/tests` — en production, la forme contournerait aussi l'horloge de l'état.
const HORLOGE_REELLE: &str = r"\b(jour_suisse|HorlogeFixe)\s*\(\s*(::\s*)?([A-Za-z_][A-Za-z0-9_]*\s*::\s*)*Utc\s*::\s*now\s*\(";

/// La liste close des lectures admises, chacune justifiée par la ligne lue (revue P1 de la
/// 35-5c, B2/E6 ; C-e355c-8).
const HORLOGE_REELLE_ADMISES: &[Exemption] = &[
    (
        "kesh-db/src/repositories/journal_entries.rs",
        1,
        "35-5c (D1, C-e355c-1)",
        "`setup()` du `mod tests` : `let today = kesh_core::horloge::jour_suisse(chrono::Utc::now());` \
         — le binaire `lib` de `kesh-db` n'a pas d'horloge de test ; le jour est lu une fois et \
         réemployé par les 50 appelants",
    ),
    (
        "kesh-api/src/middleware/auth.rs",
        1,
        "35-5a0",
        "`test_state()` du `mod tests` : `horloge: …HorlogeFixe(chrono::Utc::now())` — les tests \
         du middleware d'authentification ne datent rien au jour ; un `mod tests` de `src` ne lit \
         pas la variable d'instant (commentaire de la ligne précédente)",
    ),
];

fn ecarts_horloge_reelle(sources: &[(String, String)]) -> Vec<String> {
    let motif = re(HORLOGE_REELLE);
    let trouve: BTreeMap<String, usize> = sources
        .iter()
        .map(|(f, t)| (f.clone(), motif.find_iter(&sans_commentaires(t)).count()))
        .filter(|(_, n)| *n > 0)
        .collect();
    ecarts(&trouve, HORLOGE_REELLE_ADMISES)
        .into_iter()
        .map(|e| format!("[horloge réelle d'un test] {e}"))
        .collect()
}

#[test]
fn horloge_reelle_d_un_test() {
    let e = ecarts_horloge_reelle(&sources());
    assert!(
        e.is_empty(),
        "horloge réelle passée à jour_suisse / HorlogeFixe (liste close, égalité stricte) — \
         un test de `kesh-api` prend `common::jour::aujourd_hui_test()` :\n{}",
        e.join("\n")
    );
}

fn tous_les_ecarts(sources: &[(String, String)]) -> Vec<String> {
    let mut v = ecarts(&inventaire(sources), EXEMPTIONS);
    for d in DURCISSEMENTS {
        v.extend(ecarts_durcissement(d, sources));
    }
    v.extend(ecarts_memoire(sources));
    v.extend(ecarts_horloge_reelle(sources));
    v
}

#[test]
fn durcissement_de_la_plomberie_de_test() {
    let s = sources();
    let mut e = Vec::new();
    for d in DURCISSEMENTS {
        e.extend(ecarts_durcissement(d, &s));
    }
    e.extend(ecarts_memoire(&s));
    assert!(e.is_empty(), "durcissement :\n{}", e.join("\n"));
}

// ---------------------------------------------------------------------------
// Sondes (AC5) — sur les sources réelles, un fichier modifié à la fois
// ---------------------------------------------------------------------------

/// Les sources réelles, `ajout` ajouté en fin du fichier `f` (créé s'il manque).
fn avec(f: &str, ajout: &str) -> Vec<(String, String)> {
    let mut s = sources();
    match s.iter_mut().find(|(g, _)| g == f) {
        Some((_, t)) => {
            t.push('\n');
            t.push_str(ajout);
            t.push('\n');
        }
        None => s.push((f.to_string(), ajout.to_string())),
    }
    s
}

/// La sonde rougit, et par la garde attendue (`marque` : un fragment de son message).
fn rouge(f: &str, ajout: &str, marque: &str) {
    let e = tous_les_ecarts(&avec(f, ajout));
    assert!(
        e.iter().any(|l| l.contains(marque) && l.contains(f)),
        "sonde muette — {f} + {ajout:?} devait rougir par « {marque} » : {e:?}"
    );
}

fn vert(f: &str, ajout: &str) {
    let e = tous_les_ecarts(&avec(f, ajout));
    assert!(e.is_empty(), "sonde rouge — {f} + {ajout:?} : {e:?}");
}

#[test]
fn sonde_les_sources_reelles_sont_vertes_et_ce_fichier_seul_est_exclu() {
    let s = sources();
    assert!(tous_les_ecarts(&s).is_empty());
    assert!(s.iter().all(|(f, _)| f != CE_FICHIER));
    assert!(
        s.iter().any(|(f, _)| f == "kesh-api/tests/horloge_etat.rs"),
        "un autre fichier du même dossier est lu"
    );
}

#[test]
fn sonde_une_forme_ajoutee_rougit_entiere_ou_coupee() {
    rouge(
        "kesh-api/src/lib.rs",
        "fn f() { let _ = chrono::Utc::now().date_naive(); }",
        "kesh-api/src/lib.rs : 1",
    );
    rouge(
        "kesh-api/src/lib.rs",
        "fn f() {\n    let _ = chrono::Utc::now()\n        .date_naive();\n}",
        "kesh-api/src/lib.rs : 1",
    );
    rouge(
        "kesh-api/tests/horloge_etat.rs",
        "const Q: &str = \"SELECT CURDATE()\";",
        "horloge_etat.rs : 1",
    );
    // Un compte + 1 dans un fichier déjà exempté rougit aussi (Story 35-5c : l'entrée de
    // `journal_entries.rs` sortie de la liste, la sonde vise une entrée permanente).
    rouge(
        "kesh-api/src/routes/admin.rs",
        "fn f() { let _ = chrono::Utc::now().date_naive(); }",
        "2 occurrence(s), 1 écrite(s)",
    );
    // Story 35-5c (AC4) — dans un test : un module de tests de `src` sorti de la liste, un
    // fichier de `tests` sorti de la liste, entière ou coupée sur deux lignes.
    rouge(
        "kesh-db/src/repositories/journal_entries.rs",
        "fn f() { let _ = chrono::Utc::now().date_naive(); }",
        "journal_entries.rs : 1 occurrence(s), 0 écrite(s)",
    );
    rouge(
        "kesh-api/tests/accounts_e2e.rs",
        "fn f() { let _ = Utc::now().date_naive(); }",
        "accounts_e2e.rs : 1 occurrence(s), 0 écrite(s)",
    );
    rouge(
        "kesh-api/tests/invoice_unvalidate_e2e.rs",
        "fn f() {\n    let _ = chrono::Utc::now()\n        .naive_utc()\n        .date();\n}",
        "invoice_unvalidate_e2e.rs : 1 occurrence(s), 0 écrite(s)",
    );
    // Une ligne `//` est écartée.
    vert("kesh-api/src/lib.rs", "// chrono::Utc::now().date_naive()");
}

#[test]
fn sonde_une_exemption_portee_a_plus_un_rougit() {
    let mut liste: Vec<Exemption> = EXEMPTIONS.to_vec();
    let i = liste
        .iter()
        .position(|(f, ..)| *f == "kesh-core/src/horloge.rs")
        .unwrap();
    liste[i].1 += 1;
    let e = ecarts(&inventaire(&sources()), &liste);
    assert_eq!(
        e,
        vec!["kesh-core/src/horloge.rs : 1 occurrence(s), 2 écrite(s)".to_string()]
    );
}

#[test]
fn sonde_les_gardes_de_durcissement_rougissent() {
    rouge(
        "kesh-api/src/lib.rs",
        "const V: &str = \"KESH_TEST_INSTANT\";",
        "[littéral KESH_TEST_INSTANT]",
    );
    rouge(
        "kesh-api/src/routes/admin.rs",
        "fn f() -> kesh_api::horloge::HorlogeSysteme { kesh_api::horloge::HorlogeSysteme }",
        "[HorlogeSysteme construit]",
    );
    rouge(
        "kesh-db/tests/support/jour.rs",
        "static H: std::sync::OnceLock<u8> = std::sync::OnceLock::new();",
        "[mémoire]",
    );
    rouge(
        "kesh-api/src/routes/admin.rs",
        "fn f(p: P, c: C, r: R, i: I) { let _ = crate::AppState::new_for_tests(p, c, r, i); }",
        "[new_for_tests]",
    );
    rouge(
        "kesh-api/src/routes/admin.rs",
        "fn f() { let _ = crate::AppState::new_for_tests; }",
        "[new_for_tests]",
    );
}

#[test]
fn sonde_l_horloge_reelle_d_un_test_rougit() {
    let m = "[horloge réelle d'un test]";
    // Dans un fichier de `tests`, qualifiée, puis coupée sur plusieurs lignes.
    rouge(
        "kesh-api/tests/accounts_e2e.rs",
        "fn f() { let _ = kesh_core::horloge::jour_suisse(chrono::Utc::now()); }",
        m,
    );
    rouge(
        "kesh-api/tests/horloge_etat.rs",
        "fn f() {\n    let _ = jour_suisse(\n        Utc::now(),\n    );\n}",
        m,
    );
    // `HorlogeFixe(Utc::now())` dans `tests/`, que la garde `HorlogeFixe` (portée `src`) ignore.
    rouge(
        "kesh-api/tests/credit_card_e2e.rs",
        "fn f() { let _ = HorlogeFixe(chrono::Utc::now()); }",
        m,
    );
    // Une seconde lecture dans le module admis rougit (le compte, non le fichier, est clos).
    rouge(
        "kesh-db/src/repositories/journal_entries.rs",
        "fn f() { let _ = kesh_core::horloge::jour_suisse(chrono::Utc::now()); }",
        "journal_entries.rs : 2 occurrence(s), 1 écrite(s)",
    );
    // Un instant donné, un `Utc::now()` qui ne va pas au jour, un commentaire : vert.
    vert(
        "kesh-api/tests/horloge_etat.rs",
        "fn f(i: chrono::DateTime<chrono::Utc>) { let _ = jour_suisse(i); let _ = chrono::Utc::now().timestamp(); }",
    );
    vert(
        "kesh-api/tests/horloge_etat.rs",
        "// jour_suisse(chrono::Utc::now())",
    );
}

#[test]
fn sonde_main_rs_ne_peut_ni_lire_la_variable_ni_fixer_l_horloge() {
    let m = "kesh-api/src/main.rs";
    rouge(
        m,
        "fn f() { let _ = std::env::var(NOM_VARIABLE_INSTANT); }",
        "[NOM_VARIABLE_INSTANT]",
    );
    rouge(
        m,
        "fn f() { let _ = instant_de_test(None); }",
        "[instant_de_test]",
    );
    rouge(
        m,
        "fn f() { let _ = horloge_depuis(None); }",
        "[horloge_depuis]",
    );
    rouge(
        m,
        "fn f() { let _ = HorlogeFixe(chrono::Utc::now()); }",
        "[HorlogeFixe]",
    );
    // L'import compte, même renommé.
    rouge(
        m,
        "use kesh_api::horloge::horloge_de_test as h;",
        "[horloge_de_test]",
    );
    // Un commentaire ne compte pas.
    vert(m, "// NOM_VARIABLE_INSTANT");
    // Un import groupé de `HorlogeSysteme` n'est pas une construction.
    vert(m, "use kesh_api::{\n    horloge::HorlogeSysteme,\n};");
}
