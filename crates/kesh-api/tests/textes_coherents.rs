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
//! | G4-bis | #488 | tout numéro de compte cité en exemple existe dans un plan livré, sous son nom |
//! | G5 | #458 | aucun « dossier surveillé » : l'import de factures se lance à la main |
//! | G6 | #449 | la faille KF-036 n'est plus annoncée ouverte : corrigée depuis la v0.10.0 |
//! | G7 | #432 | toute référence d'issue du README est un lien vers la même issue |
//! | G9 | #569, #547 | les replis Rust égalent la valeur fr-CH de leur clé |
//! | G12 | #569 | le manuel et le guide d'API disent l'ordre de réouverture |

use std::collections::HashMap;
use std::path::PathBuf;

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
/// un blanc prolonge la valeur de la clé précédente (jointe par une espace, comme
/// `valeurs_brutes` de `kesh-i18n/src/loader.rs` et `valeurDuCatalogueFr` du Vitest
/// G13 — revue de code P1, B-4). Sans cela, une clé passée sur plusieurs lignes serait
/// lue tronquée à sa première ligne, et G9 rougirait sans régression.
///
/// ⚠️ Trois analyseurs du même format, dans trois crates ou langages qui ne
/// partagent pas de code de test : ils sont tenus **identiques** par la règle
/// ci-dessus, et le test `le_catalogue_fr_joint_les_continuations` l'exerce sur une
/// clé réelle du catalogue.
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
                v.push(' ');
                v.push_str(ligne.trim());
            }
        } else {
            // Commentaire, ligne vide : la valeur en cours est close.
            courante = None;
        }
    }
    assert!(out.len() > 100, "catalogue fr-CH lu à vide");
    out
}

/// Anti-test-muet de [`catalogue_fr`] : une clé réelle sur plusieurs lignes
/// (`email-password-reset-body`) est lue entière, et la clé suivante n'en hérite rien.
#[test]
fn le_catalogue_fr_joint_les_continuations() {
    let fr = catalogue_fr();
    let corps = &fr["email-password-reset-body"];
    assert!(
        corps.contains("mot de passe Kesh.") && corps.contains("ignorez cet email"),
        "continuations non jointes : {corps}"
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

/// **G4-bis** (#488, revue de code P1, E-1/E-5) — tout numéro de compte que le
/// manuel utilisateur ou le guide de démarrage cite **existe dans un des trois plans
/// livrés**, et, quand il est nommé, **sous le nom que ce plan lui donne**.
///
/// ⚠️ Pourquoi un inventaire, et non une liste de numéros interdits : l'exemple
/// « Association » faisait créer 3600/3601, absents des trois plans, alors que le plan
/// association porte 3000 « Cotisations des membres » et 3100 « Dons reçus » ; la
/// relecture a trouvé six autres numéros ou noms faux (1030, 4200 « Charges de
/// personnel », 6500 « Entretien », 6997/7997, 3200 « Honoraires »). Interdire ces
/// formes-là laisserait passer la suivante ; la garde inventorie donc **tous** les
/// nombres de quatre chiffres isolés et exige que chacun soit un compte livré.
///
/// - Un sous-compte (`3200.1`) est un compte **à créer** : seul son parent doit
///   exister — le texte qui le cite doit le dire à créer, ce que la garde ne lit pas.
/// - Les nombres de 2001 à 2099 sont des années (aucun plan livré n'y a de compte,
///   ce que la garde vérifie) ; 1000, montant ou borne (« de 1 à 1000 »), est aussi
///   le compte Caisse et passe donc — faux vert accepté, il ne cache aucun compte faux.
/// - Formes nommées contrôlées : `NNNN « Nom »`, `\texttt{NNNN Nom}`,
///   `NNNN \emph{Nom}`, `` `NNNN Nom` `` et les listes entre parenthèses
///   `(NNNN Nom, NNNN Nom, etc.)`. Un nom écrit autrement (« Crédit 3000 Ventes … »)
///   n'est contrôlé que par son numéro : angle mort déclaré.
#[test]
fn les_comptes_cites_en_exemple_existent_dans_les_plans_livres() {
    use kesh_core::chart_of_accounts::load_chart;
    let mut noms: HashMap<String, Vec<String>> = HashMap::new();
    for org in ["Pme", "Independant", "Association"] {
        for e in load_chart(org).expect("plan livré") {
            let nom = e.name.get("fr").cloned().unwrap_or_default();
            noms.entry(e.number).or_default().push(nom);
        }
    }
    // Positifs : les plans sont lus, et le trou des années est réel.
    assert!(noms.contains_key("6940") && noms.contains_key("3100"));
    assert!(
        !noms
            .keys()
            .any(|n| n.len() == 4 && ("2001".."2100").contains(&n.as_str())),
        "un plan livré a un compte entre 2001 et 2099 : la garde le prendrait pour une année"
    );

    let nombre =
        Regex::new(r"(?:^|[^\d.,'’\-/:#{_A-Za-z])([1-9]\d{3})(\.\d+)?(?:$|[^\d'’%A-Za-z_}])")
            .unwrap();
    let nommes = [
        Regex::new(r"\b([1-9]\d{3}) « ([^»]+?) »").unwrap(),
        Regex::new(r"\\texttt\{([1-9]\d{3}) ([^}]+)\}").unwrap(),
        Regex::new(r"\b([1-9]\d{3}) \\emph\{([^}]+)\}").unwrap(),
        Regex::new(r"`([1-9]\d{3}) ([^`]+)`").unwrap(),
    ];
    let parentheses = Regex::new(r"\(([^()]*)\)").unwrap();
    let element = Regex::new(r"^([1-9]\d{3}) ([A-ZÉ][^«»]*)$").unwrap();

    let mut vus = 0;
    let mut nommes_vus = 0;
    for nom_fichier in [
        "docs/manual/fr/user-manual.tex",
        "docs/user-guide/fr/getting-started.md",
    ] {
        let texte = lire(nom_fichier);
        for ligne in texte.lines() {
            for c in nombre.captures_iter(ligne) {
                let n = &c[1];
                if ("2001".."2100").contains(&n) {
                    continue;
                }
                assert!(
                    noms.contains_key(n),
                    "{nom_fichier} : compte {n}{} absent des trois plans livrés : {ligne}",
                    c.get(2).map_or("", |m| m.as_str())
                );
                vus += 1;
            }
            let mut controler = |n: &str, nom: &str| {
                let livres = noms
                    .get(n)
                    .unwrap_or_else(|| panic!("{nom_fichier} : compte {n} absent : {ligne}"));
                assert!(
                    livres.iter().any(|l| l == nom.trim()),
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
                        controler(&c[1], &c[2]);
                    }
                }
            }
        }
    }
    assert!(vus > 30, "numéros de compte lus : {vus}");
    assert!(nommes_vus > 20, "comptes nommés lus : {nommes_vus}");
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
/// Table fermée (fichier → clés, nombre de sites). La comparaison se fait après
/// normalisation des continuations de chaîne Rust (`\` en fin de ligne, saut de
/// ligne et blancs de tête retirés) : un repli écrit sur plusieurs lignes est juste.
#[test]
fn les_replis_rust_suivent_le_catalogue() {
    const TABLE: [(&str, &[(&str, usize)]); 3] = [
        (
            "crates/kesh-api/src/errors.rs",
            &[
                ("settlement-cancel-blocked-fiscal-year-closed", 1),
                ("reconciliation-cancel-blocked-fiscal-year-closed", 1),
                ("supplier-invoices-cancel-blocked-fiscal-year-closed", 1),
                ("error-invoice-pdf-header-overflow", 1),
            ],
        ),
        (
            "crates/kesh-api/src/routes/fiscal_years.rs",
            &[("error-fiscal-year-reopen-blocked", 1)],
        ),
        (
            "crates/kesh-api/src/routes/opening_balances.rs",
            &[("error-opening-balances-first-year-closed", 2)],
        ),
    ];
    let continuation = Regex::new(r"\\\n\s*").unwrap();
    let fr = catalogue_fr();
    for (fichier, cles) in TABLE {
        let source = continuation.replace_all(&lire(fichier), "").into_owned();
        for (cle, sites) in cles {
            let valeur = fr
                .get(*cle)
                .unwrap_or_else(|| panic!("fr-CH : {cle} absente"));
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
