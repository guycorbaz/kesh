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
//! | G18 | #127 | une installation porte une société (Story 15-14b) |
//! | G18-ter | #127 | les textes d'écran « toutes les sociétés » (sauvegarde d'installation) sont une liste fermée assumée |
//! | G18-bis | #127 | le manuel utilisateur dit comment on entre : compte créé par l'administrateur, connexion par identifiant |
//! | G19 | release 0.13.0 | toute URL `raw.githubusercontent.com` du dépôt (manuels, README, site) pointe sur le tag `v<\keshVersion>`, jamais sur `main` |
//! | G20 | #616, #617, #619 | ni case « Compte de banque » au plan comptable, ni CAMT.053 présenté comme universel ; l'écran d'import dit « CAMT.053 ou CSV » dans les quatre langues |
//! | G21 | #697 | le `FROM rust:` du `Dockerfile` et le `channel` de `rust-toolchain.toml` désignent la même chaîne (même mineure, même patch si les deux en portent un) |
//! | G22 | #503 | la sauvegarde automatique : le manuel dit la reprise au bout d'une heure et le bandeau à partir d'au moins deux intervalles ; le rappel de l'écran dit le volume (nombre conservé plus une, une par import jamais purgée) dans les quatre langues |
//! | G23 | #723 | le serveur MCP : le manuel liste exactement les plages admises par défaut et les cinq portées (dans l'ordre), nomme `100.64.0.0/10` hors des défauts ; l'écran le nomme, ainsi que Traefik et Docker, dans les quatre langues |

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;

use regex::Regex;

#[path = "common/manuel.rs"]
mod manuel;
use manuel::normaliser;

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
    catalogue("fr-CH")
}

/// Le catalogue d'une locale (`fr-CH`, `de-CH`, `en-CH`, `it-CH`), lu par la même
/// règle que [`catalogue_fr`], qui n'en est que le cas `fr-CH` (Story 26-3, G20).
fn catalogue(locale: &str) -> HashMap<String, String> {
    let tete = Regex::new(r"^([a-zA-Z][\w-]*) = ?(.*)$").unwrap();
    let texte = lire(&format!("crates/kesh-i18n/locales/{locale}/messages.ftl"));
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
    assert!(out.len() > 100, "catalogue {locale} lu à vide");
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
/// Une date ISO `NNNN-MM-JJ` n'est pas un compte, à condition que le mois soit
/// `01`-`12` et le jour `01`-`31` (Story 32-1a, #716, E6) : `1171-99-99` est lu.
/// Limite assumée : `2026-02-30` reste une date — la regex ne connaît pas la
/// longueur des mois.
fn nombres_isoles(ligne: &str) -> Vec<(String, String, usize, usize)> {
    static NOMBRE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([1-9]\d{3})(\.\d+)?").unwrap());
    static DATE_ISO: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01])(\D|$)").unwrap());
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
        // Story 31-5 (#469, C-31-5-23) — une date ISO `NNNN-MM-JJ` (« entre
        // 1000-01-01 et 9999-12-31 ») n'est pas un compte : l'année est suivie
        // de `-MM-JJ` exactement.
        let date_iso = c.get(2).is_none() && DATE_ISO.is_match(&ligne[m.end()..]);
        if !exclu_avant && !exclu_apres && !date_iso {
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

/// Story 32-1a (#716, E6) — la reconnaissance d'une date ISO, fixée par des cas.
///
/// Négatifs : une date au mois `01`-`12` et au jour `01`-`31` n'est pas un compte
/// (aucun nombre rendu). Positifs : un mois ou un jour hors de ces bornes ne fait pas
/// d'une suite `NNNN-MM-JJ` une date, et le nombre est rendu comme compte possible.
/// Limite assumée : `2026-02-30` reste une date (la regex ne connaît pas la longueur
/// des mois).
#[test]
fn nombres_isoles_ne_prend_pour_date_que_des_mois_et_jours_bornes() {
    for date in ["entre 1000-01-01 et 9999-12-31", "au 2026-02-28."] {
        assert!(
            nombres_isoles(date).is_empty(),
            "« {date} » : une date ISO n'est pas un compte, rendu {:?}",
            nombres_isoles(date)
        );
    }
    for pas_une_date in ["1171-99-99", "1171-13-01", "1171-00-10", "1171-12-32"] {
        let rendus: Vec<String> = nombres_isoles(pas_une_date)
            .into_iter()
            .map(|(n, ..)| n)
            .collect();
        assert_eq!(
            rendus,
            vec!["1171".to_string()],
            "« {pas_une_date} » n'est pas une date : le nombre doit être rendu"
        );
    }
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
/// - l'année d'une date ISO `NNNN-MM-JJ` (`9999-12-31`, Story 31-5) ;
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
                    livres.contains(&nom),
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
    const DEVISES: &[(&str, &str)] = &[("{ $currencies }", "{currencies}")];
    const CHAMP: &[(&str, &str)] = &[("{ $field }", "{nom}")];
    const CHAMP_MAX: &[(&str, &str)] = &[("{ $field }", "{nom}"), ("{ $max }", "{maximum}")];
    // Story 32-5c (refs #723) — les refus des routes de gestion du serveur MCP.
    const PORTEE: &[(&str, &str)] = &[("{ $scope }", "{portee}")];
    const ENTREE: &[(&str, &str)] = &[("{ $entry }", "{entree}")];
    const ENTREE_ATTENDU: &[(&str, &str)] =
        &[("{ $entry }", "{entree}"), ("{ $expected }", "{attendu}")];
    type Cles = &'static [(&'static str, usize, &'static [(&'static str, &'static str)])];
    const TABLE: [(&str, Cles); 7] = [
        // Story 31-5 (#469, C-31-5-15) — les six refus du journal d'audit.
        (
            "crates/kesh-api/src/routes/audit_log.rs",
            &[
                ("audit-log-query-error-date-range-inverted", 1, &[]),
                ("audit-log-query-error-date-invalid", 1, CHAMP),
                ("audit-log-query-error-date-out-of-range", 1, CHAMP),
                ("audit-log-query-error-entity-id-not-positive", 1, &[]),
                ("audit-log-query-error-entity-id-without-type", 1, &[]),
                ("audit-log-query-error-code-too-long", 1, CHAMP_MAX),
            ],
        ),
        (
            "crates/kesh-api/src/errors.rs",
            &[
                ("settlement-cancel-blocked-fiscal-year-closed", 1, &[]),
                // Story 30-1a (#686, AC-a6, T-a13) — le texte « rapproché »,
                // vrai sur la fiche client comme sur la fiche fournisseur.
                ("settlement-cancel-blocked-bank-match", 1, &[]),
                ("reconciliation-cancel-blocked-fiscal-year-closed", 1, &[]),
                (
                    "supplier-invoices-cancel-blocked-fiscal-year-closed",
                    1,
                    &[],
                ),
                ("error-invoice-pdf-header-overflow", 1, &[]),
                // Story 15-1a2-0 (#518) — le rang 2 bis, un bras par famille, et
                // le message neutre de `LETTERING_IS_DOCUMENT` (validation P1, R-2).
                ("settlement-cancel-blocked-lettering-closed", 1, &[]),
                ("reconciliation-cancel-blocked-lettering-closed", 1, &[]),
                ("supplier-invoices-cancel-blocked-lettering-closed", 1, &[]),
                ("error-lettering-is-document", 1, &[]),
                ("error-fiscal-year-create-later-closed", 1, NOM),
                ("error-later-fiscal-year-closed", 1, NOM),
                // Story 29-1b (#232) — les deux refus de la contre-passation au
                // début de l'exercice suivant.
                ("error-next-fiscal-year-missing", 1, NOM),
                ("error-next-fiscal-year-closed", 1, NOM),
                // Story 27-1 (#632) — revue de code P1, E4 : les trois
                // replis neufs ou modifiés de la suppression d'import.
                ("error-api-key-operation-forbidden", 1, &[]),
                (
                    "bank-import-errors-has-reconciled-transactions",
                    1,
                    &[("{ $count }", "{reconciled_count}")],
                ),
                ("bank-accounts-errors-has-transactions", 1, &[]),
                // Story 27-1 — revue de code P2, E-P2-3 : les trois messages de
                // la 26-1 qu'assertent les tests `ubs_*` de `bank_imports_e2e`.
                // H2 y charge le catalogue `fr-CH` pour tout le processus : sous
                // `cargo test` ces tests lisent le catalogue, sous nextest le
                // repli — les deux doivent rester égaux.
                ("bank-import-csv-errors-amount-sign-refused-profile", 1, &[]),
                (
                    "bank-import-csv-errors-amount-sign-refused-lines",
                    1,
                    &[("{ $count }", "{total_errors}")],
                ),
                (
                    "bank-import-csv-errors-header-skips-data",
                    1,
                    &[("{ $line }", "{line}")],
                ),
                // Story 27-2 (#624) — le refus de devise, chemin CSV compris.
                ("bank-import-errors-unsupported-currency", 1, &[]),
                ("bank-import-csv-errors-mixed-currencies", 1, DEVISES),
                // Story 27-3 (#626) — refus d'une empreinte d'aperçu périmée.
                ("bank-import-errors-preview-stale", 1, &[]),
                // Story 28-5a (#526) — refus de l'annulation d'un rejet.
                ("reconciliation-errors-transaction-not-rejected", 1, &[]),
                // Story 29-8 (#670) — refus du rattachement à un règlement.
                ("reconciliation-errors-settlement-not-linkable", 1, &[]),
                // Story 28-4a (#642) — le compte bancaire clos : les cinq clés
                // neuves, et le message du principal qui ne s'archive pas
                // (« ni archivé ni clos »).
                ("bank-accounts-errors-closed", 1, &[]),
                ("bank-accounts-errors-not-closed", 1, &[]),
                ("bank-accounts-errors-cannot-close-primary", 1, &[]),
                ("bank-accounts-errors-has-pending-transactions", 1, &[]),
                ("bank-accounts-errors-has-open-payment-batches", 1, &[]),
                ("bank-accounts-errors-cannot-archive-primary", 1, &[]),
                // Story v017-3b (#687) — une ligne de facture fournisseur qui
                // n'est pas au centime.
                (
                    "supplier-invoices-errors-line-not-in-centimes",
                    1,
                    &[("{ $line }", "{line}"), ("{ $lineTotal }", "{line_total}")],
                ),
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
        // Story 32-1d (#711, L-B7) — le refus du réglage de sauvegarde hors
        // domaine.
        (
            "crates/kesh-api/src/routes/backup_schedule.rs",
            &[("admin-backup-schedule-error-out-of-domain", 1, &[])],
        ),
        // Story 32-5c (refs #723, AC-c3) — les refus de la gestion des clés MCP.
        (
            "crates/kesh-api/src/routes/mcp_keys.rs",
            &[
                ("admin-mcp-keys-error-name-required", 1, &[]),
                ("admin-mcp-keys-error-name-too-long", 1, &[]),
                ("admin-mcp-keys-error-scopes-empty", 1, &[]),
                ("admin-mcp-keys-error-scope-unknown", 1, PORTEE),
                ("admin-mcp-keys-error-expiry-past", 1, &[]),
                // Revue de code P1 de la 32-5c (E1 = B9) — l'an 9999 de `DATETIME`.
                ("admin-mcp-keys-error-expiry-out-of-range", 1, &[]),
                ("admin-mcp-keys-error-send-validation-unknown", 1, &[]),
                ("admin-mcp-keys-error-version-invalid", 1, &[]),
            ],
        ),
        // Story 32-5c (refs #723, AC-c4) — les refus des réglages du serveur MCP.
        (
            "crates/kesh-api/src/routes/mcp_server_settings.rs",
            &[
                ("admin-mcp-settings-error-too-many-entries", 1, &[]),
                ("admin-mcp-settings-error-entry-too-long", 1, &[]),
                ("admin-mcp-settings-error-network-invalid", 1, ENTREE),
                (
                    "admin-mcp-settings-error-network-host-bits",
                    1,
                    ENTREE_ATTENDU,
                ),
                ("admin-mcp-settings-error-origin-invalid", 1, ENTREE),
            ],
        ),
    ];
    let continuation = Regex::new(r"\\\n\s*").unwrap();
    let fr = catalogue_fr();
    // Revue de code P1 de la 27-2 (E-4) : une variable Fluent renommée dans une
    // seule locale laisserait l'utilisateur de cette langue sans la valeur —
    // chaque paramètre se contrôle donc dans les QUATRE catalogues.
    let autres: Vec<(&str, HashMap<String, String>)> = ["de-CH", "en-CH", "it-CH"]
        .into_iter()
        .map(|l| (l, catalogue(l)))
        .collect();
    for (fichier, cles) in TABLE {
        let source = continuation.replace_all(&lire(fichier), "").into_owned();
        for (cle, sites, parametres) in cles {
            let mut valeur = fr
                .get(*cle)
                .unwrap_or_else(|| panic!("fr-CH : {cle} absente"))
                .clone();
            for (fluent, rust) in *parametres {
                assert!(valeur.contains(fluent), "fr-CH : {cle} sans {fluent}");
                for (locale, cat) in &autres {
                    let v = cat
                        .get(*cle)
                        .unwrap_or_else(|| panic!("{locale} : {cle} absente"));
                    assert!(v.contains(fluent), "{locale} : {cle} sans {fluent}");
                }
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

/// Le domaine de G18 : la documentation utilisateur (manuels français, README,
/// guide de démarrage) — la première part de l'inventaire de l'AC 3 de la
/// 15-14b. Les catalogues et le code (noms de table, de route, commentaires)
/// sont assumés en bloc à la fiche.
fn documentation_utilisateur() -> Vec<(String, String)> {
    let mut docs: Vec<(String, String)> = manuels_fr()
        .into_iter()
        .map(|(nom, texte)| (nom.to_string(), texte))
        .collect();
    docs.push(("README.md".into(), lire("README.md")));
    for chemin in fichiers_sous("docs/user-guide/fr", &|n: &str| n.ends_with(".md")) {
        let relatif = chemin
            .strip_prefix(racine())
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let texte = std::fs::read_to_string(&chemin).unwrap();
        docs.push((relatif, texte));
    }
    assert!(
        docs.len() >= 5,
        "documentation utilisateur lue à vide : {}",
        docs.len()
    );
    docs
}

/// Les formes qui promettent plusieurs sociétés par installation (motif de
/// l'inventaire de l'AC 3, une alternative par entrée). Le crate `regex` n'a
/// pas de lookahead : chaque alternative est cherchée **séparément**, ce qui
/// compte les occurrences qui se recouvrent (`plusieurs companies` et
/// `companies`) comme la commande `perl` d'inventaire (C-15-14-27).
/// « compte dédié » n'y entre que dans ses locutions `ou` / `via` : l'expression
/// nue est d'usage courant (« un compte dédié aux frais »). Élargi en revue de
/// code P1 (E-2, A-6) aux tournures plurielles et au titre « Multi-sociétés ».
const MULTI_SOCIETE: &[&str] = &[
    r"(?i)toutes? (?:les )?(?:sociétés|entreprises|companies)",
    r"(?i)ensemble des sociétés",
    r"(?i)entre (?:plusieurs )?(?:sociétés|dossiers)",
    r"(?i)(?:autre|seconde|deuxième) société",
    r"(?i)multi-soci\w*",
    // Revue de code P2 (E2-6) : formes équivalentes, aucun site au 2026-10-09.
    r"(?i)plusieurs (?:entreprises|mandants)",
    r"(?i)multi-?entreprises?",
    r"(?i)vos sociétés",
    r"(?i)plusieurs (?:sociétés|companies)",
    r"(?i)\bcompanies\b",
    r"(?i)(?:ou|via) un compte dédié",
    r"(?i)nouvelle company",
    r"(?i)super.?admin",
    r"(?i)kesh-cli",
    r"(?i)même instance",
    r"(?i)sélecteur (?:de société|multi-dossiers|permettant de basculer)",
];

/// Les occurrences **assumées** de [`MULTI_SOCIETE`] : liste fermée, un
/// fragment normalisé par site, chacun trouvé une et une seule fois.
const MULTI_SOCIETE_ASSUME: &[(&str, &str)] = &[
    // Dit l'absence (deux occurrences : `super-admin`, `sélecteur de société`).
    (
        "docs/manual/fr/admin-manual.tex",
        "il n'existe pas de rôle « super-admin » cross-société ni de sélecteur de société",
    ),
    // Dit l'absence.
    (
        "docs/manual/fr/user-manual.tex",
        "Il n'y a pas de sélecteur permettant de basculer entre plusieurs dossiers depuis un même compte",
    ),
    // Intention de feuille de route.
    (
        "docs/manual/fr/user-manual.tex",
        "Un sélecteur multi-dossiers est envisagé pour une version ultérieure",
    ),
    // Dit l'absence.
    (
        "docs/manual/fr/marketing-brochure.tex",
        "ne propose pas encore de créer ni de basculer entre plusieurs sociétés",
    ),
    // Intention de feuille de route, au même titre que la ligne qui précède.
    (
        "docs/manual/fr/marketing-brochure.tex",
        "Un sélecteur permettant de basculer entre dossiers depuis un même compte est prévu pour une version ultérieure",
    ),
    // Nom de table.
    ("README.md", "FK vers `companies.id`"),
    // Nom de table : la clé étrangère recréée par le remède de moteur (Story
    // 28-3, #633, § « Exporter (sauvegarde complète) »).
    (
        "docs/manual/fr/admin-manual.tex",
        "REFERENCES companies(id) ON DELETE CASCADE",
    ),
    // Ajoutés en revue de code P1 (motif élargi) — absence ou modèle, vrais
    // pour une installation à une société.
    // L'absence, deux phrases distinctes (public visé ; configuration).
    (
        "docs/manual/fr/admin-manual.tex",
        "Un dossier = une instance : l'interface ne crée pas de seconde société",
    ),
    (
        "docs/manual/fr/admin-manual.tex",
        "Mais une installation porte une société : l'interface ne crée pas de seconde société",
    ),
    // Le modèle de données.
    (
        "docs/manual/fr/admin-manual.tex",
        "Le modèle de données de Kesh est multi-société",
    ),
    // L'absence.
    (
        "docs/manual/fr/admin-manual.tex",
        "aucun écran ne rattache un compte à une autre société",
    ),
    // Le modèle de données.
    (
        "docs/manual/fr/marketing-brochure.tex",
        "Le modèle de données est multi-société",
    ),
    // L'absence, et l'intention de feuille de route.
    (
        "docs/manual/fr/marketing-brochure.tex",
        "(La bascule entre sociétés depuis l'interface reste à venir.)",
    ),
    // Story 32-6c (§ « Diagnostiquer le filtre d'adresse du serveur MCP ») :
    // ce que le journal écrit pour une installation qui porte plus d'une
    // société — le cas existe (réglages MCP par société, installations
    // touchées par #528/#542), la phrase décrit le journal et ne promet pas
    // qu'on puisse en créer une seconde.
    (
        "docs/manual/fr/admin-manual.tex",
        "Avec plusieurs sociétés activées, chaque requête écrit",
    ),
    // Story 32-7 (§ « Le serveur MCP », fiche AC-18 (4) et (5)) : ce que voit la
    // clé d'une société quand une AUTRE société servie admet l'adresse ou
    // l'origine — le cas existe (réglages MCP par société, installations
    // touchées par #528/#542) ; les phrases décrivent le refus et ne promettent
    // pas qu'on puisse créer une seconde société.
    (
        "docs/manual/fr/admin-manual.tex",
        "Sur une installation à plusieurs sociétés, la clé d'une société ainsi coupée",
    ),
    (
        "docs/manual/fr/admin-manual.tex",
        "quand elle vient d'une adresse qu'une autre société servie admet",
    ),
    (
        "docs/manual/fr/admin-manual.tex",
        "admise par une autre société servie --- reçoit le 401",
    ),
];

/// Nombre **exact** d'occurrences de [`MULTI_SOCIETE`] dans la documentation
/// utilisateur, toutes assumées (recompté le 2026-10-09 en revue de code P1 :
/// 7 de la forme d'origine + 9 du motif élargi ; + 1 le 2026-10-13, Story 28-3 :
/// le nom de table du remède de moteur ; + 1 le 2026-10-26, Story 32-6c : le
/// journal du filtre d'adresse ; + 3 le 2026-10-27, Story 32-7 : le second volet
/// de P7-H3 et le 401 d'une origine admise par une autre société, § « Le serveur
/// MCP »). Un motif qui cesserait de voir
/// un site rougit (A-9, B10 : `>=` laissait passer une occurrence perdue).
const MULTI_SOCIETE_TOTAL: usize = 21;

/// **G18** (#127) — une installation porte une société : la documentation
/// utilisateur ne promet pas le contraire. Chaque occurrence de
/// [`MULTI_SOCIETE`] (texte normalisé, [`normaliser`]) tombe dans un fragment
/// de [`MULTI_SOCIETE_ASSUME`] ; chaque fragment est encore trouvé (exemption
/// morte → rouge) ; la phrase positive « une instance par dossier » figure au
/// manuel d'administration.
#[test]
fn une_installation_une_societe() {
    let motifs: Vec<Regex> = MULTI_SOCIETE
        .iter()
        .map(|m| Regex::new(m).unwrap())
        .collect();
    let mut erreurs = Vec::new();
    let mut total = 0;
    let mut admin = String::new();
    for (nom, brut) in documentation_utilisateur() {
        let texte = normaliser(&manuel::sans_commentaires(&brut));
        if nom == "docs/manual/fr/admin-manual.tex" {
            admin = texte.clone();
        }
        let fragments: Vec<&str> = MULTI_SOCIETE_ASSUME
            .iter()
            .filter(|(f, _)| *f == nom)
            .map(|(_, frag)| *frag)
            .collect();
        let mut couverts: Vec<(usize, usize)> = Vec::new();
        for frag in &fragments {
            let n = texte.matches(frag).count();
            if n != 1 {
                erreurs.push(format!(
                    "{nom} : fragment assumé trouvé {n} fois (1 attendue) : « {frag} »"
                ));
            }
            couverts.extend(texte.match_indices(frag).map(|(i, f)| (i, i + f.len())));
        }
        for motif in &motifs {
            for m in motif.find_iter(&texte) {
                total += 1;
                if !couverts
                    .iter()
                    .any(|(d, f)| m.start() >= *d && m.end() <= *f)
                {
                    let d = texte[..m.start()]
                        .char_indices()
                        .rev()
                        .nth(50)
                        .map_or(0, |(i, _)| i);
                    let f = (m.end() + 50).min(texte.len());
                    let f = (f..=texte.len())
                        .find(|i| texte.is_char_boundary(*i))
                        .unwrap();
                    erreurs.push(format!("{nom} : « {} » — …{}…", m.as_str(), &texte[d..f]));
                }
            }
        }
    }
    // Anti-test-muet : les motifs voient exactement les occurrences assumées.
    if total != MULTI_SOCIETE_TOTAL {
        erreurs.push(format!(
            "{total} occurrence(s) du motif, {MULTI_SOCIETE_TOTAL} attendues : motif muet ou site neuf"
        ));
    }
    if !admin.contains("une instance par dossier") {
        erreurs
            .push("admin-manual.tex : phrase positive « une instance par dossier » absente".into());
    }
    assert!(
        erreurs.is_empty(),
        "G18 — {} écart(s) :\n  - {}",
        erreurs.len(),
        erreurs.join("\n  - ")
    );
}

/// **G18-bis** (#127, revue de code P1 A-6) — le manuel utilisateur dit comment
/// on entre dans Kesh : aucune invitation par e-mail (il n'en existe pas — le
/// compte est créé par l'administrateur, `CreateUserRequest`), aucune connexion
/// par e-mail (on se connecte par son identifiant, `LoginRequest`), aucun
/// renvoi à un écran de changement de mot de passe en session (il n'en existe
/// pas). Sur le texte normalisé, commentaires retirés.
#[test]
fn le_manuel_utilisateur_dit_comment_on_entre() {
    let texte = normaliser(&manuel::sans_commentaires(&lire(
        "docs/manual/fr/user-manual.tex",
    )));
    let mut erreurs = Vec::new();
    for interdit in [
        r"(?i)invitation par (?:e-?mail|courriel)",
        r"(?i)saisir votre (?:adresse )?(?:e-?mail|courriel|adresse électronique) et votre mot de passe",
        r"(?i)passez par votre compte une fois connecté",
    ] {
        if let Some(m) = Regex::new(interdit).unwrap().find(&texte) {
            erreurs.push(format!("« {} »", m.as_str()));
        }
    }
    for exige in [
        "Saisir votre identifiant (nom d'utilisateur) et votre mot de passe",
        "L'administrateur de l'installation crée votre compte",
    ] {
        if !texte.contains(exige) {
            erreurs.push(format!("phrase absente : « {exige} »"));
        }
    }
    assert!(
        erreurs.is_empty(),
        "G18-bis — {} écart(s) :\n  - {}",
        erreurs.len(),
        erreurs.join("\n  - ")
    );
}

/// Les textes d'écran et commentaires du catalogue fr-CH et du panneau de
/// sauvegarde que le motif [`MULTI_SOCIETE`] attrape, **assumés** (revue de code P2,
/// A2-5 = E2-5 ; C-15-14-70) : la sauvegarde d'une installation contient
/// « toutes les sociétés » de l'installation — vrai, il y en a une —, et la
/// réécrire toucherait le code de production (quatre catalogues et un repli)
/// pour une nuance. Les locales de-CH (« alle Firmen »), it-CH (« tutte le
/// società ») et en-CH (« all companies ») portent la même clé traduite ; elles
/// ne sont pas lues ici (motif français), et suivent la valeur fr-CH par la garde
/// G9 de la 15-14a pour le repli.
const ECRANS_ASSUMES: &[(&str, &str)] = &[
    // Nom de colonne, dans un commentaire du catalogue.
    (
        "crates/kesh-i18n/locales/fr-CH/messages.ftl",
        "locale = companies.accounting_language",
    ),
    // admin-backup-page-description : le contenu d'une sauvegarde d'installation
    // (complétée des pièces justificatives par la Story 31-1b, AC22).
    (
        "crates/kesh-i18n/locales/fr-CH/messages.ftl",
        "(toutes les sociétés, les utilisateurs, les données système et les pièces justificatives :",
    ),
    // Commentaire de tête du composant : même contenu.
    (
        "frontend/src/lib/features/admin-backup/AdminBackupPanel.svelte",
        "(toutes sociétés + utilisateurs + système)",
    ),
    // Repli de admin-backup-page-description (égal à la valeur fr-CH).
    (
        "frontend/src/lib/features/admin-backup/AdminBackupPanel.svelte",
        "(toutes les sociétés, les utilisateurs, les données système et les pièces justificatives :",
    ),
];

/// **G18-ter** (#127, revue de code P2 A2-5) — dans le catalogue fr-CH et le
/// panneau de sauvegarde, chaque occurrence du motif [`MULTI_SOCIETE`] tombe dans
/// un fragment de [`ECRANS_ASSUMES`], et chaque fragment est trouvé une et une
/// seule fois : une promesse multi-société ajoutée à l'écran rougit, une
/// exemption morte aussi.
#[test]
fn les_ecrans_multi_societe_sont_assumes() {
    let motifs: Vec<Regex> = MULTI_SOCIETE
        .iter()
        .map(|m| Regex::new(m).unwrap())
        .collect();
    let mut erreurs = Vec::new();
    let blancs = Regex::new(r"\s+").unwrap();
    let mut fichiers: Vec<&str> = ECRANS_ASSUMES.iter().map(|(f, _)| *f).collect();
    fichiers.dedup();
    for nom in fichiers {
        let texte = blancs.replace_all(&lire(nom), " ").into_owned();
        assert!(texte.len() > 1000, "{nom} lu à vide");
        let mut couverts = Vec::new();
        for (_, frag) in ECRANS_ASSUMES.iter().filter(|(f, _)| *f == nom) {
            let n = texte.matches(frag).count();
            if n != 1 {
                erreurs.push(format!(
                    "{nom} : fragment assumé trouvé {n} fois (1 attendue) : « {frag} »"
                ));
            }
            couverts.extend(texte.match_indices(frag).map(|(i, f)| (i, i + f.len())));
        }
        for motif in &motifs {
            for m in motif.find_iter(&texte) {
                if !couverts
                    .iter()
                    .any(|(d, f)| m.start() >= *d && m.end() <= *f)
                {
                    erreurs.push(format!(
                        "{nom} : « {} » hors des fragments assumés",
                        m.as_str()
                    ));
                }
            }
        }
    }
    assert!(
        erreurs.is_empty(),
        "G18-ter — {} écart(s) :\n  - {}",
        erreurs.len(),
        erreurs.join("\n  - ")
    );
}

/// Version publiée que portent les manuels : argument de `\providecommand{\keshVersion}`
/// dans `docs/manual/shared/kesh-style.sty` (point 4-bis du `CLAUDE.md`).
fn version_des_manuels() -> String {
    let sty = lire("docs/manual/shared/kesh-style.sty");
    let re = Regex::new(r"\\providecommand\{\\keshVersion\}\{([^}]+)\}").unwrap();
    let version = re
        .captures(&sty)
        .unwrap_or_else(|| panic!("\\keshVersion introuvable dans kesh-style.sty"))[1]
        .to_string();
    // Positif : une version X.Y.Z, non un gabarit.
    assert!(
        Regex::new(r"^\d+\.\d+\.\d+$").unwrap().is_match(&version),
        "\\keshVersion n'est pas une version X.Y.Z : {version:?}"
    );
    version
}

/// **G19** (release 0.13.0) — une URL `raw.githubusercontent.com/guycorbaz/kesh/<réf>/…`
/// que le lecteur recopie (compose, `.env.example`, scripts Synology) désigne le **tag de
/// la version publiée**, `v<\keshVersion>`, jamais `main` ni une autre réf.
///
/// Pourquoi : `main` peut être en avance sur l'image publiée — un compose ou un gabarit
/// `.env` de `main` s'emploierait alors avec une image qui ne le comprend pas. Les URL
/// vivent dans des `lstlisting`, où la macro `\keshVersion` ne s'étend pas : la version
/// y est écrite en dur, et ce test la tient égale à la macro. Au bump de `\keshVersion`
/// (point 4-bis), il rougit tant que les URL n'ont pas suivi.
#[test]
fn les_url_raw_du_depot_pointent_sur_la_version_publiee() {
    let attendue = format!("v{}", version_des_manuels());
    let re = Regex::new(r"raw\.githubusercontent\.com/guycorbaz/kesh/([^/\s\x22'`)]+)").unwrap();
    let mut sources = fichiers_sous("docs/manual", &|n| n.ends_with(".tex"));
    sources.push(racine().join("README.md"));
    sources.extend(fichiers_sous("website", &|n| {
        n.ends_with(".html") || n.ends_with(".md")
    }));
    let mut vues = 0;
    let mut erreurs = Vec::new();
    for chemin in &sources {
        let texte = std::fs::read_to_string(chemin)
            .unwrap_or_else(|e| panic!("lecture de {}: {e}", chemin.display()));
        for (n, ligne) in texte.lines().enumerate() {
            for c in re.captures_iter(ligne) {
                vues += 1;
                if c[1] != attendue {
                    erreurs.push(format!(
                        "{}:{} — réf « {} », attendu « {attendue} »",
                        chemin.display(),
                        n + 1,
                        &c[1]
                    ));
                }
            }
        }
    }
    // Positif : les quatre URL du manuel d'administration (compose, `.env.example`,
    // et les deux `DEPOT=` de Synology) ont bien été lues.
    assert!(vues >= 4, "URL raw du dépôt lues : {vues}");
    assert!(
        erreurs.is_empty(),
        "G19 — {} URL hors du tag {attendue} :\n  - {}",
        erreurs.len(),
        erreurs.join("\n  - ")
    );
}

/// **G20** (#616, #617, #619 — Story 26-3) — le manuel ne fait plus chercher une case
/// « Compte de banque » au plan comptable ni ne promet le CAMT.053 de « toutes les
/// banques suisses », et l'écran d'import dit « CAMT.053 ou CSV ».
///
/// **Négatives ciblées sur la forme du défaut, sensibles à la casse**, sur la source
/// `.tex` **brute** des trois manuels français : « son propre compte de banque » est
/// juste (six sites des manuels, six messages `fr-CH`) et ne doit pas rougir — une
/// garde insensible à la casse frapperait du texte correct et serait « réparée » en
/// devenant muette. `[\s~]+` entre les mots : ni un retour à la ligne de source ni
/// l'espace insécable LaTeX `~` ne contournent la garde (`\s` couvre U+00A0 et
/// U+202F). La forme `[Tt]outes les banques suisses` interdit aussi « toutes les
/// banques suisses ne le proposent pas » : le manuel écrit « la plupart … pas
/// toutes ».
///
/// **Catalogues** : dans chacun des quatre, `bank-import-labels-drop-zone` contient
/// `CSV` et `bank-import-labels-page-title` ne contient pas `CAMT.053` ; l'écran lit
/// aussi ces deux clés pour son `aria-label` et son `<title>`.
///
/// **Positives adossées** : `propre compte de banque` et `Compte comptable lié`
/// (la phrase « comment un compte devient compte bancaire ») au manuel utilisateur ;
/// `SAUVEGARDE_RESEAU=proxy` et `SAUVEGARDE_SERVICE=` (listings du compose adapté) au
/// manuel d'administration — la prose `SAUVEGARDE_SERVICE (kesh-api)` est tenue par
/// G16 (g) ; chaque catalogue lu porte plus de 1000 clés.
///
/// **Angles morts écrits** : `\textit{…}`, `\keshpath{…}`, `\enquote{…}` autour de
/// « Compte de banque » (aucun employé par les manuels) ; les replis Svelte des deux
/// clés (non lus ici, égaux à la valeur `fr-CH` au 2026-10-11) ; le PDF (contrôles
/// `occ` journalisés au Dev Agent Record de la story).
#[test]
fn le_manuel_et_l_ecran_d_import_disent_vrai() {
    let negatives: [(&str, &str); 3] = [
        (
            r"\\(?:textbf|emph)\{Compte[\s~]+de[\s~]+banque\}",
            "case « Compte de banque » du plan comptable, inexistante (#616)",
        ),
        (
            r"«[\s~]*Compte[\s~]+de[\s~]+banque[\s~]*»",
            "case « Compte de banque » du plan comptable, inexistante (#616)",
        ),
        (
            r"[Tt]outes[\s~]+les[\s~]+banques[\s~]+suisses",
            "le CAMT.053 présenté comme universel (#617)",
        ),
    ];
    let mut erreurs = Vec::new();
    for (nom, texte) in manuels_fr() {
        for (re, quoi) in &negatives {
            for m in Regex::new(re).unwrap().find_iter(&texte) {
                let ligne = texte[..m.start()].matches('\n').count() + 1;
                erreurs.push(format!("{nom}:{ligne} — {quoi} : « {} »", m.as_str()));
            }
        }
    }
    let positives: [(&str, &str); 4] = [
        (
            "docs/manual/fr/user-manual.tex",
            r"propre[\s~]+compte[\s~]+de[\s~]+banque",
        ),
        (
            "docs/manual/fr/user-manual.tex",
            r"Compte[\s~]+comptable[\s~]+lié",
        ),
        (
            "docs/manual/fr/admin-manual.tex",
            r"SAUVEGARDE_RESEAU=proxy",
        ),
        ("docs/manual/fr/admin-manual.tex", r"SAUVEGARDE_SERVICE="),
    ];
    for (fichier, re) in positives {
        if !Regex::new(re).unwrap().is_match(&lire(fichier)) {
            erreurs.push(format!(
                "{fichier} : « {re} » absent (lecture à vide, ou phrase retirée)"
            ));
        }
    }
    for locale in ["fr-CH", "de-CH", "en-CH", "it-CH"] {
        let c = catalogue(locale);
        assert!(
            c.len() > 1000,
            "catalogue {locale} : {} clés, lu à vide ?",
            c.len()
        );
        match c.get("bank-import-labels-drop-zone") {
            Some(v) if v.contains("CSV") => {}
            autre => erreurs.push(format!(
                "{locale} : bank-import-labels-drop-zone ne dit pas « CSV » ({autre:?}) (#617)"
            )),
        }
        match c.get("bank-import-labels-page-title") {
            Some(v) if !v.contains("CAMT.053") => {}
            autre => erreurs.push(format!(
                "{locale} : bank-import-labels-page-title dit « CAMT.053 » ou manque ({autre:?}) (#617)"
            )),
        }
    }
    assert!(
        erreurs.is_empty(),
        "G20 — {} écart(s) :\n  - {}",
        erreurs.len(),
        erreurs.join("\n  - ")
    );
}

/// Version de Rust lue dans un texte de configuration : la mineure `X.Y` et, s'il est
/// écrit, le patch `Z`.
#[derive(Debug, PartialEq, Eq)]
struct VersionRust {
    mineure: String,
    patch: Option<String>,
}

/// Découpe `X.Y` ou `X.Y.Z` — rien d'autre — en [`VersionRust`].
fn version_rust(texte: &str) -> Option<VersionRust> {
    static FORME: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^(\d+\.\d+)(?:\.(\d+))?$").unwrap());
    FORME.captures(texte).map(|c| VersionRust {
        mineure: c[1].to_string(),
        patch: c.get(2).map(|p| p.as_str().to_string()),
    })
}

/// Retient l'unique ligne de `texte` que reconnaît `motif` ; zéro ou plusieurs lignes
/// rendent un refus explicite, nommé par `quoi` et `fichier`.
fn ligne_unique<'a>(
    texte: &'a str,
    motif: &Regex,
    fichier: &str,
    quoi: &str,
    absent: &str,
) -> Result<&'a str, String> {
    let lignes: Vec<&str> = texte.lines().filter(|l| motif.is_match(l)).collect();
    match lignes.as_slice() {
        [une] => Ok(une),
        [] => Err(format!("{fichier} : {absent}")),
        plusieurs => Err(format!(
            "{fichier} : {} lignes {quoi}, une seule attendue",
            plusieurs.len()
        )),
    }
}

/// **Fonction pure de G21** (#697, Story 31-3) — extrait la version de Rust de l'étape
/// `FROM rust:` du `Dockerfile` et le `channel` de `rust-toolchain.toml`, et dit s'ils
/// s'accordent. Rend les deux versions lues (`Dockerfile`, puis toolchain) ou un message
/// qui nomme l'écart.
///
/// Règle (AC7) :
/// - **Dockerfile** : lignes `FROM rust:` non commentées, `FROM` sans égard à la casse
///   (une instruction Dockerfile l'est) ; exactement une. Le tag est `X.Y[.Z]`, suivi ou
///   non de `-<suffixe>` (`1.97-bookworm`, `1.97.1-bookworm`, `1.97`), puis éventuellement
///   de `AS <nom>`. Un tag sans mineure (`1-bookworm`, `bookworm`, `latest`) est refusé.
/// - **`rust-toolchain.toml`** : lignes `channel = …` non commentées ; exactement une ; la
///   valeur, entre guillemets doubles ou simples, est `X.Y` ou `X.Y.Z` (`stable`,
///   `nightly`, `beta` sont refusés).
/// - **Comparaison** : mineures égales ; patchs égaux si les deux en portent un — sinon le
///   patch n'est pas comparé, le `Dockerfile` suivant la convention du tag mineur
///   (C-31-3-2).
///
/// **Angles morts écrits** : une image tirée par `FROM --platform=… rust:…`, par une
/// variable (`FROM ${IMAGE}`) ou par un digest (`rust:1.97@sha256:…`) n'est pas
/// reconnue — la première et la deuxième rendent « aucune ligne », la troisième « sans
/// mineure » : un refus explicite, jamais un vert muet. Une table `[toolchain]` écrite en
/// ligne (`toolchain = { channel = … }`) rend « absent ».
fn rust_versions_accordees(
    dockerfile: &str,
    toolchain: &str,
) -> Result<(VersionRust, VersionRust), String> {
    static FROM_RUST: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)^\s*FROM\s+rust:").unwrap());
    static TAG: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)^\s*FROM\s+rust:(\S+)(?:\s+AS\s+\S+)?\s*$").unwrap());
    static VERSION_DU_TAG: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^(\d+\.\d+(?:\.\d+)?)(?:-[A-Za-z0-9._-]+)?$").unwrap());
    static CHANNEL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\s*channel\s*=").unwrap());
    static VALEUR_DU_CHANNEL: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"^\s*channel\s*=\s*(?:"([^"]*)"|'([^']*)')\s*(?:#.*)?$"#).unwrap()
    });

    // Dockerfile : une seule étape `FROM rust:`, d'un tag X.Y[.Z][-suffixe].
    let ligne = ligne_unique(
        dockerfile,
        &FROM_RUST,
        "Dockerfile",
        "`FROM rust:`",
        "aucune ligne `FROM rust:` non commentée",
    )?;
    let tag = TAG
        .captures(ligne)
        .map(|c| c[1].to_string())
        .ok_or_else(|| format!("Dockerfile : ligne `FROM rust:` illisible : « {ligne} »"))?;
    let docker = VERSION_DU_TAG
        .captures(&tag)
        .and_then(|c| version_rust(&c[1]))
        .ok_or_else(|| {
            format!("Dockerfile : tag `rust:{tag}` sans mineure X.Y (attendu `X.Y[.Z][-suffixe]`)")
        })?;

    // rust-toolchain.toml : un seul `channel`, numérique.
    let ligne = ligne_unique(
        toolchain,
        &CHANNEL,
        "rust-toolchain.toml",
        "`channel`",
        "`channel` absent",
    )?;
    let valeur = VALEUR_DU_CHANNEL
        .captures(ligne)
        .and_then(|c| c.get(1).or_else(|| c.get(2)))
        .map(|v| v.as_str().to_string())
        .ok_or_else(|| format!("rust-toolchain.toml : ligne `channel` illisible : « {ligne} »"))?;
    let chaine = version_rust(&valeur).ok_or_else(|| {
        format!(
            "rust-toolchain.toml : `channel = \"{valeur}\"` non numérique (attendu X.Y ou X.Y.Z)"
        )
    })?;

    if docker.mineure != chaine.mineure {
        return Err(format!(
            "désaccord de mineure : Dockerfile `rust:{tag}` contre rust-toolchain.toml `{valeur}`"
        ));
    }
    if let (Some(a), Some(b)) = (&docker.patch, &chaine.patch)
        && a != b
    {
        return Err(format!(
            "désaccord de patch : Dockerfile `rust:{tag}` contre rust-toolchain.toml `{valeur}`"
        ));
    }
    Ok((docker, chaine))
}

/// G21 — cas de la fonction pure (AC7) : accords et chaque branche de refus, chacune
/// reconnue à un fragment de son message (un refus pour une autre raison ne compte pas).
#[test]
fn g21_la_regle_d_accord_dockerfile_toolchain() {
    const TC_1971: &str = "[toolchain]\nchannel = \"1.97.1\"\n";
    // (Dockerfile, rust-toolchain.toml, Ok(()) ou fragment attendu du message d'erreur)
    let cas: [(&str, &str, Result<(), &str>); 16] = [
        ("FROM rust:1.97-bookworm\n", TC_1971, Ok(())),
        (
            "FROM rust:1.97.1-bookworm AS rust-builder\n",
            TC_1971,
            Ok(()),
        ),
        ("FROM rust:1.97-bookworm\n", "channel = \"1.97\"\n", Ok(())),
        ("FROM rust:1.97\n", TC_1971, Ok(())),
        ("from rust:1.97-bookworm as rust-builder\n", TC_1971, Ok(())),
        ("FROM rust:1.97-bookworm\n", "channel = '1.97.1'\n", Ok(())),
        (
            "FROM rust:1.96-bookworm AS rust-builder\n",
            TC_1971,
            Err("mineure"),
        ),
        ("FROM rust:1.97.0-bookworm\n", TC_1971, Err("patch")),
        ("FROM node:22-bookworm-slim\n", TC_1971, Err("aucune ligne")),
        (
            "# FROM rust:1.97-bookworm\nFROM debian:bookworm-slim\n",
            TC_1971,
            Err("aucune ligne"),
        ),
        (
            "FROM rust:1.97-bookworm AS a\nFROM rust:1.97-bookworm AS b\n",
            TC_1971,
            Err("2 lignes `FROM rust:`"),
        ),
        ("FROM rust:1-bookworm\n", TC_1971, Err("sans mineure")),
        ("FROM rust:latest\n", TC_1971, Err("sans mineure")),
        (
            "FROM rust:1.97-bookworm\n",
            "channel = \"stable\"\n",
            Err("non numérique"),
        ),
        (
            "FROM rust:1.97-bookworm\n",
            "[toolchain]\n# channel = \"1.97.1\"\n",
            Err("absent"),
        ),
        (
            "FROM rust:1.97-bookworm\n",
            "channel = \"1.97.1\"\nchannel = \"1.97.1\"\n",
            Err("2 lignes `channel`"),
        ),
    ];
    let mut erreurs = Vec::new();
    for (i, (dockerfile, toolchain, attendu)) in cas.iter().enumerate() {
        let rendu = rust_versions_accordees(dockerfile, toolchain);
        let juste = match (attendu, &rendu) {
            (Ok(()), Ok(_)) => true,
            (Err(fragment), Err(message)) => message.contains(fragment),
            _ => false,
        };
        if !juste {
            erreurs.push(format!(
                "cas {i} ({dockerfile:?} / {toolchain:?}) : attendu {attendu:?}, rendu {rendu:?}"
            ));
        }
    }
    assert!(
        erreurs.is_empty(),
        "G21 — {} cas faux :\n  - {}",
        erreurs.len(),
        erreurs.join("\n  - ")
    );
    // Positif : les versions rendues sont celles écrites, mineure et patch séparés.
    assert_eq!(
        rust_versions_accordees("FROM rust:1.97-bookworm\n", TC_1971),
        Ok((
            VersionRust {
                mineure: "1.97".into(),
                patch: None
            },
            VersionRust {
                mineure: "1.97".into(),
                patch: Some("1".into())
            },
        ))
    );
}

/// **G21** (#697, Story 31-3) — le `FROM rust:` du `Dockerfile` et le `channel` de
/// `rust-toolchain.toml` désignent la même chaîne d'outils.
///
/// Pourquoi : ce sont deux sources de vérité indépendantes. `rustup` lit
/// `rust-toolchain.toml` (CI, station) ; l'image part du `Dockerfile`. Un bump de l'un sans
/// l'autre ne fait rien rougir — l'image se bâtit sur une autre chaîne que la CI, ou
/// télécharge la bonne à chaque build. La règle est celle de [`rust_versions_accordees`].
///
/// **Positives adossées** : les deux fichiers sont lus et reconnus (`rust-builder`,
/// `[toolchain]`), et la fonction rend `Ok` avec une mineure extraite de chacun — un motif
/// qui ne trouverait rien rend un refus explicite, jamais un vert muet.
#[test]
fn le_dockerfile_et_rust_toolchain_designent_la_meme_chaine() {
    let dockerfile = lire("Dockerfile");
    let toolchain = lire("rust-toolchain.toml");
    assert!(dockerfile.contains("rust-builder"), "Dockerfile lu à vide");
    assert!(
        toolchain.contains("[toolchain]"),
        "rust-toolchain.toml lu à vide"
    );
    match rust_versions_accordees(&dockerfile, &toolchain) {
        Ok((docker, chaine)) => {
            assert!(
                !docker.mineure.is_empty() && docker.mineure == chaine.mineure,
                "G21 — versions extraites incohérentes : {docker:?} / {chaine:?}"
            );
        }
        Err(e) => panic!("G21 — Dockerfile et rust-toolchain.toml en désaccord : {e}"),
    }
}

/// **G22** (#503, revue de code P1 de la Story 31-1c, L-B4 et F-A2) — la sauvegarde
/// automatique dit vrai :
///
/// - le manuel d'administration dit qu'un échec est retenté **au bout d'une heure**,
///   sans la parenthèse « ou de l'intervalle, s'il est plus court » (aucune fréquence
///   admise n'est inférieure à une heure, C-31-1-91) ;
/// - il dit que le bandeau s'allume quand la dernière réussite date d'**au moins** deux
///   intervalles (`is_overdue` compare par `>=`), non de « plus de » deux ;
/// - le rappel permanent de l'écran (`admin-backup-auto-reminder`) dit le volume de
///   l'AC33 — le nombre conservé plus une dans `auto/`, plus une sauvegarde par import,
///   jamais purgée — dans les quatre langues (un marqueur par locale).
///
/// **Positives adossées** : la phrase de reprise et celle du bandeau sont trouvées ; la
/// clé existe dans chaque catalogue. **Angle mort écrit** : les marqueurs de/en/it
/// prouvent que la phrase du volume y est, pas la qualité de sa traduction ; **aucune**
/// garde ne tient le repli Svelte égal au fr-CH (angle mort écrit ; `i18n-un-repli-par-cle`
/// compare les sites entre eux, sur ses seuls préfixes ; #758). *(Corrigé par la Story
/// 32-7 : ce commentaire attribuait cette tenue à `i18n-un-repli-par-cle`, qui ne compare
/// pas au catalogue et dont les préfixes n'incluent pas `admin-backup-`.)*
#[test]
fn la_sauvegarde_automatique_dit_vrai() {
    let manuel = normaliser(&lire("docs/manual/fr/admin-manual.tex"));
    let mut erreurs = Vec::new();
    let negatives: [(&str, &str); 2] = [
        (
            r"retentée[^.]*plus court",
            "reprise « ou de l'intervalle, s'il est plus court » : cas inatteignable",
        ),
        (
            r"plus de deux intervalles",
            "« plus de deux intervalles » alors que le bandeau s'allume à l'égalité",
        ),
    ];
    for (re, quoi) in negatives {
        for m in Regex::new(re).unwrap().find_iter(&manuel) {
            erreurs.push(format!("admin-manual.tex — {quoi} : « {} »", m.as_str()));
        }
    }
    let positives = [
        r"retentée au bout d'une heure\.",
        r"au moins deux intervalles",
    ];
    for re in positives {
        if !Regex::new(re).unwrap().is_match(&manuel) {
            erreurs.push(format!("admin-manual.tex : « {re} » absent"));
        }
    }
    let marqueurs = [
        ("fr-CH", "jamais purgée"),
        ("de-CH", "nie gelöscht"),
        ("en-CH", "never purged"),
        ("it-CH", "mai eliminat"),
    ];
    for (locale, marqueur) in marqueurs {
        match catalogue(locale).get("admin-backup-auto-reminder") {
            Some(v) if v.contains(marqueur) => {}
            autre => erreurs.push(format!(
                "{locale} : admin-backup-auto-reminder ne dit pas « {marqueur} » ({autre:?})"
            )),
        }
    }
    assert!(
        erreurs.is_empty(),
        "G22 — {} écart(s) :\n  - {}",
        erreurs.len(),
        erreurs.join("\n  - ")
    );
}

/// **Fonction pure de G23** — le texte entre `% <nom>:debut` et `% <nom>:fin` d'un source
/// LaTeX **brut** (jamais normalisé : `normaliser` efface les sauts de ligne, et les
/// marqueurs avec eux). Erreur si un marqueur manque : sans elle, un bloc supprimé ferait
/// passer G23 (a) et (b) à vide.
fn bloc_marque<'a>(source: &'a str, nom: &str) -> Result<&'a str, String> {
    let debut = format!("% {nom}:debut");
    let fin = format!("% {nom}:fin");
    let i = source
        .find(&debut)
        .ok_or(format!("bloc absent : {debut}"))?
        + debut.len();
    let j = source[i..]
        .find(&fin)
        .ok_or(format!("bloc non fermé : {fin}"))?;
    Ok(&source[i..i + j])
}

/// Le contenu de chaque `\texttt{…}` d'un texte, `\_` rendu `_`.
fn textes_texttt(texte: &str) -> Vec<String> {
    Regex::new(r"\\texttt\{([^}]*)\}")
        .unwrap()
        .captures_iter(texte)
        .map(|c| c[1].replace("\\_", "_"))
        .collect()
}

/// **Fonction pure de G23 (b)** — le PREMIER `\texttt{…}` de chaque rangée d'un tableau,
/// rangées découpées sur `\\` (non sur les sauts de ligne du source : une rangée peut
/// s'étendre sur plusieurs lignes) ; une rangée sans `\texttt` n'ajoute rien (en-tête).
fn premiers_codes_des_rangees(bloc: &str) -> Vec<String> {
    bloc.split("\\\\")
        .filter_map(|rangee| textes_texttt(rangee).into_iter().next())
        .collect()
}

/// Le texte de la sous-section de `label` : de `\label{label}` à la prochaine
/// `\subsection` ou `\section`.
fn sous_section<'a>(source: &'a str, label: &str) -> Result<&'a str, String> {
    let marque = format!("\\label{{{label}}}");
    let i = source.find(&marque).ok_or(format!("{marque} absent"))? + marque.len();
    let reste = &source[i..];
    let j = Regex::new(r"\\(sub)?section\{")
        .unwrap()
        .find(reste)
        .map_or(reste.len(), |m| m.start());
    Ok(&reste[..j])
}

/// G23 — témoins des fonctions pures : un bloc absent ou non fermé est une erreur ; une
/// rangée qui cite une seconde portée n'ajoute que la première ; une rangée écrite sur
/// deux lignes de source, dont la seconde commence par une portée, n'en ajoute pas.
#[test]
fn g23_les_fonctions_pures() {
    assert!(bloc_marque("rien", "mcp-portees").is_err());
    assert!(bloc_marque("% mcp-portees:debut\nx", "mcp-portees").is_err());
    let bloc = bloc_marque(
        "a\n% mcp-portees:debut\n\\texttt{books:read} & x \\\\\n\\texttt{invoices:read} & sans \\texttt{contacts:read} \\\\\n\\texttt{documents:read} & sur deux\n\\texttt{contacts:read} lignes \\\\\n% mcp-portees:fin\n",
        "mcp-portees",
    )
    .unwrap();
    assert_eq!(
        premiers_codes_des_rangees(bloc),
        ["books:read", "invoices:read", "documents:read"]
    );
    assert_eq!(
        premiers_codes_des_rangees("Portée & Ce qu'elle ouvre \\\\ \\hline \\texttt{a} \\\\"),
        ["a"]
    );
    assert_eq!(textes_texttt("\\texttt{mcp\\_server}"), ["mcp_server"]);
    assert_eq!(
        sous_section("\\subsection{A}\\label{x} corps \\subsection{B}", "x").unwrap(),
        " corps "
    );
}

/// **G23** (#723, Story 32-7) — le serveur MCP dit vrai :
///
/// - (a) le bloc `% mcp-reseaux-defaut` du manuel d'administration liste **exactement**
///   `DEFAULT_ALLOWED_NETWORKS` (ensemble égal dans les deux sens), et `100.64.0.0/10`
///   paraît dans `sec:mcp-filtre` **hors** de ce bloc ;
/// - (b) le bloc `% mcp-portees` nomme exactement les cinq `McpScope::ALL`, dans cet
///   ordre — le premier `\texttt` de chaque rangée ;
/// - (c) dans les quatre catalogues, `mcp-server-guarantee-tailscale` contient
///   `100.64.0.0/10`, et `mcp-server-guarantee-proxy` contient `Traefik` et `Docker`.
///
/// Les blocs s'extraient du source **brut** (`lire`), jamais après `normaliser`.
/// **Angle mort écrit** : les phrases de portée du manuel et de l'écran sont tenues vraies
/// par la relecture du code, non par ce test (seul l'ordre des codes l'est).
#[test]
fn le_serveur_mcp_dit_vrai() {
    use kesh_db::entities::mcp_key::McpScope;
    use kesh_db::entities::mcp_server_settings::DEFAULT_ALLOWED_NETWORKS;

    let manuel = lire("docs/manual/fr/admin-manual.tex");
    let mut erreurs = Vec::new();

    match bloc_marque(&manuel, "mcp-reseaux-defaut") {
        Ok(bloc) => {
            let lus: std::collections::BTreeSet<String> = textes_texttt(bloc).into_iter().collect();
            let attendus: std::collections::BTreeSet<String> = DEFAULT_ALLOWED_NETWORKS
                .iter()
                .map(|s| s.to_string())
                .collect();
            if lus != attendus {
                erreurs.push(format!("(a) défauts du manuel {lus:?} ≠ code {attendus:?}"));
            }
            if bloc.contains("100.64.0.0/10") {
                erreurs.push("(a) 100.64.0.0/10 dans le bloc des défauts".to_string());
            }
        }
        Err(e) => erreurs.push(format!("(a) {e}")),
    }
    match sous_section(&manuel, "sec:mcp-filtre") {
        Ok(texte) => {
            let hors_bloc = match (
                texte.find("% mcp-reseaux-defaut:debut"),
                texte.find("% mcp-reseaux-defaut:fin"),
            ) {
                (Some(i), Some(j)) => format!("{}{}", &texte[..i], &texte[j..]),
                _ => texte.to_string(),
            };
            if !hors_bloc.contains("100.64.0.0/10") {
                erreurs.push("(a) 100.64.0.0/10 absent de sec:mcp-filtre hors du bloc".to_string());
            }
        }
        Err(e) => erreurs.push(format!("(a) {e}")),
    }

    match bloc_marque(&manuel, "mcp-portees") {
        Ok(bloc) => {
            let lus = premiers_codes_des_rangees(bloc);
            let attendus: Vec<String> = McpScope::ALL
                .iter()
                .map(|s| s.as_str().to_string())
                .collect();
            if lus != attendus {
                erreurs.push(format!(
                    "(b) portées du manuel {lus:?} ≠ McpScope::ALL {attendus:?}"
                ));
            }
        }
        Err(e) => erreurs.push(format!("(b) {e}")),
    }

    for locale in ["fr-CH", "de-CH", "it-CH", "en-CH"] {
        let cat = catalogue(locale);
        match cat.get("mcp-server-guarantee-tailscale") {
            Some(v) if v.contains("100.64.0.0/10") => {}
            autre => erreurs.push(format!(
                "(c) {locale} : mcp-server-guarantee-tailscale sans 100.64.0.0/10 ({autre:?})"
            )),
        }
        match cat.get("mcp-server-guarantee-proxy") {
            Some(v) if v.contains("Traefik") && v.contains("Docker") => {}
            autre => erreurs.push(format!(
                "(c) {locale} : mcp-server-guarantee-proxy sans Traefik et Docker ({autre:?})"
            )),
        }
    }

    assert!(
        erreurs.is_empty(),
        "G23 — {} écart(s) :\n  - {}",
        erreurs.len(),
        erreurs.join("\n  - ")
    );
}
