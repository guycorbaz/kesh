//! Garde des libellés du journal d'audit — Story 25-1c-a (AC 16, 18).
//!
//! # Ce qu'elle établit
//!
//! ✅ Que les deux listes de `kesh_api::audit_labels` sont **exactement** ce que
//! le code de production écrit dans `audit_log.action` et `.entity_type`, et que
//! **chaque** code a son libellé dans les **quatre** catalogues.
//!
//! # Pourquoi un diff ensembliste, et bilatéral
//!
//! ⚠️ Un compteur ne verrait pas un code retiré pendant qu'un autre est ajouté.
//! Et le diff se fait **dans les deux sens** : le sens « code → liste » attrape
//! l'action neuve qu'on oublierait de libeller ; le sens « liste → code »
//! attrape la ligne morte — et, accessoirement, une coupe `#[cfg(test)]` devenue
//! trop gourmande, qui priverait l'extraction d'un site réel.
//!
//! # L'inventaire des sites NON RÉSOLUS (D4-ter du `CLAUDE.md`)
//!
//! ⛔ L'extracteur ci-dessous lit un **littéral à une position d'argument**.
//! Sept sites du dépôt n'en portent pas : l'action y est une variable ou un
//! ternaire. Énumérer « les formes qui marchent » laisserait une huitième forme
//! passer sans que rien ne rougisse. On inventorie donc l'**ensemble clos de ce
//! qui ne résout pas** : tout site indirect doit figurer dans
//! `SITES_INDIRECTS`, avec les codes qu'il produit, écrits à la main et relus.
//! Un site indirect nouveau fait rougir cette garde même si son code est déjà
//! libellé — c'est voulu : personne ne l'aurait relu.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use kesh_api::audit_labels::{
    ACTIONS, ACTOR_TYPES, ENTITY_TYPES, PREFIX_ACTION, PREFIX_ACTOR_TYPE, PREFIX_ENTITY,
    message_key,
};

/// Les formes de construction d'une entrée d'audit, et la **position** de leurs
/// arguments — `(motif, index de l'action, index du type d'entité)`.
///
/// ⚠️ Les positions diffèrent d'un constructeur à l'autre : `user` et
/// `from_current_user` prennent l'auteur en premier, `for_actor` et `api_key` en
/// prennent **deux**. Un extracteur qui chercherait « le premier littéral »
/// rendrait le nom d'utilisateur pour une action.
const FORMES: &[(&str, usize, Option<usize>)] = &[
    ("NewAuditLogEntry::user(", 1, Some(2)),
    ("NewAuditLogEntry::from_current_user(", 1, Some(2)),
    ("NewAuditLogEntry::for_actor(", 2, Some(3)),
    ("NewAuditLogEntry::api_key(", 2, Some(3)),
    // Helper local des exercices comptables : il fixe `"fiscal_year"` en dur,
    // d'où l'absence de position pour le type.
    ("build_audit_entry(", 1, None),
];

/// Le type d'entité que `fiscal_years::build_audit_entry` écrit en dur.
const TYPE_DU_HELPER_EXERCICES: &str = "fiscal_year";

// ⚠️ **Il n'y a PAS de liste de fichiers exclus, et c'est une correction.**
//
// Une première rédaction sautait `crates/kesh-api/src/audit.rs` au motif que
// c'est un passe-plat — il relaie l'action de ses appelants, extraits par
// ailleurs. L'argument était juste sur le constat et **faux sur la conclusion** :
// l'inventaire n'est pas une liste de coupables, c'est **l'ensemble clos de ce
// que l'extracteur ne résout pas**. Exclure le FICHIER échangeait une ligne de
// bruit contre un TROU : un littéral d'action ajouté demain dans `audit.rs`
// n'entrait dans aucun ensemble, aucune assertion ne bougeait, et le code
// s'affichait brut à l'écran comme dans le CSV.
//
// `audit.rs` est donc inventorié comme site indirect à codes vides — ce qui dit
// exactement la bonne chose.
//
// ⚠️ **Et c'est le RETRAIT de l'exclusion qui protège, non l'entrée
// d'inventaire.** Une première rédaction de ce commentaire attribuait la
// protection au « volet (a) » : c'est faux, ce volet ne voit que les sites dont
// l'argument n'est PAS littéral. Un littéral ajouté dans `audit.rs` est attrapé
// par le **diff des actions**, parce que le fichier est de nouveau balayé.
// L'entrée à codes vides, elle, n'apporte pas cette protection-là — elle
// empêche `inconnus` de rougir sur un fichier désormais lu. *Se tromper
// là-dessus conduirait à réintroduire une exclusion de fichier en croyant bien
// faire.*
//
// ⚠️ **Elle apporte en revanche une SECONDE ligne de défense**, depuis que le
// contrôle de l'inventaire est bilatéral : tout littéral en forme de code
// trouvé dans `audit.rs` devra être dans `ACTIONS`, sa liste déclarée étant
// vide. Le premier état de ce commentaire ne le disait pas.

/// **L'ensemble clos des sites dont l'action n'est pas un littéral.**
///
/// Chaque entrée : le fichier, le motif de l'indirection, et les codes que ce
/// site produit — établis en lisant les branches, pas en devinant.
const SITES_INDIRECTS: &[(&str, &str, &[&str])] = &[
    (
        "crates/kesh-api/src/routes/users.rs",
        "deux libellés selon que le RÔLE change (Story 25-1b)",
        &["user.role_changed", "user.updated"],
    ),
    (
        "crates/kesh-db/src/repositories/invoice_settlements_write.rs",
        "l'action nomme l'EFFET RÉEL : un règlement partiel n'est pas `invoice.paid`",
        &["invoice.paid", "invoice.partially_settled"],
    ),
    (
        "crates/kesh-api/src/routes/reconciliation.rs",
        "même règle qu'au règlement manuel, côté rapprochement (Story 24-2)",
        &["invoice.paid", "invoice.partially_settled"],
    ),
    (
        "crates/kesh-api/src/routes/projects.rs",
        "ternaire `if archived` écrit dans l'appel lui-même",
        &["project.archived", "project.unarchived"],
    ),
    (
        "crates/kesh-db/src/repositories/invoices.rs",
        "ternaire `if paused` pour la suspension de relance",
        &["invoice.dunning_paused", "invoice.dunning_resumed"],
    ),
    (
        "crates/kesh-db/src/repositories/fiscal_years.rs",
        "les cinq appels passent par `build_audit_entry`, qui relaie son `&str`",
        &[
            "fiscal_year.created",
            "fiscal_year.updated",
            "fiscal_year.closed",
            "fiscal_year.reopened",
        ],
    ),
    (
        "crates/kesh-api/src/audit.rs",
        "passe-plat : `from_current_user` relaie l'action de ses appelants, extraits normalement",
        // ⚠️ Codes vides À DESSEIN : ce fichier ne PRODUIT aucun code. L'entrée
        // n'est pas une accusation, c'est la déclaration d'un site que
        // l'extracteur ne résout pas. ⛔ Elle ne protège rien par elle-même :
        // ce qui rendrait visible un littéral ajouté ici, c'est que le fichier
        // soit BALAYÉ — cf. le commentaire sur l'absence de liste d'exclusion.
        &[],
    ),
];

/// La racine `crates/` du dépôt.
fn racine_crates() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/kesh-api a un parent")
        .to_path_buf()
}

/// Tous les `.rs` de production — `crates/*/src/**`, les répertoires `tests/`
/// exclus par construction.
fn fichiers_de_production() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for crate_dir in std::fs::read_dir(racine_crates()).expect("lire crates/") {
        let src = crate_dir.expect("entrée de crates/").path().join("src");
        if src.is_dir() {
            collecter(&src, &mut out);
        }
    }
    out.sort();
    out
}

fn collecter(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).expect("lire un répertoire source") {
        let p = e.expect("entrée de répertoire").path();
        if p.is_dir() {
            collecter(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// Reprise du décommentage de `admin_pat_denied_e2e.rs` — même limite connue :
/// un `//` **dans une chaîne** tronquerait la ligne. Aucun argument d'audit n'en
/// porte, et le diff bilatéral le signalerait s'il en apparaissait un.
fn strip_line_comments(src: &str) -> String {
    src.lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

/// La source d'un fichier, prête à être lue : commentaires masqués et blocs
/// `#[cfg(test)]` **retirés par appariement d'accolades**. **Les trois lecteurs
/// de ce fichier passent par ici.**
///
/// ⛔ **On MASQUE le bloc, on ne TRONQUE pas le fichier — et il a fallu deux
/// passes pour y venir.**
///
/// - Première rédaction : `brut.split("#[cfg(test)]")` sur la source brute.
///   `config.rs:406` porte cette chaîne dans un **doc-comment**, bien avant son
///   vrai attribut : **83 %** du fichier sortait du balayage.
/// - Deuxième : coupe sur une **ligne entière**, ce qui fermait la variante
///   « sous-chaîne dans la prose » — et laissait intacte la plus coûteuse. Un
///   `#[cfg(test)]` posé sur une **méthode** (`invoice_email.rs:901`) ou un `mod
///   tests` placé **au milieu** d'un fichier (`version.rs:97`,
///   `entities/user.rs:178`) sont deux arrangements Rust ordinaires : tronquer
///   à la première occurrence perdait **686, 256 et 75 lignes de PRODUCTION**,
///   dont `send_reminder_batch`, `check_downgrade_protection` et `UserUpdate`.
///   ⚠️ Or `invoice_email.rs` **écrit déjà de l'audit**, et la zone perdue
///   appelait cette écriture : le trou était sous le pied du prochain code
///   d'audit.
///
/// *C'est l'appariement d'accolades que l'AC 18 demandait dès l'origine ; je
/// l'avais simplifié en `split`, en comptant sur le diff bilatéral pour
/// rattraper. Il ne rattrape rien pour une action NEUVE, qui n'entre dans aucun
/// ensemble.*
///
/// ⚠️ Les accolades sont comptées **hors chaînes** — sans quoi une accolade
/// isolée dans un littéral (`"Texte { non fermé"`, réel dans
/// `kesh-core/src/email_template_engine.rs`) déséquilibrerait le compte.
///
/// ⚠️ Limites assumées, héritées de `strip_line_comments` : un `//` **dans une
/// chaîne** tronque la ligne, et les commentaires de **bloc** `/* … */` ne sont
/// pas retirés. Aucun argument d'audit n'en porte aujourd'hui, et les deux vont
/// dans le sens sûr — un rouge injustifié, jamais un vert silencieux.
fn source_assainie(brut: &str) -> String {
    let sans_commentaires = strip_line_comments(brut);
    let lignes: Vec<&str> = sans_commentaires.lines().collect();
    let mut gardees: Vec<&str> = Vec::new();
    let mut i = 0;

    while i < lignes.len() {
        if lignes[i].trim() != "#[cfg(test)]" {
            gardees.push(lignes[i]);
            i += 1;
            continue;
        }
        // Bloc de test : sauter l'attribut, puis l'item qu'il garde, jusqu'à ce
        // que ses accolades se referment.
        i += 1;
        let mut profondeur = 0i32;
        let mut ouvert = false;
        while i < lignes.len() {
            let (o, f) = accolades_hors_chaines(lignes[i]);
            profondeur += o - f;
            if o > 0 {
                ouvert = true;
            }
            i += 1;
            if ouvert && profondeur <= 0 {
                break;
            }
        }
    }
    gardees.join("\n")
}

/// Compte les accolades d'une ligne **en ignorant celles des chaînes**.
fn accolades_hors_chaines(ligne: &str) -> (i32, i32) {
    let (mut ouvrantes, mut fermantes) = (0, 0);
    let (mut dans_chaine, mut echappe) = (false, false);
    for ch in ligne.chars() {
        if dans_chaine {
            if echappe {
                echappe = false;
            } else if ch == '\\' {
                echappe = true;
            } else if ch == '"' {
                dans_chaine = false;
            }
            continue;
        }
        match ch {
            '"' => dans_chaine = true,
            '{' => ouvrantes += 1,
            '}' => fermantes += 1,
            _ => {}
        }
    }
    (ouvrantes, fermantes)
}

/// Découpe les arguments d'un appel — `src` commence **après** la parenthèse
/// ouvrante. Rend `None` si l'appel n'est pas refermé.
fn args_de(src: &str) -> Option<Vec<String>> {
    let (mut depth, mut cur, mut out) = (0usize, String::new(), Vec::new());
    let (mut in_str, mut echappe) = (false, false);
    for ch in src.chars() {
        if in_str {
            cur.push(ch);
            if echappe {
                echappe = false;
            } else if ch == '\\' {
                echappe = true;
            } else if ch == '"' {
                in_str = false;
            }
            continue;
        }
        match ch {
            '"' => {
                in_str = true;
                cur.push(ch);
            }
            '(' | '[' | '{' => {
                depth += 1;
                cur.push(ch);
            }
            ')' | ']' | '}' => {
                if depth == 0 {
                    out.push(cur);
                    return Some(out);
                }
                depth -= 1;
                cur.push(ch);
            }
            ',' if depth == 0 => {
                out.push(std::mem::take(&mut cur));
            }
            _ => cur.push(ch),
        }
    }
    None
}

/// Le code porté par un argument, s'il est littéral.
///
/// ⚠️ `"product.created".to_string()` EST un littéral : le refuser rapporterait
/// quatre-vingts faux sites indirects et noierait les sept vrais.
fn litteral(arg: &str) -> Option<String> {
    let mut a = arg.trim();
    if let Some(reste) = a.strip_prefix("String::from(") {
        a = reste.trim_end().strip_suffix(')')?.trim();
    }
    for suffixe in [".to_string()", ".into()", ".to_owned()"] {
        if let Some(reste) = a.strip_suffix(suffixe) {
            a = reste.trim();
        }
    }
    let interieur = a.strip_prefix('"')?.strip_suffix('"')?;
    (!interieur.contains('"')).then(|| interieur.to_string())
}

/// Ce que le balayage du code rapporte.
#[derive(Default)]
struct Releve {
    actions: BTreeSet<String>,
    types: BTreeSet<String>,
    /// Fichiers (chemin relatif au dépôt) portant un site non résolu.
    sites_non_resolus: BTreeSet<String>,
}

fn relever() -> Releve {
    let mut r = Releve::default();
    let racine = racine_crates();
    for chemin in fichiers_de_production() {
        let relatif = format!(
            "crates/{}",
            chemin
                .strip_prefix(&racine)
                .expect("sous crates/")
                .to_string_lossy()
        );
        let brut = std::fs::read_to_string(&chemin).expect("lire un fichier source");

        // ⛔ Les `mod tests` construisent des entrées avec des codes inventés :
        // leurs blocs sont MASQUÉS par appariement d'accolades.
        //
        // ⚠️ **Ne pas se fier au filet qu'une première rédaction annonçait ici**
        // — « une coupe trop gourmande rougirait sur le code perdu ». C'est vrai
        // du seul cas où le code perdu était l'**unique** site d'une action
        // **déjà** dans `ACTIONS`. Une action **NEUVE** posée dans une zone
        // perdue n'entre dans aucun ensemble et ne fait **rien** rougir. C'est
        // cette phrase rassurante qui a rendu le défaut invisible à trois
        // relectures.
        let source = source_assainie(&brut);

        for (motif, idx_action, idx_type) in FORMES {
            let mut reste = source.as_str();
            while let Some(pos) = reste.find(motif) {
                let avant = &reste[..pos];
                let apres = &reste[pos + motif.len()..];
                reste = apres;

                // La DÉFINITION du helper porte le même motif que ses appels.
                if avant.ends_with("fn ") {
                    continue;
                }
                let Some(args) = args_de(apres) else { continue };
                if args.len() <= *idx_action {
                    continue;
                }
                match litteral(&args[*idx_action]) {
                    Some(code) => {
                        r.actions.insert(code);
                    }
                    None => {
                        r.sites_non_resolus.insert(relatif.clone());
                    }
                }
                match idx_type {
                    None => {
                        r.types.insert(TYPE_DU_HELPER_EXERCICES.to_string());
                    }
                    Some(i) if args.len() > *i => match litteral(&args[*i]) {
                        Some(code) => {
                            r.types.insert(code);
                        }
                        None => {
                            r.sites_non_resolus.insert(relatif.clone());
                        }
                    },
                    Some(_) => {}
                }
            }
        }
    }
    r
}

#[test]
fn tout_site_dont_l_action_n_est_pas_litterale_est_inventorie() {
    let releve = relever();
    let declares: BTreeSet<String> = SITES_INDIRECTS
        .iter()
        .map(|(f, _, _)| (*f).to_string())
        .collect();

    let inconnus: Vec<_> = releve.sites_non_resolus.difference(&declares).collect();
    assert!(
        inconnus.is_empty(),
        "⛔ Site(s) d'audit dont le code n'est pas un littéral, et que \
         l'inventaire n'accueille pas : {inconnus:?}\n\n\
         Soit le code y est écrit en toutes lettres comme partout ailleurs, soit \
         il faut AJOUTER une entrée à SITES_INDIRECTS en y listant, à la main, \
         les codes que ce site produit. ⚠️ Ne pas élargir l'extracteur sans \
         élargir cet inventaire : c'est lui qui empêche une forme imprévue de \
         passer inaperçue."
    );

    let evanouis: Vec<_> = declares.difference(&releve.sites_non_resolus).collect();
    assert!(
        evanouis.is_empty(),
        "⛔ Entrée(s) de SITES_INDIRECTS que le balayage ne retrouve plus : \
         {evanouis:?} — le site a été réécrit en littéral, ou supprimé. Retirer \
         l'entrée, faute de quoi l'inventaire protège un site qui n'existe plus."
    );
}

#[test]
fn les_codes_de_l_inventaire_sont_verifies_dans_le_fichier_de_leur_site() {
    // ⛔ **Sans ce test, l'inventaire AFFIRME au lieu de VÉRIFIER.**
    //
    // Le volet (a) absorbe les codes de `SITES_INDIRECTS` dans `attendues`
    // **inconditionnellement**. Si un site renommait son code — `dunning_paused`
    // devenant `dunning_suspended` —, l'inventaire ET `ACTIONS` porteraient tous
    // deux l'ANCIENNE valeur : le diff bilatéral serait vide, le test vert, et la
    // production écrirait un code **sans libellé**, affiché brut à l'écran et dans
    // le CSV. C'est l'assertion vraie par construction que le `CLAUDE.md` décrit —
    // les deux côtés sortant de la même source.
    //
    // L'AC 18 (d) demandait « ses valeurs **vérifiées** » ; la première rédaction
    // n'avait vérifié que leur présence dans `ACTIONS`, ce qui va de soi.
    //
    // Ici on confronte chaque code déclaré au **texte du fichier** : les branches
    // d'une conditionnelle sont des littéraux, elles y sont donc lisibles même
    // quand l'extracteur ne sait pas les résoudre.
    let racine = racine_crates()
        .parent()
        .expect("le dépôt est le parent de crates/")
        .to_path_buf();

    for (fichier, _motif, codes) in SITES_INDIRECTS {
        let brut = std::fs::read_to_string(racine.join(fichier))
            .unwrap_or_else(|e| panic!("⛔ site inventorié introuvable — {fichier} : {e}"));
        // ⚠️ **Le MÊME assainissement que `relever()`**, et non la source brute.
        // Une première rédaction lisait le fichier tel quel : un code présent
        // dans un simple **commentaire** — ou dans une fixture de `mod tests` —
        // suffisait alors à satisfaire le contrôle, qui s'éteignait en silence.
        let source = source_assainie(&brut);

        // ── Sens 1 : tout code DÉCLARÉ est présent dans le fichier ───────────
        // Ferme le RENOMMAGE.
        for code in *codes {
            let litteral = format!("\"{code}\"");
            assert!(
                source.contains(&litteral),
                "⛔ L'inventaire déclare que « {fichier} » produit « {code} », et ce \
                 littéral ne s'y trouve PAS.\n\n\
                 Soit le site a été renommé — et l'entrée de SITES_INDIRECTS doit \
                 suivre, FAUTE DE QUOI la production écrit un code sans libellé sans \
                 que rien ne rougisse —, soit le code a changé de fichier. ⚠️ Ne pas \
                 corriger en retirant la ligne : c'est le libellé manquant qui est le \
                 défaut, pas l'inventaire."
            );
        }

        // ── Sens 2 : tout code EN FORME D'ACTION trouvé dans le fichier est ──
        // ── soit déclaré, soit déjà connu d'`ACTIONS` ───────────────────────
        //
        // ⛔ **Ferme l'AJOUT, et c'est le sens qui manquait.** Le sens 1 seul
        // laissait passer une branche NEUVE : `else if définitif {
        // "invoice.dunning_cancelled" }` n'est ni relevé par l'extracteur — la
        // variable n'est pas un littéral — ni déclaré à l'inventaire, donc il
        // n'entrait dans AUCUN ensemble. Les sept tests restaient verts pendant
        // que la production écrivait un code sans libellé.
        //
        // *Le contrôle n'était unilatéral que dans un sens : il fallait le rendre
        // bilatéral sur le périmètre restreint des fichiers inventoriés.*
        for code in litteraux_en_forme_de_code(&source) {
            assert!(
                codes.contains(&code.as_str()) || ACTIONS.contains(&code.as_str()),
                "⛔ « {fichier} » contient le littéral « {code} », qui a la forme d'un \
                 code d'action et n'est **ni** déclaré à son entrée de SITES_INDIRECTS \
                 **ni** présent dans `ACTIONS`.\n\n\
                 Si c'est une action neuve : lui donner son libellé dans les quatre \
                 catalogues, l'ajouter à `ACTIONS`, et l'inscrire aux codes de ce site. \
                 Si ce n'en est pas une, c'est que le tamis de \
                 `litteraux_en_forme_de_code` est trop large — l'y exclure \
                 explicitement, jamais en retirant ce contrôle."
            );
        }
    }
}

/// Littéraux qui ont la forme d'un code d'action **sans en être un**, et qu'on
/// déclare plutôt que d'élargir le tamis.
///
/// ⚠️ **Un ensemble clos, borné aux SEPT fichiers inventoriés** — et non une
/// liste ouverte sur tout le dépôt : elle ne grossit que si l'on écrit un
/// littéral de cette forme dans l'un d'eux. Chaque entrée porte son motif, comme
/// l'inventaire lui-même. *(« six » ici était faux : `SITES_INDIRECTS` en compte
/// sept depuis l'ajout d'`audit.rs` — décompte relevé par la passe 3, dans le
/// commentaire même qui corrigeait un défaut de comptage voisin.)*
const PAS_DES_CODES: &[(&str, &str)] = &[(
    "bank_account.journal_account_id",
    "clé de CONFIGURATION nommant le champ manquant (`DbError::ConfigurationRequired`), \
     pas un acte — la forme `table.colonne` est ici fortuite",
)];

/// Les littéraux d'un source qui ont la **forme** d'un code d'action :
/// `entité.acte`, un seul point, ni espace ni ponctuation.
///
/// ⚠️ **Un tamis, pas une preuve** : il repère les codes *candidats* dans un
/// fichier inventorié, là où l'extracteur positionnel ne sait pas lire. Les
/// requêtes SQL (espaces), les formats de date (`%`, `-`) et les clés JSON (pas
/// de point) en sortent d'elles-mêmes.
///
/// ⛔ **Le préfixe doit faire au moins trois caractères**, ce qui écarte la
/// famille entière des **alias SQL** — `"i.date"`, `"c.name"`, relevés dans le
/// `match` qui traduit un enum de tri en noms de colonnes. Aucun type d'entité
/// de Kesh ne descend sous quatre caractères (`user`), la borne est donc sûre.
/// *Sans elle, il aurait fallu déclarer cinq exceptions qui n'ont rien à voir
/// avec l'audit.*
fn litteraux_en_forme_de_code(source: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();

    // ⛔ **Suivi d'état, et non `find('"')`.** Une première rédaction appariait
    // les guillemets naïvement, sans regarder les ÉCHAPPEMENTS — alors qu'`args_de`,
    // vingt lignes plus haut, les suit. Un seul `\"` en nombre impair
    // désynchronisait l'appariement pour **tout le reste du fichier** : l'ensemble
    // rendu devenait VIDE, la boucle d'assertion ne s'exécutait plus, et le trou
    // que ce tamis existe pour fermer se rouvrait — en silence, la garde restant
    // verte. Ce n'est pas un cas tordu : `reconciliation.rs` porte déjà des
    // guillemets échappés, en nombre pair par chance.
    let mut courant = String::new();
    let (mut dans_chaine, mut echappe) = (false, false);
    for ch in source.chars() {
        if dans_chaine {
            if echappe {
                echappe = false;
                // ⚠️ Le `\` est CONSERVÉ : sans lui, `"erreur.\ndetail"` devenait
                // `erreur.ndetail`, que le tamis retenait comme un code. Un faux
                // positif — donc un rouge injustifié —, latent aujourd'hui, mais
                // que le scanner naïf n'avait pas.
                courant.push('\\');
                courant.push(ch);
            } else if ch == '\\' {
                echappe = true;
            } else if ch == '"' {
                dans_chaine = false;
                if let Some(code) = retenir_si_code(&courant) {
                    out.insert(code);
                }
                courant.clear();
            } else {
                courant.push(ch);
            }
        } else if ch == '"' {
            dans_chaine = true;
            courant.clear();
        }
    }
    out
}

/// Le contenu d'un littéral, s'il a la forme d'un code d'action.
fn retenir_si_code(contenu: &str) -> Option<String> {
    if PAS_DES_CODES.iter().any(|(c, _)| *c == contenu) {
        return None;
    }
    let (prefixe, acte) = contenu.split_once('.')?;
    (prefixe.len() >= 3
        && !acte.is_empty()
        && !acte.contains('.')
        && contenu
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == '.' || c == '_'))
    .then(|| contenu.to_string())
}

#[test]
fn aucune_route_ne_derive_une_cle_de_code_hors_du_module_source_unique() {
    // ⛔ **Ce test existe parce que son absence a été PROUVÉE coûteuse.**
    //
    // La cellule « type d'auteur » de l'export redérivait sa clé à la main —
    // `message_key(PREFIX_ACTOR_TYPE, …)` — au lieu d'appeler `actor_type_label`.
    // Le rendu était identique, si bien qu'**aucun** des quinze tests HTTP ne
    // rougissait : vérifié en remettant la dérivation manuelle, suite verte.
    //
    // Le défaut n'est donc pas un mauvais affichage, c'est qu'un **second chemin
    // de dérivation** vivait hors du module déclaré source unique : son test
    // d'appartenance n'était plus exercé par le code qui sert le fichier, et la
    // fonction imposée par l'AC 16 survivait à sa propre inutilité, couverte par
    // ses seuls tests unitaires.
    //
    // Un test de SORTIE ne peut pas trancher — les deux chemins rendent le même
    // texte, et `actor_type` étant un ENUM à deux valeurs, aucun code inconnu
    // n'est insérable en base. Ce qui se vérifie, c'est le CHEMIN.
    // ⚠️ **Tout `kesh-api/src`, et non le seul fichier de la route.** Une
    // première rédaction lisait `include_str!("../src/routes/audit_log.rs")` :
    // la règle invoquée est une règle de DÉPÔT, et une dérivation posée dans un
    // autre fichier de route lui échappait. Le balayage large passe
    // immédiatement — aucun second consommateur n'existe aujourd'hui — et
    // couvrira la 25-1c-b, qui ouvrira la surface.
    let mut fichiers = Vec::new();
    collecter(&racine_crates().join("kesh-api/src"), &mut fichiers);

    for chemin in fichiers {
        // Le module EST l'endroit légitime de la dérivation.
        if chemin.ends_with("audit_labels.rs") {
            continue;
        }
        let brut = std::fs::read_to_string(&chemin).expect("lire un fichier source");
        // Coupe `#[cfg(test)]` comme `relever()` : un test unitaire qui
        // construirait une clé attendue n'enfreint aucune règle, et le faire
        // rougir pousserait à contourner la garde — donc à l'affaiblir.
        let source = source_assainie(&brut);

        for interdit in [
            "message_key(",
            "PREFIX_ACTOR_TYPE",
            "PREFIX_ACTION",
            "PREFIX_ENTITY",
        ] {
            assert!(
                !source.contains(interdit),
                "⛔ `{}` emploie « {interdit} » : il dérive donc une clé de code \
                 lui-même, alors que l'AC 12 impose que « les libellés viennent des \
                 fonctions de l'AC 16, et d'elles seules ».\n\n\
                 Appeler `audit_labels::{{action,entity_type,actor_type}}_label`. ⚠️ Si \
                 un repli est nécessaire quand la clé manque, sa place est DANS la \
                 fonction du module, jamais au site d'appel — sinon le test \
                 d'appartenance cesse d'être exercé par le seul code qui s'en sert.",
                chemin.display()
            );
        }
    }

    // ⚠️ **Ce que cette garde NE ferme PAS, et il faut le dire plutôt que de le
    // laisser croire** : elle est TEXTUELLE. Un alias d'import
    // (`use …::message_key as mk;`) ou une dérivation réécrite à la main
    // (`format!("audit-log-action-{}", …)`) lui échappent, de même qu'un `//`
    // placé dans une chaîne AVANT l'appel, qui tronque la ligne au masquage des
    // commentaires. Elle attrape l'oubli et la reprise distraite — la forme la
    // plus probable —, pas la contorsion délibérée.

    // ⚠️ `traduire_ou_replier` reste légitime : elle sert les en-têtes de colonne
    // (`audit-log-csv-header-*`), qui ne sont **pas** des codes du journal et
    // n'ont donc pas de fonction de libellé dans le module.
}

#[test]
fn le_masquage_des_blocs_de_test_ne_laisse_aucun_attribut_derriere_lui() {
    // ⛔ **La garde du masqueur lui-même** (D4-ter : inventorier ce qui ne
    // résout pas, plutôt qu'énumérer les formes qui marchent).
    //
    // Si `source_assainie` rendait une source contenant encore un
    // `#[cfg(test)]`, c'est que l'appariement d'accolades a échoué — accolade
    // dans un littéral non détectée, attribut sur une forme imprévue — et le
    // bloc de test serait **balayé comme de la production**. À l'inverse, une
    // troncature emporterait la production qui suit, ce qui était le défaut de
    // la rédaction précédente.
    //
    // ⚠️ Cette assertion est **comptable et décidable** : elle rougit sur tout
    // fichier du workspace, sans énumérer les arrangements acceptables.
    for chemin in fichiers_de_production() {
        let brut = std::fs::read_to_string(&chemin).expect("lire un fichier source");
        if !brut.contains("#[cfg(test)]") {
            continue;
        }
        let assainie = source_assainie(&brut);
        assert!(
            !assainie.contains("#[cfg(test)]"),
            "⛔ `{}` : le masquage des blocs de test a échoué — un attribut \
             subsiste dans la source assainie.\n\n\
             L'appariement d'accolades de `source_assainie` n'a pas su refermer \
             un bloc : accolade isolée dans un littéral, ou attribut posé sur une \
             forme imprévue. ⚠️ Ne pas « corriger » en tronquant le fichier : \
             c'est ce que faisait la rédaction précédente, et elle emportait \
             jusqu'à 686 lignes de PRODUCTION.",
            chemin.display()
        );
    }
}

#[test]
fn l_inventaire_compte_ce_que_le_fichier_annonce() {
    // ⚠️ Un nombre écrit en prose se périme **en silence** : « six sites » est
    // resté faux à trois endroits de ce fichier après l'ajout du septième, dont
    // deux qu'un patch de remédiation avait laissés. Cette assertion, elle, ne
    // se périme pas — elle rougit.
    assert_eq!(
        SITES_INDIRECTS.len(),
        7,
        "⛔ Le nombre de sites indirects a changé. Mettre à jour les mentions en \
         toutes lettres de l'en-tête de ce fichier — elles sont trois — puis ce \
         nombre. `grep -niE '\\b(six|sept|huit)\\b'` les trouve."
    );
}

#[test]
fn aucun_code_ne_contient_de_slash_qui_tromperait_le_masquage_des_commentaires() {
    // ⚠️ `strip_line_comments` coupe à `//`. Un code qui en contiendrait serait
    // tronqué **avant** l'extraction, et le site deviendrait invisible. Aucun
    // code n'en porte aujourd'hui ; cette assertion empêche le premier d'arriver
    // sans qu'on s'en aperçoive.
    for (nom, liste) in [
        ("ACTIONS", ACTIONS),
        ("ENTITY_TYPES", ENTITY_TYPES),
        ("ACTOR_TYPES", ACTOR_TYPES),
    ] {
        for code in liste {
            assert!(
                !code.contains('/'),
                "⛔ « {code} » de {nom} contient « / » : le masquage des commentaires \
                 tronquerait son site d'écriture, qui disparaîtrait de l'extraction."
            );
        }
    }
}

#[test]
fn les_actions_de_la_liste_sont_exactement_celles_que_le_code_ecrit() {
    let releve = relever();
    let mut attendues = releve.actions;
    for (_, _, codes) in SITES_INDIRECTS {
        attendues.extend(codes.iter().map(|c| (*c).to_string()));
    }
    let declarees: BTreeSet<String> = ACTIONS.iter().map(|a| (*a).to_string()).collect();

    let manquantes: Vec<_> = attendues.difference(&declarees).collect();
    assert!(
        manquantes.is_empty(),
        "⛔ Action(s) écrite(s) par le code et absente(s) d'`ACTIONS` : \
         {manquantes:?}\n\n\
         Les ajouter à `crates/kesh-api/src/audit_labels.rs` ET leur donner un \
         libellé dans les quatre catalogues. Sans libellé, le journal affiche le \
         code brut à son lecteur."
    );

    let mortes: Vec<_> = declarees.difference(&attendues).collect();
    assert!(
        mortes.is_empty(),
        "⛔ Action(s) déclarée(s) dans `ACTIONS` que plus aucun site n'écrit : \
         {mortes:?}\n\n\
         ⚠️ Avant de les retirer : le journal CONSERVE les codes des versions \
         antérieures, et une entrée de 2026 doit rester lisible. Si le code n'est \
         plus produit mais reste en base, c'est ici qu'il faut le dire — sinon, \
         retirer la ligne."
    );
}

#[test]
fn les_types_d_entite_de_la_liste_sont_exactement_ceux_que_le_code_ecrit() {
    let releve = relever();
    let declares: BTreeSet<String> = ENTITY_TYPES.iter().map(|t| (*t).to_string()).collect();

    let manquants: Vec<_> = releve.types.difference(&declares).collect();
    assert!(
        manquants.is_empty(),
        "⛔ Type(s) d'entité écrit(s) par le code et absent(s) d'`ENTITY_TYPES` : \
         {manquants:?} — les ajouter, avec leur libellé dans les quatre \
         catalogues."
    );

    let morts: Vec<_> = declares.difference(&releve.types).collect();
    assert!(
        morts.is_empty(),
        "⛔ Type(s) déclaré(s) dans `ENTITY_TYPES` que plus aucun site n'écrit : \
         {morts:?} — même réserve que pour les actions : le journal conserve \
         l'ancien."
    );
}

#[test]
fn chaque_code_a_son_libelle_dans_les_quatre_catalogues() {
    // ⛔ On lit les `.ftl` DIRECTEMENT, et non via `I18nBundle`.
    //
    // `format` replie sur le français et `all_messages` comble les trous avec le
    // français : passer par le bundle ferait passer au vert une locale qui n'a
    // aucune des 122 clés. Le seul endroit où l'absence se voit est le fichier.
    let locales = racine_crates().join("kesh-i18n/locales");
    let attendues: Vec<String> = [
        (PREFIX_ENTITY, ENTITY_TYPES),
        (PREFIX_ACTION, ACTIONS),
        (PREFIX_ACTOR_TYPE, ACTOR_TYPES),
    ]
    .iter()
    .flat_map(|(prefixe, liste)| liste.iter().map(|code| message_key(prefixe, code)))
    .collect();

    for locale in ["fr-CH", "de-CH", "it-CH", "en-CH"] {
        let ftl = std::fs::read_to_string(locales.join(locale).join("messages.ftl"))
            .unwrap_or_else(|e| panic!("lire le catalogue {locale} : {e}"));
        let presentes: BTreeSet<&str> = ftl
            .lines()
            .filter(|l| !l.starts_with([' ', '#', '.']))
            .filter_map(|l| l.split_once(" =").map(|(cle, _)| cle.trim()))
            .collect();

        let absentes: Vec<&String> = attendues
            .iter()
            .filter(|cle| !presentes.contains(cle.as_str()))
            .collect();
        assert!(
            absentes.is_empty(),
            "⛔ {} clé(s) de libellé d'audit absente(s) de {locale} : {:?}…\n\n\
             Un code sans libellé s'affiche en clair dans le journal ET dans le \
             CSV exporté. Les quatre catalogues doivent porter les {} clés.",
            absentes.len(),
            &absentes[..absentes.len().min(8)],
            attendues.len()
        );
    }
}
