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
//! Six sites du dépôt n'en portent pas : l'action y est une variable ou un
//! ternaire. Énumérer « les formes qui marchent » laisserait une septième forme
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

/// Fichiers dont les sites ne produisent **aucun** code : ce sont des
/// passe-plats qui relaient l'action de leurs appelants.
///
/// ⚠️ `audit.rs` implémente `from_current_user` en appelant `::api_key` ou
/// `::user` avec ses propres paramètres. L'y compter rapporterait deux sites
/// « non résolus » qui ne sont l'angle mort de personne — les vrais sites sont
/// les appelants de `from_current_user`, extraits normalement.
const PASSE_PLATS: &[&str] = &["crates/kesh-api/src/audit.rs"];

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
/// quatre-vingts faux sites indirects et noierait les six vrais.
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
        if PASSE_PLATS.contains(&relatif.as_str()) {
            continue;
        }
        let brut = std::fs::read_to_string(&chemin).expect("lire un fichier source");

        // ⛔ Les `mod tests` construisent des entrées avec des codes inventés.
        // On coupe au premier `#[cfg(test)]`. Une coupe trop gourmande ne passe
        // pas inaperçue : le volet « liste → code » rougirait sur le code perdu.
        let source = strip_line_comments(brut.split("#[cfg(test)]").next().unwrap_or(""));

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
