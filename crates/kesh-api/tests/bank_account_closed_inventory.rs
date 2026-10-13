//! Registre statique des écrivains d'une ligne rattachée à un compte bancaire
//! — Story 28-4a (#642), AC8.
//!
//! Un compte bancaire **clos** est en lecture seule : tout chemin d'écriture
//! qui le viserait doit le refuser côté serveur. La garde est posée aux points
//! d'étranglement du dépôt ; ce test tient l'**inventaire clos** de ces
//! points. Il lit les sources de production de `crates/kesh-db/src/` et
//! `crates/kesh-api/src/` — **hors** blocs `#[cfg(test)]` et **hors**
//! commentaires —, extrait tout littéral qui porte une instruction `INSERT
//! [IGNORE] INTO`, `REPLACE INTO`, `UPDATE` ou `DELETE FROM` visant l'une des
//! sept tables (`bank_imports`, `bank_transactions`, `payment_batches`,
//! `payment_batch_items`, `invoice_settlements`, `supplier_invoices`,
//! `bank_accounts`), la rattache à sa fonction englobante, et exige que chaque
//! couple `(fichier, fonction)` figure au [`REGISTRE`] avec un statut. Diff
//! **dans les deux sens** : une écriture neuve non classée rougit, une entrée
//! sans site rougit.
//!
//! Second inventaire, **par appelant** : la garde du règlement par virement ne
//! vit pas dans `invoice_settlements::create_in_tx` mais chez ses appelants
//! (C-28-4-14) ; tout appel `invoice_settlements::create_in_tx(` hors tests
//! exige donc un statut au [`APPELANTS_REGLEMENT`], diff dans les deux sens.
//!
//! ⛔ **Ce que le test n'établit pas** : que la garde **s'exécute** — c'est
//! l'affaire des tests de l'AC6 (`bank_accounts_closed_e2e.rs`). Il ferme le
//! mode d'échec « un écrivain ajouté demain contourne la clôture sans que rien
//! ne le signale ».
//!
//! ⚠️ **Ce qu'il ne voit pas** — angles morts assumés, aucun n'est une entrée
//! du registre :
//!
//! - les écritures dont le **nom de table n'est pas littéral** : `format!` à
//!   nom variable et `QueryBuilder` dont la table est poussée par
//!   `push(table)` (un `QueryBuilder::new("INSERT INTO bank_transactions …")`
//!   à littéral **est** vu). Trois sites connus de la première forme :
//!   (1) et (2) `kesh-db/src/backup.rs`, `restore_body` —
//!   `format!("DELETE FROM `{table}`")` et `format!("INSERT INTO `{table}` …")`
//!   —, par `restore_tables_in_tx`, qu'appelle la restauration d'installation
//!   (`kesh-api/src/routes/admin.rs`) : hors périmètre, elle remplace toute la
//!   base par l'état d'une sauvegarde qui respectait les invariants ;
//!   (3) `kesh-seed/src/lib.rs`, remise à zéro de la démo
//!   (`format!("DELETE FROM `{table}`")`, toutes tables), hors des deux crates
//!   lues ;
//! - le SQL hors des deux crates lues, dont le rejeu post-restauration
//!   (`kesh-db/src/post_restore/*.sql`, fichiers SQL) ;
//! - un appel du créateur de règlement par un chemin importé
//!   (`use invoice_settlements::create_in_tx;` puis `create_in_tx(`) ;
//! - une requête assemblée de plusieurs littéraux dont aucun, seul, ne porte à
//!   la fois le verbe et la table.
//!
//! Les routes de test (`routes/test_endpoints.rs`) n'écrivent aucune des sept
//! tables. Si un écrivain hors périmètre devient un jour visible de
//! l'extraction, il se classe avec un statut créé pour lui, à son premier
//! site.

#[path = "../../kesh-db/tests/lexique/mod.rs"]
mod lexique;

use lexique::{
    blocs_de_test, decouper, fonction_englobante, fonctions, neutraliser_echappements,
    racine_crates, sources,
};
use std::collections::BTreeSet;

/// Les sept tables qui portent un compte bancaire (fermé :
/// `grep -rn "REFERENCES bank_accounts" crates/kesh-db/migrations/`, plus
/// `bank_accounts` elle-même et `payment_batch_items`, rattachée au lot).
const TABLES: [&str; 7] = [
    "bank_imports",
    "bank_transactions",
    "payment_batches",
    "payment_batch_items",
    "invoice_settlements",
    "supplier_invoices",
    "bank_accounts",
];

/// Le statut d'un écrivain au regard du compte clos.
#[derive(Debug)]
enum Statut {
    /// Refuse un compte clos : la fonction de garde ou l'instruction qui lit
    /// `closed_on`.
    Gardee(&'static str),
    /// Hors d'atteinte par invariant (`I1` : aucune `pending` sur un compte
    /// clos ; `I2` : aucun lot `generated` ; `I3` : jamais principal).
    Invariant(&'static str, &'static str),
    /// L'écriture ne porte aucun compte bancaire, ou porte un compte neuf.
    SansCompteBancaire(&'static str),
    /// Ne fait que **sortir** de l'état qu'un invariant interdit sur un compte
    /// clos, laissé sans garde pour réparer ce que l'angle mort L1 (#651) a
    /// laissé passer.
    SortieAutorisee(&'static str),
}

impl Statut {
    /// La raison ne peut pas être vide : un statut sans raison serait un
    /// classement de confiance.
    fn raison(&self) -> &'static str {
        match self {
            Statut::Gardee(r)
            | Statut::Invariant(_, r)
            | Statut::SansCompteBancaire(r)
            | Statut::SortieAutorisee(r) => r,
        }
    }
}

/// Le registre des écrivains : `(fichier relatif à crates/, fonction, statut)`.
const REGISTRE: &[(&str, &str, Statut)] = &[
    // --- kesh-api : réconciliation ---------------------------------------
    (
        "kesh-api/src/routes/reconciliation.rs",
        "accept_one_invoice",
        Statut::Invariant(
            "I1",
            "transaction pending dans l'état commité jusqu'au commit du lot ; sous L1 (#651), le pré-vol R1 refuse",
        ),
    ),
    (
        "kesh-api/src/routes/reconciliation.rs",
        "accept_one_split",
        Statut::Invariant("I1", "transaction pending ; pré-vol R1"),
    ),
    (
        "kesh-api/src/routes/reconciliation.rs",
        "accept_one_rule",
        Statut::Invariant("I1", "transaction pending ; pré-vol R1"),
    ),
    (
        "kesh-api/src/routes/reconciliation.rs",
        "reject_batch",
        Statut::Invariant("I1", "transaction pending ; pré-vol R2"),
    ),
    (
        "kesh-api/src/routes/reconciliation.rs",
        "post_manual_once",
        Statut::Invariant("I1", "transaction pending ; pré-vol R3"),
    ),
    (
        "kesh-api/src/routes/reconciliation.rs",
        "post_split_once",
        Statut::Invariant("I1", "transaction pending ; pré-vol R4"),
    ),
    (
        "kesh-api/src/routes/reconciliation.rs",
        "unreject_once",
        Statut::Gardee(
            "pré-vol R8 (ensure_bank_account_open) et bank_accounts::ensure_open_in_tx après le verrou de la ligne",
        ),
    ),
    // --- kesh-api : comptes bancaires -----------------------------------
    (
        "kesh-api/src/routes/bank_accounts.rs",
        "create_bank_account",
        Statut::SansCompteBancaire("compte neuf, ouvert"),
    ),
    // --- kesh-db : comptes bancaires ------------------------------------
    (
        "kesh-db/src/repositories/bank_accounts.rs",
        "create",
        Statut::SansCompteBancaire("compte neuf, ouvert"),
    ),
    (
        "kesh-db/src/repositories/bank_accounts.rs",
        "upsert_primary_in_tx",
        Statut::Invariant(
            "I3",
            "l'UPDATE vise le principal, jamais clos ; l'INSERT crée un compte neuf",
        ),
    ),
    (
        "kesh-db/src/repositories/bank_accounts.rs",
        "set_journal_account_id_for_company",
        Statut::Gardee("closed_on lu dans son FOR UPDATE, après la version (AC5)"),
    ),
    (
        "kesh-db/src/repositories/bank_accounts.rs",
        "update_for_company",
        Statut::Gardee("closed_on lu dans son FOR UPDATE, après la version (AC5)"),
    ),
    (
        "kesh-db/src/repositories/bank_accounts.rs",
        "archive_for_company",
        Statut::Gardee(
            "closed_on lu dans son FOR UPDATE, après la version ; et pré-vol de la route (AC5)",
        ),
    ),
    (
        "kesh-db/src/repositories/bank_accounts.rs",
        "flip_primary_off_for_company",
        Statut::Invariant("I3", "vise le principal, jamais clos"),
    ),
    (
        "kesh-db/src/repositories/bank_accounts.rs",
        "write_closed_on",
        Statut::Gardee(
            "le geste lui-même (close_in_tx / reopen_in_tx) : ligne verrouillée par lock_for_closure, état contrôlé avant l'écriture",
        ),
    ),
    // --- kesh-db : règlements clients -----------------------------------
    (
        "kesh-db/src/repositories/invoice_settlements.rs",
        "create_in_tx",
        Statut::Gardee("par ses appelants : inventaire des appelants (APPELANTS_REGLEMENT)"),
    ),
    (
        "kesh-db/src/repositories/invoice_settlements_write.rs",
        "cancel_settlement_in_tx",
        Statut::Gardee(
            "bank_accounts::ensure_open_in_tx sur settlement_bank_account_id, après (2-bis)",
        ),
    ),
    // --- kesh-db : rapprochement ----------------------------------------
    (
        "kesh-db/src/repositories/reconciliation_cancel.rs",
        "cancel_in_tx",
        Statut::Gardee("bank_accounts::ensure_open_in_tx sur bt.bank_account_id, après (3)"),
    ),
    // --- kesh-db : lots de paiement -------------------------------------
    (
        "kesh-db/src/repositories/payment_batches.rs",
        "create_batch",
        Statut::Gardee("closed_on lu dans la lecture existante de l'étape (0)"),
    ),
    (
        "kesh-db/src/repositories/payment_batches.rs",
        "confirm_batch",
        Statut::Gardee("closed_on lu à l'étape (2) ; chaque facture par pay_in_tx (W27) ; et I2"),
    ),
    (
        "kesh-db/src/repositories/payment_batches.rs",
        "cancel_batch",
        Statut::SortieAutorisee(
            "annuler un lot generated laissé par L1 (#651) : le geste qui en fait sortir (C-28-4-23)",
        ),
    ),
    // --- kesh-db : imports bancaires ------------------------------------
    (
        "kesh-db/src/repositories/bank_imports.rs",
        "create_with_transactions",
        Statut::Gardee("bank_accounts::ensure_open_unarchived_in_tx en tête de la fonction"),
    ),
    (
        "kesh-db/src/repositories/bank_imports.rs",
        "delete_unreconciled_in_tx",
        Statut::Gardee(
            "bank_accounts::ensure_open_in_tx après le comptage de l'étape 3, avant le DELETE",
        ),
    ),
    // --- kesh-db : factures fournisseurs --------------------------------
    (
        "kesh-db/src/repositories/supplier_invoices.rs",
        "create_in_tx",
        Statut::SansCompteBancaire(
            "la facture naît ouverte ; l'INSERT ne nomme aucun settlement_*",
        ),
    ),
    (
        "kesh-db/src/repositories/supplier_invoices.rs",
        "pay_in_tx",
        Statut::Gardee("closed_on, archived lus dans le FOR UPDATE du compte, étape (3)"),
    ),
    (
        "kesh-db/src/repositories/supplier_invoices.rs",
        "cancel_in_tx",
        Statut::Gardee(
            "bank_accounts::ensure_open_in_tx sur settlement_bank_account_id, après le verrou de l'exercice d'achat",
        ),
    ),
    (
        "kesh-db/src/repositories/supplier_invoices.rs",
        "cancel_settlement_in_tx",
        Statut::Gardee(
            "bank_accounts::ensure_open_in_tx sur settlement_bank_account_id, après (1-bis)",
        ),
    ),
];

/// Second inventaire : les appelants de `invoice_settlements::create_in_tx(`.
const APPELANTS_REGLEMENT: &[(&str, &str, Statut)] = &[
    (
        "kesh-db/src/repositories/invoice_settlements_write.rs",
        "settle_invoice",
        Statut::Gardee("closed_on, archived lus dans le FOR UPDATE du compte, étape (3)"),
    ),
    (
        "kesh-api/src/routes/reconciliation.rs",
        "accept_one_invoice",
        Statut::Invariant(
            "I1",
            "transaction pending dans l'état commité jusqu'au commit du lot ; sous L1 (#651), le pré-vol R1 refuse",
        ),
    ),
    (
        "kesh-db/src/repositories/invoice_settlements_write.rs",
        "write_off_invoice",
        Statut::SansCompteBancaire("passage en perte : aucun compte bancaire"),
    ),
];

/// Modificateurs qui peuvent séparer le verbe de `INTO` / `FROM` ou de la
/// table (`INSERT IGNORE INTO`, `UPDATE LOW_PRIORITY t`, `DELETE QUICK FROM`).
const MODIFICATEURS: [&str; 5] = [
    "IGNORE",
    "LOW_PRIORITY",
    "HIGH_PRIORITY",
    "DELAYED",
    "QUICK",
];

/// Les `(verbe, table)` d'écriture qu'un texte porte, sur l'une des
/// [`TABLES`] — insensible à la casse, blancs et échappements compris.
fn ecritures_du_texte(texte: &str) -> Vec<(&'static str, &'static str)> {
    let mots: Vec<String> = texte
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|m| !m.is_empty())
        .map(|m| m.to_ascii_uppercase())
        .collect();
    let mut out = Vec::new();
    for (n, mot) in mots.iter().enumerate() {
        let (verbe, liaison): (&'static str, Option<&str>) = match mot.as_str() {
            "INSERT" => ("INSERT", Some("INTO")),
            "REPLACE" => ("REPLACE", Some("INTO")),
            "DELETE" => ("DELETE", Some("FROM")),
            "UPDATE" => ("UPDATE", None),
            _ => continue,
        };
        let mut i = n + 1;
        while mots
            .get(i)
            .is_some_and(|m| MODIFICATEURS.contains(&m.as_str()))
        {
            i += 1;
        }
        if let Some(l) = liaison {
            if mots.get(i).map(String::as_str) != Some(l) {
                continue;
            }
            i += 1;
        }
        if let Some(table) = mots
            .get(i)
            .and_then(|m| TABLES.iter().find(|t| m.eq_ignore_ascii_case(t)))
        {
            out.push((verbe, *table));
        }
    }
    out
}

/// Un site d'écriture : `(fichier, fonction, extrait)`.
type Site = (String, String, String);

/// Les écritures d'une source, hors blocs `#[cfg(test)]` et hors
/// commentaires (le découpage ne prend pour littéraux que les chaînes du code).
/// Deux lectures du littéral, et leur union : neutralisée et brute (P7).
fn sites_d_ecriture(fichier: &str, source: &str) -> Vec<Site> {
    let (litteraux, masque) = decouper(source);
    let tests = blocs_de_test(&masque);
    let fns = fonctions(&masque);
    let mut out = Vec::new();
    for l in litteraux {
        if tests.iter().any(|(de, a)| l.debut >= *de && l.debut <= *a) {
            continue;
        }
        let mut vus: BTreeSet<(&str, &str)> = BTreeSet::new();
        vus.extend(ecritures_du_texte(&neutraliser_echappements(&l.texte)));
        vus.extend(ecritures_du_texte(&l.texte));
        let fonction = fonction_englobante(&fns, l.debut)
            .map_or("<hors fonction>".to_string(), |(nom, _, _)| nom.clone());
        for (verbe, table) in vus {
            out.push((
                fichier.to_string(),
                fonction.clone(),
                format!("{verbe} {table}"),
            ));
        }
    }
    out
}

/// Les appels `appel` d'une source, hors tests et commentaires :
/// `(fichier, fonction)`.
fn appels(fichier: &str, source: &str, appel: &str) -> Vec<(String, String)> {
    let (_, masque) = decouper(source);
    let tests = blocs_de_test(&masque);
    let fns = fonctions(&masque);
    let mut out = Vec::new();
    let mut depuis = 0;
    while let Some(n) = masque[depuis..].find(appel) {
        let p = depuis + n;
        depuis = p + appel.len();
        if tests.iter().any(|(de, a)| p >= *de && p <= *a) {
            continue;
        }
        let fonction = fonction_englobante(&fns, p)
            .map_or("<hors fonction>".to_string(), |(nom, _, _)| nom.clone());
        out.push((fichier.to_string(), fonction));
    }
    out
}

/// Les sources de production des deux crates lues : `(chemin relatif à
/// crates/, contenu)`.
fn sources_lues() -> Vec<(String, String)> {
    let racine = racine_crates();
    let mut fichiers = Vec::new();
    for krate in ["kesh-db", "kesh-api"] {
        sources(&racine.join(krate).join("src"), &mut fichiers);
    }
    assert!(
        fichiers.len() > 50,
        "balayage trop court ({})",
        fichiers.len()
    );
    fichiers.sort();
    fichiers
        .into_iter()
        .map(|f| {
            let rel = f
                .strip_prefix(&racine)
                .expect("sous crates/")
                .display()
                .to_string();
            (rel, std::fs::read_to_string(&f).expect("source"))
        })
        .collect()
}

/// Le diff ensembliste dans les deux sens : `(couples non classés, entrées
/// sans site)`.
fn ecarts(
    couples: &BTreeSet<(String, String)>,
    registre: &[(&str, &str, Statut)],
) -> (Vec<String>, Vec<String>) {
    let classes: BTreeSet<(String, String)> = registre
        .iter()
        .map(|(f, n, _)| (f.to_string(), n.to_string()))
        .collect();
    let non_classes = couples
        .difference(&classes)
        .map(|(f, n)| format!("{f}::{n}"))
        .collect();
    let sans_site = classes
        .difference(couples)
        .map(|(f, n)| format!("{f}::{n}"))
        .collect();
    (non_classes, sans_site)
}

/// T-a33 (AC8) — chaque écrivain des sept tables est classé, et chaque
/// entrée du registre a un site.
#[test]
fn every_bank_account_writer_is_classified() {
    let mut sites = Vec::new();
    for (fichier, source) in sources_lues() {
        sites.extend(sites_d_ecriture(&fichier, &source));
    }
    let couples: BTreeSet<(String, String)> = sites
        .iter()
        .map(|(f, n, _)| (f.clone(), n.clone()))
        .collect();
    let (non_classes, sans_site) = ecarts(&couples, REGISTRE);
    let detail: Vec<String> = sites
        .iter()
        .map(|(f, n, e)| format!("  {f}::{n} — {e}"))
        .collect();
    assert!(
        non_classes.is_empty(),
        "⛔ écrivain(s) d'une ligne rattachée à un compte bancaire NON CLASSÉ(S) \
         (Story 28-4a, AC8) — un compte clos doit être refusé sur ce chemin, ou \
         l'écrivain se classer avec sa raison au REGISTRE :\n{}\n\nSites vus :\n{}",
        non_classes.join("\n"),
        detail.join("\n")
    );
    assert!(
        sans_site.is_empty(),
        "entrée(s) du REGISTRE sans site d'écriture — renommée, déplacée ou \
         supprimée ? Un registre qui décrit un monde qui n'existe plus ne garde \
         rien :\n{}",
        sans_site.join("\n")
    );
    for (f, n, statut) in REGISTRE {
        assert!(!statut.raison().trim().is_empty(), "{f}::{n} : raison vide");
        if let Statut::Invariant(i, _) = statut {
            assert!(["I1", "I2", "I3"].contains(i), "{f}::{n} : invariant {i}");
        }
    }
    // Anti-muet : le nombre de sites vus (W1-W30 de la fiche, plus
    // `write_closed_on`, l'écrivain neuf de la story). Un détecteur qui
    // cesserait de voir rendrait un registre « vert » vide.
    assert_eq!(sites.len(), 31, "sites vus :\n{}", detail.join("\n"));
}

/// Second inventaire (AC8, C-28-4-22) — chaque appelant du créateur de
/// règlement porte son propre statut.
#[test]
fn every_invoice_settlement_creator_is_classified() {
    let mut couples = BTreeSet::new();
    for (fichier, source) in sources_lues() {
        couples.extend(appels(
            &fichier,
            &source,
            "invoice_settlements::create_in_tx(",
        ));
    }
    let (non_classes, sans_site) = ecarts(&couples, APPELANTS_REGLEMENT);
    assert!(
        non_classes.is_empty(),
        "⛔ appelant(s) de `invoice_settlements::create_in_tx` non classé(s) \
         (Story 28-4a, AC8) — un règlement par virement vers un compte clos \
         doit y être refusé :\n{}",
        non_classes.join("\n")
    );
    assert!(
        sans_site.is_empty(),
        "appelant(s) du registre introuvable(s) :\n{}",
        sans_site.join("\n")
    );
    assert_eq!(couples.len(), 3, "trois créateurs de règlement connus");
}

/// T-a34 (AC8) — le volet inverse du diff, et l'extraction elle-même, sur un
/// cas construit : une entrée sans site est signalée, une écriture neuve non
/// classée aussi ; les formes larges du verbe sont vues, `FOR UPDATE` non.
#[test]
fn the_diff_reports_both_directions_on_a_built_case() {
    let source = r#"
        fn ecrit() {
            let _ = "INSERT IGNORE INTO bank_transactions (a) VALUES (?)";
            let _ = "UPDATE\n bank_accounts SET x = 1";
            let _ = "SELECT id FROM bank_accounts WHERE id = ? FOR UPDATE";
            // "DELETE FROM bank_imports" — commentaire : ignoré
        }
        fn lit() { let _ = "SELECT * FROM payment_batches"; }
        #[cfg(test)]
        mod tests { fn t() { let _ = "DELETE FROM bank_imports"; } }
    "#;
    let sites = sites_d_ecriture("x.rs", source);
    let extraits: Vec<&str> = sites.iter().map(|(_, _, e)| e.as_str()).collect();
    assert_eq!(
        extraits,
        ["INSERT bank_transactions", "UPDATE bank_accounts"],
        "{sites:?}"
    );
    assert!(sites.iter().all(|(_, n, _)| n == "ecrit"));
    let couples: BTreeSet<(String, String)> = sites
        .iter()
        .map(|(f, n, _)| (f.clone(), n.clone()))
        .collect();
    let registre = [(
        "x.rs",
        "disparue",
        Statut::SansCompteBancaire("cas construit"),
    )];
    let (non_classes, sans_site) = ecarts(&couples, &registre);
    assert_eq!(non_classes, ["x.rs::ecrit"]);
    assert_eq!(sans_site, ["x.rs::disparue"]);
}

/// T-a47 (AC6-bis) — assertion textuelle sur la route d'import : exactement
/// deux appels `bank_imports::create_with_transactions(`, chacun suivi de
/// `.await.map_err(import_create_error)` dans la source **aplatie** (blancs et
/// sauts de ligne retirés : `rustfmt` peut rompre la chaîne).
#[test]
fn both_import_calls_map_the_archived_account() {
    let chemin = racine_crates().join("kesh-api/src/routes/bank_imports.rs");
    let source = std::fs::read_to_string(chemin).expect("bank_imports.rs");
    let (_, masque) = decouper(&source);
    let tests = blocs_de_test(&masque);
    // Code de production seulement : blocs de test effacés, puis aplati.
    let mut production = masque.into_bytes();
    for (de, a) in tests {
        for x in production.iter_mut().take(a + 1).skip(de) {
            *x = b' ';
        }
    }
    let aplati: String = String::from_utf8(production)
        .expect("utf8")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let appel = "bank_imports::create_with_transactions(";
    let n = aplati.matches(appel).count();
    assert_eq!(n, 2, "deux appels attendus (CAMT et CSV), {n} trouvés");
    for (p, _) in aplati.match_indices(appel) {
        let suite = &aplati[p..];
        let fin = suite.find(';').expect("fin d'instruction");
        assert!(
            suite[..fin].contains(".await.map_err(import_create_error)?"),
            "un appel de create_with_transactions sans import_create_error : {}",
            &suite[..fin]
        );
    }
}
