//! Gardes de texte — Story 15-14a (C-15-14-8) : la documentation et les replis
//! disent ce que Kesh fait réellement.
//!
//! Ces tests ne démarrent ni serveur ni base : ils lisent des fichiers du dépôt
//! (`env!("CARGO_MANIFEST_DIR")` + `../../`). ⚠️ **Chaque assertion négative est
//! adossée à une assertion positive sur le même fichier** — le fichier a bien été
//! lu et contient la section attendue —, sans quoi un chemin faux ou une lecture à
//! vide rendrait un vert muet.
//!
//! | Garde | Issue | Objet |
//! |---|---|---|
//! | G2 | #547 | aucun renvoi au menu « Réglages » (le menu s'appelle « Paramètres ») |
//! | G4 | #488, #291 | trois plans comptables réels, ni Sterchi, ni KMU, ni import fictif |
//! | G4-bis | #488 | tout numéro de compte cité en exemple (manuel, guide, catalogues) existe dans un plan livré, sous son nom |
//! | G4-ter | #488 | la forme « NNNN Nom » de compte est juste partout ailleurs (manuels, README, code) |
//! | G5 | #458 | aucun « dossier surveillé » : l'import de factures se lance à la main |
//! | G6 | #449 | la faille KF-036 n'est plus annoncée ouverte : corrigée depuis la v0.10.0 |
//! | G7 | #432 | toute référence d'issue du README est un lien vers la même issue |
//! | G9 | #569, #547 | les replis Rust égalent la valeur fr-CH de leur clé |
//! | G12 | #569 | le manuel et le guide d'API disent l'ordre de réouverture |

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;

use regex::Regex;

/// Racine du dépôt.
fn racine() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Lit un fichier du dépôt (chemin relatif à la racine) ; panique s'il est absent.
fn lire(relatif: &str) -> String {
    let chemin = racine().join(relatif);
    std::fs::read_to_string(&chemin)
        .unwrap_or_else(|e| panic!("lecture de {}: {e}", chemin.display()))
}

/// Les trois sources LaTeX des manuels français, avec leur nom.
fn manuels_fr() -> Vec<(&'static str, String)> {
    let manuels = [
        "docs/manual/fr/user-manual.tex",
        "docs/manual/fr/admin-manual.tex",
        "docs/manual/fr/marketing-brochure.tex",
    ];
    let lus: Vec<_> = manuels.iter().map(|m| (*m, lire(m))).collect();
    // Positif : chacun est bien un document LaTeX non vide.
    for (nom, texte) in &lus {
        assert!(texte.contains("\\begin{document}"), "{nom} lu à vide");
    }
    // Aucun manuel fr n'échappe à la liste ci-dessus.
    let presents = std::fs::read_dir(racine().join("docs/manual/fr"))
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .path()
                .extension()
                .and_then(|x| x.to_str())
                == Some("tex")
        })
        .count();
    assert_eq!(
        presents,
        manuels.len(),
        "un .tex de docs/manual/fr n'est pas gardé"
    );
    lus
}

/// Valeurs `fr-CH`, **lignes de continuation comprises** : une ligne qui commence par
/// un blanc prolonge la valeur de la clé précédente (jointe par une espace, sauf à une
/// valeur encore vide — forme bloc `clé =` puis lignes indentées). Toute autre ligne —
/// tête de clé, commentaire, ligne vide, `}` de clôture d'un sélecteur écrite en
/// colonne 0 — clôt la valeur sans s'y ajouter. Sans cela, une clé passée sur
/// plusieurs lignes serait lue tronquée à sa première ligne, et G9 rougirait sans
/// régression.
///
/// ⚠️ Trois analyseurs du même format, dans deux crates et un fichier TypeScript qui ne
/// partagent pas de code de test : celui-ci, `valeurs_brutes` (`kesh-i18n/src/loader.rs`)
/// et `valeurDuCatalogueFr` (Vitest G13). Ils appliquent **la même règle** (tête
/// `^([a-zA-Z][\w-]*) = ?(.*)$`, continuation indentée, jonction ci-dessus) depuis la
/// revue de code P2 (B2-4, E2-4, A-4), et **chacun a son anti-test-muet**, sur les
/// mêmes trois cas réels du catalogue : clé multiligne en forme bloc
/// (`email-password-reset-body`), clé suivante qui n'en hérite rien
/// (`auth-recovery-forgot-title`), `}` de sélecteur non ajouté
/// (`error-account-not-postable`). La règle n'est pas toute la syntaxe Fluent (termes
/// `-x`, attributs `.x` ne sont pas lus) : aucune clé gardée ne les emploie.
fn catalogue_fr() -> HashMap<String, String> {
    let tete = Regex::new(r"^([a-zA-Z][\w-]*) = ?(.*)$").unwrap();
    let texte = lire("crates/kesh-i18n/locales/fr-CH/messages.ftl");
    let mut out: HashMap<String, String> = HashMap::new();
    let mut courante: Option<String> = None;
    for ligne in texte.lines() {
        if let Some(c) = tete.captures(ligne) {
            out.insert(c[1].to_string(), c[2].to_string());
            courante = Some(c[1].to_string());
        } else if ligne.starts_with([' ', '\t']) && !ligne.trim().is_empty() {
            if let Some(cle) = &courante {
                let v = out.get_mut(cle).unwrap();
                if !v.is_empty() {
                    v.push(' ');
                }
                v.push_str(ligne.trim());
            }
        } else {
            // Commentaire, ligne vide, `}` en colonne 0 : la valeur en cours est close.
            courante = None;
        }
    }
    assert!(out.len() > 100, "catalogue fr-CH lu à vide");
    out
}

/// Anti-test-muet de [`catalogue_fr`] — les trois cas réels de la règle commune aux
/// trois analyseurs (même test dans `kesh-i18n` et dans le Vitest G13) : une clé en
/// forme bloc sur cinq lignes est lue entière et sans blanc de tête, la clé suivante
/// n'en hérite rien, et le `}` de clôture d'un sélecteur n'est pas ajouté.
#[test]
fn le_catalogue_fr_joint_les_continuations() {
    let fr = catalogue_fr();
    let corps = &fr["email-password-reset-body"];
    assert!(
        corps.starts_with("Vous avez demandé") && corps.ends_with("ignorez cet email."),
        "continuations non jointes, ou blanc de tête : {corps:?}"
    );
    assert_eq!(fr["auth-recovery-forgot-title"], "Mot de passe oublié");
    let selecteur = &fr["error-account-not-postable"];
    assert!(
        selecteur.ends_with("choisissez des comptes imputables."),
        "`}}` de sélecteur ajouté, ou variante perdue : {selecteur:?}"
    );
}

/// **G2** (#547) — le menu s'appelle « Paramètres » ; aucun texte ne renvoie aux « Réglages ».
///
/// ⚠️ Élargi en revue de code P1 (B-1) : la première version cherchait des **formes**
/// (`\emph{Réglages}`, `Réglages et`, `(Réglages)`, `dans les réglages`), et
/// `README.md:213` — « éditables (Réglages, Admin) » — passait entre elles. Désormais
/// le mot capitalisé « Réglages » est interdit **sous toute forme** dans les manuels,
/// le README et `.env.example` : aucun de ces textes ne l'emploie comme nom d'objet
/// (les noms d'objet assumés — entités du journal d'audit — vivent au catalogue).
#[test]
fn aucun_renvoi_au_menu_reglages() {
    for (nom, texte) in manuels_fr() {
        assert!(!texte.contains("Réglages"), "{nom} : « Réglages »");
    }
    let manuel = lire("docs/manual/fr/user-manual.tex");
    assert!(
        manuel.contains("\\emph{Paramètres}"),
        "user-manual.tex : positif"
    );

    let env = lire(".env.example");
    assert!(!env.contains("Réglages"), ".env.example : « Réglages »");
    assert!(env.contains("(Paramètres)"), ".env.example : positif");

    let readme = lire("README.md");
    assert!(!readme.contains("Réglages"), "README.md : « Réglages »");
    assert!(
        !readme.contains("dans les réglages"),
        "README.md : dans les réglages"
    );
    assert!(
        readme.contains("dans les Paramètres"),
        "README.md : positif"
    );
}

/// **G4** (#488, #291) — Kesh livre trois plans (PME, indépendant, association),
/// choisis par le type d'organisation ; aucun plan « Sterchi » ni « KMU », aucun
/// import de plan ni de contacts.
#[test]
fn plans_comptables_reels() {
    let url = Regex::new(r"https?://\S+").unwrap();
    let kmu = Regex::new(r"\bKMU\b").unwrap();
    let mut sources = manuels_fr();
    sources.push(("README.md", lire("README.md")));
    sources.push((
        "docs/user-guide/fr/getting-started.md",
        lire("docs/user-guide/fr/getting-started.md"),
    ));
    for (nom, texte) in &sources {
        let sans_url = url.replace_all(texte, "");
        assert!(!texte.contains("Sterchi"), "{nom} : Sterchi");
        assert!(!kmu.is_match(&sans_url), "{nom} : KMU hors URL");
        assert!(
            !texte.contains("\\emph{Import CSV}"),
            "{nom} : \\emph{{Import CSV}}"
        );
        assert!(
            !texte.contains("import CSV custom"),
            "{nom} : import CSV custom"
        );
    }
    // Positifs : le texte neuf est là où on l'attend.
    assert!(lire("docs/manual/fr/user-manual.tex").contains("Kesh fournit trois plans comptables"));
    assert!(lire("docs/manual/fr/admin-manual.tex").contains("Kesh livre trois plans comptables"));
    assert!(
        lire("docs/user-guide/fr/getting-started.md")
            .contains("il détermine le plan comptable que Kesh met en place")
    );
}

/// Les trois plans livrés, lus par `kesh_core::chart_of_accounts::load_chart` (la
/// source, non une copie) : pour chaque langue (`fr`, `de`, `it`, `en`), numéro →
/// noms que les plans lui donnent, apostrophes typographiques ramenées à `'`.
fn plans_livres() -> HashMap<&'static str, HashMap<String, Vec<String>>> {
    use kesh_core::chart_of_accounts::load_chart;
    let mut out: HashMap<&'static str, HashMap<String, Vec<String>>> = HashMap::new();
    for org in ["Pme", "Independant", "Association"] {
        for e in load_chart(org).expect("plan livré") {
            for langue in ["fr", "de", "it", "en"] {
                let nom = apostrophes(e.name.get(langue).map_or("", String::as_str));
                out.entry(langue)
                    .or_default()
                    .entry(e.number.clone())
                    .or_default()
                    .push(nom);
            }
        }
    }
    // Positifs : les plans sont lus, et le trou des années est réel.
    let fr = &out["fr"];
    assert!(fr.contains_key("6940") && fr.contains_key("3100"));
    assert!(
        !fr.keys()
            .any(|n| n.len() == 4 && ("2001".."2100").contains(&n.as_str())),
        "un plan livré a un compte entre 2001 et 2099 : la garde le prendrait pour une année"
    );
    out
}

/// Ramène l'apostrophe typographique à l'apostrophe droite : un nom écrit
/// « Différences d’arrondi » est le même que celui du plan, « Différences d'arrondi ».
fn apostrophes(s: &str) -> String {
    s.replace('’', "'")
}

/// Premier mot de chaque nom de compte d'une langue : c'est ce qui fait reconnaître
/// la forme libre « NNNN Nom » sans prendre un montant ou un NPA pour un compte.
fn premiers_mots(noms: &HashMap<String, Vec<String>>) -> std::collections::HashSet<String> {
    noms.values()
        .flatten()
        .filter_map(|n| n.split_whitespace().next().map(str::to_string))
        .collect()
}

/// Un nombre de quatre chiffres **isolé** de `ligne` : `(numéro, décimales, début, fin)`.
///
/// Les bornes se lisent sur le caractère qui précède et celui qui suit, **sans les
/// consommer** (revue de code P2, E2-3) : dans `1000 1030`, les deux nombres sont lus.
/// Ne sont pas des comptes : un nombre collé à un chiffre, une lettre ASCII, `_`, une
/// apostrophe (séparateur de milliers), un `%` ; précédé de `.`, `,`, `:`, `#` ; ou d'un
/// `-` lui-même collé à un caractère alphanumérique (`ISO-8859`, `F-2026-0042`).
/// `{` et `}` ne bornent plus rien (revue de code P2, B2-2) : `\textbf{3600}` est lu.
fn nombres_isoles(ligne: &str) -> Vec<(String, String, usize, usize)> {
    static NOMBRE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([1-9]\d{3})(\.\d+)?").unwrap());
    let mut out = Vec::new();
    for c in NOMBRE.captures_iter(ligne) {
        let m = c.get(0).unwrap();
        let mut avant = ligne[..m.start()].chars().rev();
        let precedent = avant.next();
        let exclu_avant = match precedent {
            None => false,
            Some('-') => avant.next().is_some_and(char::is_alphanumeric),
            Some(p) => p.is_ascii_digit() || p.is_ascii_alphabetic() || ".,'’:#_".contains(p),
        };
        let exclu_apres = ligne[m.end()..]
            .chars()
            .next()
            .is_some_and(|s| s.is_ascii_digit() || s.is_ascii_alphabetic() || "'’%_".contains(s));
        if !exclu_avant && !exclu_apres {
            out.push((
                c[1].to_string(),
                c.get(2).map_or("", |d| d.as_str()).to_string(),
                m.start(),
                m.end(),
            ));
        }
    }
    out
}

/// Vrai si le nombre `[debut, fin)` de `ligne` est un **montant** (une devise le
/// précède ou le suit) ou un **NPA** (sans décimales, suivi d'un mot à majuscule qui
/// n'ouvre aucun nom de compte et qui termine le segment : `1003 Lausanne,`).
fn montant_ou_npa(
    ligne: &str,
    debut: usize,
    fin: usize,
    decimales: &str,
    premiers: &std::collections::HashSet<String>,
) -> bool {
    let blancs: &[char] = &[' ', '~', '\u{a0}'];
    let avant = ligne[..debut].trim_end_matches(blancs);
    let apres = ligne[fin..].trim_start_matches(blancs);
    if ["CHF", "Fr.", "EUR"].iter().any(|d| avant.ends_with(d))
        || ["CHF", "EUR", "francs"]
            .iter()
            .any(|d| apres.starts_with(d))
    {
        return true;
    }
    static NPA: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^ (\p{Lu}[\p{L}-]+)(?:$|[,.;)}\\])").unwrap());
    decimales.is_empty()
        && NPA
            .captures(&ligne[fin..])
            .is_some_and(|c| !premiers.contains(&c[1]))
}

/// **G4-bis** (#488, revue de code P1, E-1/E-5 ; P2, E2-1, E2-3, B2-2, B2-3) — tout
/// numéro de compte que le manuel utilisateur, le guide de démarrage ou un **catalogue**
/// (quatre locales) cite **existe dans un des trois plans livrés**, et, quand il est
/// nommé, **sous le nom que ce plan lui donne**.
///
/// ⚠️ Pourquoi un inventaire, et non une liste de numéros interdits : l'exemple
/// « Association » faisait créer 3600/3601, absents des trois plans ; la relecture a
/// trouvé six autres numéros ou noms faux (1030, 4200 « Charges de personnel »,
/// 6500 « Entretien », 6997/7997, 3200 « Honoraires »), puis la revue P2 l'écran
/// *Comptes bancaires* (« 1020 Caisse, 1030 Banque », quatre locales), hors du
/// périmètre d'alors. Interdire ces formes-là laisserait passer la suivante ; la garde
/// inventorie donc les nombres de quatre chiffres isolés ([`nombres_isoles`]) et exige
/// que chacun soit un compte livré, **sauf** :
///
/// - les années 2001 à 2099 (aucun plan livré n'y a de compte, ce que la garde
///   vérifie) ;
/// - un montant : une devise (`CHF`, `Fr.`, `EUR`, `francs`) le précède ou le suit ;
/// - un NPA : nombre sans décimales suivi d'un mot à majuscule qui n'ouvre aucun nom de
///   compte et qui clôt le segment (`1003 Lausanne,` ; [`montant_ou_npa`]).
///
/// Un sous-compte (`3200.1`, `1020.001`) est un compte **à créer** : seul son parent
/// doit exister — le texte doit le dire à créer, ce que la garde ne lit pas.
///
/// Formes nommées contrôlées : `NNNN « Nom »`, `\texttt{NNNN Nom}`, `NNNN \emph{Nom}`,
/// `` `NNNN Nom` ``, `NNNN (Nom)`, les listes entre parenthèses `(NNNN Nom, NNNN Nom)`
/// (sans les années ni les devises), et la **forme libre** de G4-ter (`NNNN Mot`, quand
/// `Mot` ouvre un nom de compte de la langue). Les formes à guillemets ne valent que
/// pour le français ; un catalogue de/it/en n'est contrôlé que par la forme libre.
///
/// **Angles morts déclarés** :
/// - un numéro faux précédé d'un `-` collé à une lettre ou un chiffre (`1000-1030`,
///   `ISO-8859`) n'est pas lu ;
/// - un nombre de 1000 à 2000, ou de 2100 à 9999, écrit pour une année (« en 1999 »)
///   ou pour une borne (« de 1 à 5000 ») est pris pour un compte : rouge bruyant, le
///   message cite la ligne — 1000 (Caisse) passe, faux vert sans conséquence ;
/// - un NPA suivi d'un nom de localité en plusieurs mots (`2300 La Chaux-de-Fonds`) ou
///   d'un mot qui ouvre aussi un nom de compte rougit ; un numéro faux suivi d'un tel
///   mot à majuscule en fin de segment passe pour un NPA ;
/// - un montant sans devise adjacente (`1500.00` seul) est pris pour un compte.
#[test]
fn les_comptes_cites_en_exemple_existent_dans_les_plans_livres() {
    let plans = plans_livres();
    let noms = &plans["fr"];

    let nommes = [
        Regex::new(r"\b([1-9]\d{3}) « ([^»]+?) »").unwrap(),
        Regex::new(r"\\texttt\{([1-9]\d{3}) ([^}]+)\}").unwrap(),
        Regex::new(r"\b([1-9]\d{3}) \\emph\{([^}]+)\}").unwrap(),
        Regex::new(r"`([1-9]\d{3}) ([^`]+)`").unwrap(),
        Regex::new(r"\b([1-9]\d{3}) \((\p{Lu}[^()\d]*)\)").unwrap(),
    ];
    let parentheses = Regex::new(r"\(([^()]*)\)").unwrap();
    let element = Regex::new(r"^([1-9]\d{3}) (\p{Lu}[^«»]*)$").unwrap();

    let mut vus = 0;
    let mut nommes_vus = 0;
    let mut catalogues_vus = 0;
    let fichiers = [
        ("docs/manual/fr/user-manual.tex", "fr"),
        ("docs/user-guide/fr/getting-started.md", "fr"),
        ("crates/kesh-i18n/locales/fr-CH/messages.ftl", "fr"),
        ("crates/kesh-i18n/locales/de-CH/messages.ftl", "de"),
        ("crates/kesh-i18n/locales/it-CH/messages.ftl", "it"),
        ("crates/kesh-i18n/locales/en-CH/messages.ftl", "en"),
    ];
    for (nom_fichier, langue) in fichiers {
        let premiers = premiers_mots(&plans[langue]);
        let texte = lire(nom_fichier);
        let catalogue = nom_fichier.ends_with(".ftl");
        for ligne in texte.lines() {
            for (n, decimales, debut, fin) in nombres_isoles(ligne) {
                if ("2001".."2100").contains(&n.as_str())
                    || montant_ou_npa(ligne, debut, fin, &decimales, &premiers)
                {
                    continue;
                }
                assert!(
                    noms.contains_key(&n),
                    "{nom_fichier} : compte {n}{decimales} absent des trois plans livrés : {ligne}"
                );
                vus += 1;
                if catalogue {
                    catalogues_vus += 1;
                }
            }
            nommes_vus += controler_forme_libre(nom_fichier, ligne, &plans[langue], &premiers);
            if langue != "fr" {
                continue;
            }
            let mut controler = |n: &str, nom: &str| {
                let livres = noms
                    .get(n)
                    .unwrap_or_else(|| panic!("{nom_fichier} : compte {n} absent : {ligne}"));
                let nom = apostrophes(nom.trim());
                assert!(
                    livres.iter().any(|l| *l == nom),
                    "{nom_fichier} : {n} « {nom} » — les plans livrés le nomment {livres:?}"
                );
                nommes_vus += 1;
            };
            for re in &nommes {
                for c in re.captures_iter(ligne) {
                    controler(&c[1], &c[2]);
                }
            }
            for p in parentheses.captures_iter(ligne) {
                for item in p[1].split(", ") {
                    if let Some(c) = element.captures(item.trim()) {
                        let (n, nom) = (&c[1], c[2].trim());
                        if ("2001".."2100").contains(&n) || ["CHF", "EUR", "Fr."].contains(&nom) {
                            continue;
                        }
                        controler(n, nom);
                    }
                }
            }
        }
    }
    assert!(vus > 30, "numéros de compte lus : {vus}");
    assert!(nommes_vus > 20, "comptes nommés lus : {nommes_vus}");
    // Les catalogues citent l'écran Comptes bancaires (1010, 1020, 1020.001, 1020),
    // dans chacune des quatre locales : lus, ils ne sont pas lus à vide.
    assert!(
        catalogues_vus >= 16,
        "numéros lus dans les catalogues : {catalogues_vus}"
    );
}

/// Forme libre « NNNN Mot » d'une ligne : quand `Mot` ouvre un nom de compte de la
/// langue (`Caisse`, `Banque`, `Kasse`…), le numéro doit exister et l'un de ses noms
/// doit commencer par ce mot. Rend le nombre de formes contrôlées.
///
/// ⚠️ C'est ce qui ne prend ni un montant (« 1500 CHF »), ni un NPA (« 1003 Lausanne »),
/// ni une année pour un compte : `CHF`, `Lausanne` n'ouvrent aucun nom de compte.
/// Angle mort déclaré : un numéro faux suivi d'un mot qui n'ouvre aucun nom de compte
/// (« 1030 BCV ») n'est pas contrôlé ici — il l'est par l'inventaire de G4-bis là où
/// celui-ci s'applique.
fn controler_forme_libre(
    nom_fichier: &str,
    ligne: &str,
    noms: &HashMap<String, Vec<String>>,
    premiers: &std::collections::HashSet<String>,
) -> usize {
    static MOT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^ ([\p{L}'’]+)").unwrap());
    let mut controles = 0;
    for (n, decimales, _, fin) in nombres_isoles(ligne) {
        if !decimales.is_empty() || ("2001".."2100").contains(&n.as_str()) {
            continue;
        }
        let Some(c) = MOT.captures(&ligne[fin..]) else {
            continue;
        };
        let m = apostrophes(&c[1]);
        if !premiers.contains(&m) {
            continue;
        }
        let livres = noms.get(&n).unwrap_or_else(|| {
            panic!("{nom_fichier} : compte {n} « {m} … » absent des trois plans livrés : {ligne}")
        });
        assert!(
            livres
                .iter()
                .any(|l| l.split_whitespace().next() == Some(m.as_str())),
            "{nom_fichier} : {n} « {m} … » — les plans livrés le nomment {livres:?} : {ligne}"
        );
        controles += 1;
    }
    controles
}

/// Fichiers d'une arborescence du dépôt dont le nom satisfait `garder`, récursivement.
fn fichiers_sous(relatif: &str, garder: &dyn Fn(&str) -> bool) -> Vec<PathBuf> {
    let mut pile = vec![racine().join(relatif)];
    let mut out = Vec::new();
    while let Some(dossier) = pile.pop() {
        for e in std::fs::read_dir(&dossier)
            .unwrap_or_else(|e| panic!("lecture de {}: {e}", dossier.display()))
        {
            let chemin = e.unwrap().path();
            let nom = chemin.file_name().unwrap().to_string_lossy().into_owned();
            if chemin.is_dir() {
                if nom != "node_modules" && nom != "target" {
                    pile.push(chemin);
                }
            } else if garder(&nom) {
                out.push(chemin);
            }
        }
    }
    out
}

/// **G4-ter** (#488, revue de code P2, E2-1) — la forme libre « NNNN Nom » de compte
/// ([`controler_forme_libre`]) est juste **partout où un texte la porte** hors des
/// fichiers de G4-bis : manuels d'administration et brochure, README, guide d'API, et
/// le code — replis Svelte et TypeScript, doc-comments et replis Rust.
///
/// ⚠️ Pourquoi la forme libre seule, et non l'inventaire de G4-bis : le code et le
/// manuel d'administration sont pleins de nombres de quatre chiffres qui ne sont pas
/// des comptes (ports, codes MariaDB, montants de test) ; l'inventaire y serait une
/// liste d'exceptions. La forme libre, elle, ne se déclenche que sur un mot qui ouvre
/// un nom de compte. Les replis frontend des clés de l'écran *Comptes bancaires* sont
/// en outre tenus égaux au catalogue par G13, que G4-bis lit en entier.
///
/// Exclus : les tests (`*.test.ts`, `*.spec.ts`, `tests/`) et
/// `crates/kesh-db/src/test_fixtures.rs`, dont les comptes (« 1100 Banque »,
/// « 2000 Capital ») sont des données de test arbitraires, non un texte montré.
#[test]
fn la_forme_libre_nnnn_nom_est_juste_partout() {
    let plans = plans_livres();
    let premiers = premiers_mots(&plans["fr"]);
    let mut sources: Vec<PathBuf> = [
        "docs/manual/fr/admin-manual.tex",
        "docs/manual/fr/marketing-brochure.tex",
        "README.md",
        "docs/api-external.md",
    ]
    .iter()
    .map(|r| racine().join(r))
    .collect();
    sources.extend(fichiers_sous("frontend/src", &|n| {
        (n.ends_with(".svelte") || n.ends_with(".ts"))
            && !n.ends_with(".test.ts")
            && !n.ends_with(".spec.ts")
    }));
    let rust = fichiers_sous("crates", &|n| n.ends_with(".rs") && n != "test_fixtures.rs");
    let rust: Vec<_> = rust
        .into_iter()
        .filter(|p| !p.components().any(|c| c.as_os_str() == "tests"))
        .collect();
    assert!(rust.len() > 100, "sources Rust lues : {}", rust.len());
    sources.extend(rust);
    let mut controles = 0;
    for chemin in &sources {
        let texte = std::fs::read_to_string(chemin)
            .unwrap_or_else(|e| panic!("lecture de {}: {e}", chemin.display()));
        let nom = chemin.display().to_string();
        for ligne in texte.lines() {
            controles += controler_forme_libre(&nom, ligne, &plans["fr"], &premiers);
        }
    }
    // Positif : le repli Svelte de l'écran Comptes bancaires et les doc-comments
    // corrigés en P2 (`errors.rs`, `bank_account.rs`) portent la forme.
    assert!(
        controles >= 6,
        "formes « NNNN Nom » contrôlées : {controles}"
    );
}

/// **G5** (#458, refs #459) — aucun processus ne surveille le dossier d'import :
/// l'import se lance depuis l'écran.
///
/// ⚠️ Le jour où #459 automatisera l'import, la phrase « L'import ne se déclenche
/// pas tout seul » du manuel utilisateur deviendra fausse : l'inverser avec ce test.
#[test]
fn aucun_dossier_surveille() {
    for (nom, texte) in manuels_fr() {
        assert!(
            !texte.contains("dossier surveillé"),
            "{nom} : dossier surveillé"
        );
    }
    assert!(
        lire("docs/manual/fr/user-manual.tex").contains("L'import ne se déclenche pas tout seul")
    );

    let env = lire(".env.example");
    assert!(
        !env.contains("dossier surveillé"),
        ".env.example : dossier surveillé"
    );
    assert!(env.contains("KESH_INBOX_DIR"), ".env.example : positif");

    let readme = lire("README.md");
    // Sans exemption : la ligne de feuille de route v0.4.0 disait elle aussi
    // « dossier surveillé », et c'était faux dès la v0.4.0, dont l'import passait
    // déjà par `POST /api/v1/inbox-import` (revue de code P1, B-2, C-15-14-44).
    assert!(
        !readme.contains("dossier surveillé"),
        "README.md : dossier surveillé"
    );
    assert!(
        readme.contains("import lancé depuis l'écran"),
        "README.md : positif"
    );
}

/// **G6** (#449) — la fermeture KF-036 (#167) est livrée depuis la v0.10.0 ; aucun
/// texte ne l'annonce encore à venir ni ne renvoie à une section `[Unreleased]`,
/// qui se renomme à chaque release.
#[test]
fn la_faille_kf036_n_est_pas_annoncee_ouverte() {
    for nom in ["docs/api-external.md", "docs/manual/fr/admin-manual.tex"] {
        let texte = lire(nom);
        for interdit in [
            "Unreleased",
            "n'est **pas** dans la v0.9.0",
            "n'est pas dans la v0.9.0",
            "Pas dans celle que décrit ce manuel",
        ] {
            assert!(!texte.contains(interdit), "{nom} : « {interdit} »");
        }
        let mut vus = 0;
        for (i, _) in texte.match_indices("Dans quelle version") {
            let debut = texte[..i].rfind('\n').map_or(0, |n| n + 1);
            let fin = texte[i..].find("\n\n").map_or(texte.len(), |n| i + n);
            assert!(
                texte[debut..fin].contains("v0.10.0"),
                "{nom} : « Dans quelle version » sans v0.10.0 : {}",
                &texte[debut..fin]
            );
            vus += 1;
        }
        assert!(vus > 0, "{nom} : « Dans quelle version » absent");
    }
}

/// **G7** (#432) — toute référence `#N` du README est un lien vers l'issue `N`.
#[test]
fn references_d_issues_du_readme_sont_des_liens() {
    let readme = lire("README.md");
    let lien =
        Regex::new(r"\[#(\d+)\]\(https://github\.com/guycorbaz/kesh/issues/(\d+)\)").unwrap();
    let mut liens = 0;
    for c in lien.captures_iter(&readme) {
        assert_eq!(&c[1], &c[2], "lien vers une autre issue : {}", &c[0]);
        liens += 1;
    }
    assert!(
        liens > 0,
        "README.md : aucun lien d'issue (lecture à vide ?)"
    );
    let reste = lien.replace_all(&readme, "");
    let nue = Regex::new(r"#\d+").unwrap();
    let nues: Vec<_> = nue.find_iter(&reste).map(|m| m.as_str()).collect();
    assert!(nues.is_empty(), "références d'issue sans lien : {nues:?}");
}

/// **G9** (#569, #547) — chaque repli Rust égale la valeur fr-CH de sa clé.
///
/// Table fermée (fichier → clés, nombre de sites, paramètres). La comparaison se fait
/// après normalisation des continuations de chaîne Rust (`\` en fin de ligne, saut de
/// ligne et blancs de tête retirés) : un repli écrit sur plusieurs lignes est juste.
///
/// Un repli **paramétré** (`format!`) se compare à la valeur du catalogue dont chaque
/// variable Fluent est réécrite en variable Rust (`{ $name }` → `{fiscal_year_name}`) :
/// revue de code P2 (A-1, E2-7), qui y a fait entrer les deux replis de la famille
/// `LATER_FISCAL_YEAR_CLOSED` — restés à la formule non bornée et aux apostrophes
/// droites, faute d'être comparés.
#[test]
fn les_replis_rust_suivent_le_catalogue() {
    const NOM: &[(&str, &str)] = &[("{ $name }", "{fiscal_year_name}")];
    type Cles = &'static [(&'static str, usize, &'static [(&'static str, &'static str)])];
    const TABLE: [(&str, Cles); 3] = [
        (
            "crates/kesh-api/src/errors.rs",
            &[
                ("settlement-cancel-blocked-fiscal-year-closed", 1, &[]),
                ("reconciliation-cancel-blocked-fiscal-year-closed", 1, &[]),
                (
                    "supplier-invoices-cancel-blocked-fiscal-year-closed",
                    1,
                    &[],
                ),
                ("error-invoice-pdf-header-overflow", 1, &[]),
                ("error-fiscal-year-create-later-closed", 1, NOM),
                ("error-later-fiscal-year-closed", 1, NOM),
            ],
        ),
        (
            "crates/kesh-api/src/routes/fiscal_years.rs",
            &[("error-fiscal-year-reopen-blocked", 1, &[])],
        ),
        (
            "crates/kesh-api/src/routes/opening_balances.rs",
            &[("error-opening-balances-first-year-closed", 2, &[])],
        ),
    ];
    let continuation = Regex::new(r"\\\n\s*").unwrap();
    let fr = catalogue_fr();
    for (fichier, cles) in TABLE {
        let source = continuation.replace_all(&lire(fichier), "").into_owned();
        for (cle, sites, parametres) in cles {
            let mut valeur = fr
                .get(*cle)
                .unwrap_or_else(|| panic!("fr-CH : {cle} absente"))
                .clone();
            for (fluent, rust) in *parametres {
                assert!(valeur.contains(fluent), "fr-CH : {cle} sans {fluent}");
                valeur = valeur.replace(fluent, rust);
            }
            let litteral = format!("\"{valeur}\"");
            assert_eq!(
                source.matches(&litteral).count(),
                *sites,
                "{fichier} : repli de {cle} différent du catalogue ({valeur})"
            );
        }
        for perime in [
            "rouvrir l'exercice pour",
            "rouvrez-le",
            "dans les réglages :",
        ] {
            assert!(!source.contains(perime), "{fichier} : « {perime} »");
        }
    }
}

/// Déplie les commandes de mise en forme LaTeX jusqu'à stabilité (imbrications
/// comprises), remplace `~` par une espace et réduit les blancs : une phrase coupée
/// sur deux lignes ou mise en gras en son milieu se lit d'un tenant.
fn normaliser(texte: &str) -> String {
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
    let blancs = Regex::new(r"\s+").unwrap();
    blancs
        .replace_all(&courant.replace('~', " "), " ")
        .into_owned()
}

/// **G12** (#569) — le manuel utilisateur et le guide d'API disent l'ordre de
/// réouverture (« en commençant par le plus récent »).
#[test]
fn le_manuel_dit_l_ordre_de_reouverture() {
    const MARQUEUR: &str = "en commençant par le plus récent";
    let manuel = normaliser(&lire("docs/manual/fr/user-manual.tex"));
    for interdit in ["doit d'abord le rouvrir", "doit d'abord rouvrir l'exercice"] {
        assert!(
            !manuel.contains(interdit),
            "user-manual.tex : « {interdit} »"
        );
    }
    let n = manuel.matches(MARQUEUR).count();
    assert!(
        n >= 7,
        "user-manual.tex : marqueur d'ordre {n} fois, 7 attendues au moins"
    );

    let api = normaliser(&lire("docs/api-external.md"));
    assert!(
        !api.contains("doit le rouvrir"),
        "api-external.md : « doit le rouvrir »"
    );
    assert!(
        api.contains(MARQUEUR),
        "api-external.md : marqueur d'ordre absent"
    );
}
