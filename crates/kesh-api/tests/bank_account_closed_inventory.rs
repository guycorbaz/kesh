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
//! Troisième inventaire, **la fenêtre de la clôture** (Story 29-4a, #651,
//! AC5) : l'ensemble clos des fonctions qui portent une garde non
//! verrouillante du compte clos, chacune revérifiée sous verrou (variante et
//! position contrôlées) ou sans fenêtre par une raison écrite, au [`FENETRE`]
//! — voir la section qui lui est consacrée, en fin de fichier.
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
    /// clos, laissé sans garde pour réparer ce que la fenêtre de la clôture
    /// (#651, fermée en v0.13.4) a pu laisser passer jusqu'à la v0.13.3.
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
            "transaction pending dans l'état commité jusqu'au commit du lot ; et le pré-vol R1 refuse un compte clos",
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
        // Story 29-4a, revue de code P1 (E-1) : l'`INSERT` vit dans la
        // tentative rejouée, non plus dans le handler.
        "create_once",
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
            "annuler un lot generated laissé sur un compte clos par la fenêtre de la v0.13.3 (#651, fermée en v0.13.4) : le geste qui en fait sortir (C-28-4-23)",
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
            "transaction pending dans l'état commité jusqu'au commit du lot ; et le pré-vol R1 refuse un compte clos",
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

/// T-a47 (AC6-bis) — assertion textuelle sur la route d'import, dans la source
/// **aplatie** (blancs et sauts de ligne retirés : `rustfmt` peut rompre la
/// chaîne). Depuis la revue de code P1 de la 29-4a (E-3 = B-6) : exactement
/// **un** appel `bank_imports::create_with_transactions(`, dans l'aide
/// `create_import_or_roll_back`, dont la branche d'erreur annule
/// explicitement la transaction puis rend `import_create_error` ; et
/// exactement **deux** appels de l'aide (CAMT et CSV), chacun suivi de
/// `.await?`. Tue M29-29 (`rollback` retiré de la branche d'erreur).
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
    assert_eq!(
        n, 1,
        "un appel attendu (l'aide create_import_or_roll_back), {n} trouvés"
    );
    let p = aplati.find(appel).expect("appel");
    let suite = &aplati[p..];
    let fin = suite.find("}}}").expect("fin du match de l'aide");
    assert!(
        suite[..fin].contains("Err(e)=>{let_=tx.rollback().await;Err(import_create_error(e))"),
        "la branche d'erreur de l'aide doit annuler la transaction puis rendre \
         import_create_error : {}",
        &suite[..fin]
    );
    let aide = "create_import_or_roll_back(tx,";
    let m = aplati.matches(aide).count();
    assert_eq!(
        m, 2,
        "deux appels de l'aide attendus (CAMT et CSV), {m} trouvés"
    );
    for (q, _) in aplati.match_indices(aide) {
        let suite = &aplati[q..];
        let fin = suite.find(';').expect("fin d'instruction");
        assert!(
            suite[..fin].ends_with(").await?"),
            "un appel de l'aide sans `.await?` : {}",
            &suite[..fin]
        );
    }
}

/// T29b-7 (Story 29-4b, T6-bis, R-5 de la passe ciblée P2 de la 29-4a, #651) —
/// assertion textuelle sur `routes/bank_accounts.rs`, corps **aplatis** (blancs
/// retirés) : la conclusion commune `conclure` annule **explicitement** la
/// transaction sur erreur (`let _ = tx.rollback().await;` puis `Err(e)`), et
/// chacune des six tentatives rejouées (`create_once`, `update_once`,
/// `archive_once`, `close_once`, `reopen_once`, `patch_once`) se termine par
/// `conclure(tx, result).await`. Tue M29-35 (`rollback` retiré de `conclure`).
///
/// ⚠️ **Textuel, et c'est un choix** : sans `rollback` explicite, sqlx 0.8.6
/// (`Transaction::drop`, `sqlx-mysql` `start_rollback`) ne fait que **mettre en
/// file** le `ROLLBACK`, et la connexion revient au pool par une tâche lancée à
/// part qui l'envoie (`PoolConnection::drop` → `return_to_pool`, `ping`) : le
/// verrou est rendu **un peu plus tard**, non jamais. Un test qui sonderait le
/// verrou après la réponse ferait la course contre cette tâche — vert ou rouge
/// selon l'ordonnanceur. Que le verrou soit rendu **avant** la réponse n'a donc
/// pas de test dynamique : déclaré non couvert (fiche 29-4, T6-bis).
#[test]
fn bank_account_attempts_roll_back_explicitly() {
    let chemin = racine_crates().join("kesh-api/src/routes/bank_accounts.rs");
    let source = std::fs::read_to_string(chemin).expect("routes/bank_accounts.rs");
    let (_, masque) = decouper(&source);
    let fns = fonctions(&masque);
    let aplatir = |de: usize, a: usize| -> String {
        masque[de..=a]
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect()
    };
    let corps_de = |nom: &str| -> String {
        let trouves: Vec<_> = fns.iter().filter(|(n, _, _)| n == nom).collect();
        assert_eq!(trouves.len(), 1, "une seule fonction `{nom}`");
        aplatir(trouves[0].1, trouves[0].2)
    };
    let conclure = corps_de("conclure");
    assert!(
        conclure.contains("Err(e)=>{let_=tx.rollback().await;Err(e)}"),
        "la branche d'erreur de `conclure` doit annuler la transaction : {conclure}"
    );
    for tentative in [
        "create_once",
        "update_once",
        "archive_once",
        "close_once",
        "reopen_once",
        "patch_once",
    ] {
        assert!(
            corps_de(tentative).ends_with("conclure(tx,result).await}"),
            "{tentative} doit se conclure par `conclure(tx, result).await`"
        );
    }
}

// ===========================================================================
// Troisième inventaire — Story 29-4a (#651), AC5, étendu en 29-4b (AC8) : la
// fenêtre de la clôture
// ===========================================================================
//
// La garde « compte clos » de la Story 28-4 est une lecture **non
// verrouillante** : une écriture dont la vue est née avant le commit d'une
// clôture pouvait encore passer (L1, #651, jusqu'à la v0.13.3). La Story 29-4 la complète d'une
// **revérification verrouillante** (`bank_accounts::recheck_open_locked_in_tx`
// pour les annulations, `bank_accounts::recheck_open_unarchived_locked_in_tx`
// pour l'import et la création de lot). Cet inventaire tient, dans les deux
// sens, l'ensemble clos `G` des fonctions qui portent une garde non
// verrouillante — extrait depuis le **symptôme**, non depuis les raisons du
// registre principal —, et exige que chacune soit revérifiée (avec la bonne
// variante, à la bonne position), ou sans fenêtre par une raison écrite.
//
// ✅ Ce qu'il voit : la présence de l'appel et sa variante ; l'ensemble `G` ;
// l'ordre textuel de l'appel par rapport aux littéraux SQL de la fonction
// (positions de la copie masquée et de `Litteral.debut`, mêlées) — pour
// l'import et le lot, entre deux littéraux nommés ; pour les cinq annulations
// (Story 29-4b, AC8), après le dernier geste verrouillant et avant l'audit
// (`Position::EnDernier`).
//
// ⚠️ Ce qu'il ne voit pas (angles morts assumés) : que la revérification
// **s'exécute** (affaire des tests d'entrelacement, T29a-2, -3, -6, -7, -11,
// -12, T29b-1, T29b-2) ; une écriture faite par une fonction appelée
// **après** l'appel, que l'ordre textuel d'une seule fonction ne suit pas, ni
// un geste verrouillant appelé sous un nom absent de
// `GESTES_VERROUILLANTS_CODE` ; une lecture non
// verrouillante de `closed_on` dont le SQL ne porte pas littéralement `FROM
// bank_accounts` (requête assemblée, `format!`) ; une garde appelée sans le
// préfixe `bank_accounts::` (importée par `use`) ; les **pré-vols** de route sur
// le pool (`is_closed()` dans `routes/bank_imports.rs`,
// `routes/reconciliation.rs`, `routes/bank_accounts.rs`), hors transaction, que
// suit chacun une garde ou un invariant du registre principal.

/// La revérification appelée — Story 29-4 (#651, AC1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Variante {
    /// `recheck_open_locked_in_tx` : `closed_on` seul — les annulations, qui ne
    /// refusent jamais un compte archivé (il ne se rouvre pas).
    Clos,
    /// `recheck_open_unarchived_locked_in_tx` : archivé puis clos — l'import et
    /// la création de lot.
    ClosOuArchive,
}

impl Variante {
    /// Le nom appelé, qualifié comme les sites l'écrivent (la forme
    /// `super::bank_accounts::` le contient). Aucun des deux n'est préfixe de
    /// l'autre.
    fn appel(self) -> &'static str {
        match self {
            Variante::Clos => "bank_accounts::recheck_open_locked_in_tx(",
            Variante::ClosOuArchive => "bank_accounts::recheck_open_unarchived_locked_in_tx(",
        }
    }
}

/// Où l'appel doit se trouver dans la source de sa fonction.
#[derive(Debug)]
enum Position {
    /// L'import et le lot (29-4a) : **après** le (dernier) littéral SQL qui
    /// contient `apres` — l'`INSERT` à clé étrangère qui a posé le partagé
    /// du compte —, et **avant** le (premier) littéral qui contient `avant`
    /// s'il est donné. Fragments comparés sur le texte du littéral aplati
    /// (échappements neutralisés, blancs réduits, majuscules).
    Litteraux {
        apres: &'static str,
        avant: Option<&'static str>,
    },
    /// Les annulations (Story 29-4b, AC8, T29b-4) : en **dernière
    /// acquisition de verrou** — l'appel suit la dernière occurrence, dans le
    /// corps de la fonction, de tout geste qui prend un verrou (les jetons
    /// SQL de [`GESTES_VERROUILLANTS_SQL`], lus dans les **littéraux** ; les
    /// appels de [`GESTES_VERROUILLANTS_CODE`], lus dans la **copie
    /// masquée**), et précède `audit_log::insert_in_tx(` quand la fonction en
    /// porte un.
    EnDernier,
}

/// Les gestes verrouillants d'un littéral SQL, pour [`Position::EnDernier`]
/// (texte aplati, majuscules ; `UPDATE ` porte son espace, `FOR UPDATE` est
/// listé à part).
const GESTES_VERROUILLANTS_SQL: [&str; 5] = [
    "INSERT INTO",
    "UPDATE ",
    "DELETE FROM",
    "FOR UPDATE",
    "LOCK IN SHARE MODE",
];

/// Les appels verrouillants des annulations, pour [`Position::EnDernier`] :
/// la contre-passation (`reverse_in_tx(`, `reverse_owned_in_tx(`), le
/// règlement annulé de W9 (`cancel_settlement_in_tx(`) — la liste de la fiche
/// —, et la dissolution du lettrage de pièce, ajoutée au développement (elle
/// écrit, et précède toujours la contre-passation : sans effet sur l'issue,
/// elle ferme un angle mort de plus). ⚠️ **Angle mort** : un geste
/// verrouillant appelé sous un nom absent de cette liste, ou un SQL assemblé
/// hors littéral, n'est pas suivi.
const GESTES_VERROUILLANTS_CODE: [&str; 5] = [
    "reverse_in_tx(",
    "reverse_owned_in_tx(",
    "cancel_settlement_in_tx(",
    "dissolve_invoice_document_group_in_tx(",
    "dissolve_supplier_invoice_document_group_in_tx(",
];

/// Le statut d'une fonction de `G` au regard de la fenêtre de la clôture.
/// Depuis la Story 29-4b, aucune fenêtre n'est plus ouverte : la variante
/// transitoire `Ouverte` de la 29-4a est retirée avec ses derniers sites.
#[derive(Debug)]
enum Fenetre {
    /// La revérification est appelée dans la fonction, par cette variante, à
    /// cette position.
    Reverifiee(Variante, Position),
    /// Pas de fenêtre — fermée par construction : la raison (invariant ou
    /// verrou) est écrite. (Le `SansFenetre` de la fiche : `clippy` refuse une
    /// variante qui finit par le nom de son type.)
    Fermee(&'static str),
}

/// Les cinq annulations (W9, W24, W25, W28, W29) : leur revérification ne lit
/// **jamais** `archived` — un compte archivé ne se rouvre pas, refuser
/// l'annulation d'un de ses règlements la rendrait définitivement impossible.
/// Écrit en dur pour que la déclaration du registre ne puisse pas dériver.
const ANNULATIONS: [(&str, &str); 5] = [
    (
        "kesh-db/src/repositories/reconciliation_cancel.rs",
        "cancel_in_tx",
    ),
    (
        "kesh-db/src/repositories/bank_imports.rs",
        "delete_unreconciled_in_tx",
    ),
    (
        "kesh-db/src/repositories/invoice_settlements_write.rs",
        "cancel_settlement_in_tx",
    ),
    (
        "kesh-db/src/repositories/supplier_invoices.rs",
        "cancel_in_tx",
    ),
    (
        "kesh-db/src/repositories/supplier_invoices.rs",
        "cancel_settlement_in_tx",
    ),
];

/// Le troisième registre : une ligne par fonction de `G`.
const FENETRE: &[(&str, &str, Fenetre)] = &[
    (
        "kesh-db/src/repositories/bank_imports.rs",
        "create_with_transactions",
        Fenetre::Reverifiee(
            Variante::ClosOuArchive,
            Position::Litteraux {
                apres: "INSERT INTO bank_imports",
                avant: Some("INSERT INTO bank_transactions"),
            },
        ),
    ),
    (
        "kesh-db/src/repositories/payment_batches.rs",
        "create_batch",
        Fenetre::Reverifiee(
            Variante::ClosOuArchive,
            // Borne haute (revue de code P1, B-4) : avant le premier
            // `UPDATE` du lot, donc avant ses articles.
            Position::Litteraux {
                apres: "INSERT INTO payment_batches",
                avant: Some("UPDATE payment_batches SET msg_id"),
            },
        ),
    ),
    (
        "kesh-db/src/repositories/payment_batches.rs",
        "confirm_batch",
        Fenetre::Fermee(
            "chaque facture par pay_in_tx, compte lu sous FOR UPDATE ; et I2 (aucun lot generated sur un compte clos)",
        ),
    ),
    (
        "kesh-api/src/routes/reconciliation.rs",
        "unreject_once",
        Fenetre::Fermee(
            "I1 : la transaction rejetée est pending, la clôture refuse tant qu'elle existe",
        ),
    ),
    (
        // Story 29-1a (#232), entrée au rebase de la 29-4a sur `b4abae5d` :
        // la constante `EXAMINED_ACCOUNTS_SQL` lit `closed_on` sans verrou,
        // hors de toute fonction (C-29-4-29).
        "kesh-db/src/repositories/closing_checks.rs",
        "<hors fonction>",
        Fenetre::Fermee(
            "lecture seule : la préparation de la clôture d'un exercice liste les comptes à examiner, ce n'est pas une garde et rien n'est écrit — aucune fenêtre",
        ),
    ),
    // Story 29-4b (#651, AC6, AC8) — les cinq annulations : `closed_on` seul
    // (`Variante::Clos`, tenu aussi par (i bis)), en dernière acquisition.
    (
        "kesh-db/src/repositories/reconciliation_cancel.rs",
        "cancel_in_tx",
        Fenetre::Reverifiee(Variante::Clos, Position::EnDernier),
    ),
    (
        "kesh-db/src/repositories/bank_imports.rs",
        "delete_unreconciled_in_tx",
        Fenetre::Reverifiee(Variante::Clos, Position::EnDernier),
    ),
    (
        "kesh-db/src/repositories/invoice_settlements_write.rs",
        "cancel_settlement_in_tx",
        Fenetre::Reverifiee(Variante::Clos, Position::EnDernier),
    ),
    (
        "kesh-db/src/repositories/supplier_invoices.rs",
        "cancel_in_tx",
        Fenetre::Reverifiee(Variante::Clos, Position::EnDernier),
    ),
    (
        "kesh-db/src/repositories/supplier_invoices.rs",
        "cancel_settlement_in_tx",
        Fenetre::Reverifiee(Variante::Clos, Position::EnDernier),
    ),
];

/// Le texte d'un littéral, aplati pour la comparaison : échappements
/// neutralisés (la continuation de ligne `\` comprise), blancs réduits à une
/// espace, majuscules.
fn aplatir(texte: &str) -> String {
    neutraliser_echappements(texte)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_uppercase()
}

/// L'ensemble `G` : les fonctions, hors tests et hors commentaires, qui (a)
/// appellent une garde non verrouillante **qualifiée**
/// (`bank_accounts::ensure_open_in_tx(` ou `…ensure_open_unarchived_in_tx(`),
/// ou (b) portent un littéral qui lit `closed_on` `FROM bank_accounts` sans
/// `FOR UPDATE` ni `LOCK IN SHARE MODE` — hors `repositories/bank_accounts.rs`,
/// dont ces lectures sont les gardes elles-mêmes (vues par (a) chez leurs
/// appelants), les gestes du compte sur lui-même et les lectures de détail.
fn gardes_non_verrouillantes(sources: &[(String, String)]) -> BTreeSet<(String, String)> {
    let mut g = BTreeSet::new();
    for (fichier, source) in sources {
        for garde in [
            "bank_accounts::ensure_open_in_tx(",
            "bank_accounts::ensure_open_unarchived_in_tx(",
        ] {
            g.extend(appels(fichier, source, garde));
        }
        if fichier == "kesh-db/src/repositories/bank_accounts.rs" {
            continue;
        }
        let (litteraux, masque) = decouper(source);
        let tests = blocs_de_test(&masque);
        let fns = fonctions(&masque);
        for l in litteraux {
            if tests.iter().any(|(de, a)| l.debut >= *de && l.debut <= *a) {
                continue;
            }
            let t = aplatir(&l.texte);
            if t.contains("CLOSED_ON")
                && t.contains("FROM BANK_ACCOUNTS")
                && !t.contains("FOR UPDATE")
                && !t.contains("LOCK IN SHARE MODE")
            {
                let fonction = fonction_englobante(&fns, l.debut)
                    .map_or("<hors fonction>".to_string(), |(nom, _, _)| nom.clone());
                g.insert((fichier.clone(), fonction));
            }
        }
    }
    g
}

/// Un appel de revérification : fichier, fonction englobante, variante,
/// position dans la source, et bornes du corps de la fonction.
struct AppelReverification {
    fichier: String,
    fonction: String,
    variante: Variante,
    position: usize,
    corps: (usize, usize),
}

/// Les appels de revérification d'un ensemble de sources, hors tests et hors
/// commentaires (copie masquée).
fn appels_de_reverification(sources: &[(String, String)]) -> Vec<AppelReverification> {
    let mut out = Vec::new();
    for (fichier, source) in sources {
        let (_, masque) = decouper(source);
        let tests = blocs_de_test(&masque);
        let fns = fonctions(&masque);
        for variante in [Variante::Clos, Variante::ClosOuArchive] {
            let appel = variante.appel();
            for (p, _) in masque.match_indices(appel) {
                if tests.iter().any(|(de, a)| p >= *de && p <= *a) {
                    continue;
                }
                let Some((nom, de, a)) = fonction_englobante(&fns, p) else {
                    out.push(AppelReverification {
                        fichier: fichier.clone(),
                        fonction: "<hors fonction>".to_string(),
                        variante,
                        position: p,
                        corps: (p, p),
                    });
                    continue;
                };
                out.push(AppelReverification {
                    fichier: fichier.clone(),
                    fonction: nom.clone(),
                    variante,
                    position: p,
                    corps: (*de, *a),
                });
            }
        }
    }
    out
}

/// Toutes les fautes du troisième inventaire, pour un ensemble de sources et
/// un registre : (i) les couples (fonction, variante) appelés égalent ceux des
/// `Reverifiee` ; (i bis) une annulation ne déclare que `Variante::Clos` ;
/// (ii) `G` égale les fonctions du registre, dans les deux sens ; (iii) chaque
/// appel est à sa position, par rapport aux littéraux de sa fonction.
fn fautes_de_la_fenetre(
    sources: &[(String, String)],
    registre: &[(&str, &str, Fenetre)],
) -> Vec<String> {
    let mut fautes = Vec::new();
    let appels = appels_de_reverification(sources);

    // (i)
    let vus: BTreeSet<(String, String, Variante)> = appels
        .iter()
        .map(|a| (a.fichier.clone(), a.fonction.clone(), a.variante))
        .collect();
    let declares: BTreeSet<(String, String, Variante)> = registre
        .iter()
        .filter_map(|(f, n, s)| match s {
            Fenetre::Reverifiee(v, _) => Some((f.to_string(), n.to_string(), *v)),
            _ => None,
        })
        .collect();
    for (f, n, v) in vus.difference(&declares) {
        fautes.push(format!(
            "(i) {f}::{n} appelle la revérification {v:?} sans l'avoir déclarée au registre"
        ));
    }
    for (f, n, v) in declares.difference(&vus) {
        fautes.push(format!(
            "(i) {f}::{n} est déclarée revérifiée par {v:?}, et ne l'appelle pas"
        ));
    }

    // (i bis)
    for (f, n, s) in registre {
        if let Fenetre::Reverifiee(v, _) = s
            && ANNULATIONS.contains(&(f, n))
            && *v != Variante::Clos
        {
            fautes.push(format!(
                "(i bis) {f}::{n} est une annulation : seule Variante::Clos y est permise \
                 (un compte archivé ne se rouvre pas), déclaré {v:?}"
            ));
        }
    }

    // (ii)
    let g = gardes_non_verrouillantes(sources);
    let (non_classes, sans_garde) = ecarts_fenetre(&g, registre);
    for c in non_classes {
        fautes.push(format!(
            "(ii) {c} porte une garde non verrouillante du compte clos sans statut au registre \
             FENETRE"
        ));
    }
    for c in sans_garde {
        fautes.push(format!(
            "(ii) {c} figure au registre FENETRE sans garde non verrouillante"
        ));
    }

    // (iii)
    let decoupes: Vec<(String, Vec<lexique::Litteral>, String)> = sources
        .iter()
        .map(|(f, s)| {
            let (lits, masque) = decouper(s);
            (f.clone(), lits, masque)
        })
        .collect();
    for a in &appels {
        let Some((_, _, Fenetre::Reverifiee(_, pos))) = registre
            .iter()
            .find(|(f, n, _)| *f == a.fichier && *n == a.fonction)
        else {
            continue;
        };
        let Some((_, lits, masque)) = decoupes.iter().find(|(f, _, _)| *f == a.fichier) else {
            continue;
        };
        let dans_le_corps: Vec<(usize, String)> = lits
            .iter()
            .filter(|l| l.debut >= a.corps.0 && l.debut <= a.corps.1)
            .map(|l| (l.debut, aplatir(&l.texte)))
            .collect();
        let (apres, avant) = match pos {
            Position::Litteraux { apres, avant } => (*apres, *avant),
            Position::EnDernier => {
                fautes.extend(fautes_en_dernier(a, &dans_le_corps, masque));
                continue;
            }
        };
        let dernier_apres = dans_le_corps
            .iter()
            .filter(|(_, t)| t.contains(&aplatir(apres)))
            .map(|(d, _)| *d)
            .max();
        match dernier_apres {
            None => fautes.push(format!(
                "(iii) {}::{} : aucun littéral « {} » dans la fonction",
                a.fichier, a.fonction, apres
            )),
            Some(d) if a.position < d => fautes.push(format!(
                "(iii) {}::{} : la revérification précède « {} » — elle prendrait un verrou \
                 explicite avant l'`INSERT`, à une position que l'analyse n'a pas faite",
                a.fichier, a.fonction, apres
            )),
            Some(_) => {}
        }
        if let Some(avant) = avant {
            let premier = dans_le_corps
                .iter()
                .filter(|(_, t)| t.contains(&aplatir(avant)))
                .map(|(d, _)| *d)
                .min();
            match premier {
                None => fautes.push(format!(
                    "(iii) {}::{} : aucun littéral « {avant} » dans la fonction",
                    a.fichier, a.fonction
                )),
                Some(d) if a.position > d => fautes.push(format!(
                    "(iii) {}::{} : la revérification suit « {avant} »",
                    a.fichier, a.fonction
                )),
                Some(_) => {}
            }
        }
    }
    fautes
}

/// (iii) pour [`Position::EnDernier`] (Story 29-4b, AC8, T29b-4) : la
/// revérification `a` doit suivre le **dernier** geste verrouillant du corps —
/// jetons SQL dans les littéraux (`dans_le_corps`, positions
/// `Litteral.debut`), appels dans la copie masquée (`masque`, chaînes et
/// commentaires effacés), positions mêlées —, et précéder le premier
/// `audit_log::insert_in_tx(` du corps s'il y en a un. Un corps sans aucun
/// geste verrouillant est lui-même une faute : la règle n'y prouverait rien.
fn fautes_en_dernier(
    a: &AppelReverification,
    dans_le_corps: &[(usize, String)],
    masque: &str,
) -> Vec<String> {
    let mut fautes = Vec::new();
    let mut dernier: Option<(usize, &str)> = None;
    let mut garder = |p: usize, geste: &'static str| {
        if dernier.is_none_or(|(q, _)| p > q) {
            dernier = Some((p, geste));
        }
    };
    for (d, t) in dans_le_corps {
        for g in GESTES_VERROUILLANTS_SQL {
            if t.contains(g) {
                garder(*d, g);
            }
        }
    }
    let corps = &masque[a.corps.0..=a.corps.1];
    for g in GESTES_VERROUILLANTS_CODE {
        for (p, _) in corps.match_indices(g) {
            garder(a.corps.0 + p, g);
        }
    }
    match dernier {
        None => fautes.push(format!(
            "(iii) {}::{} : aucun geste verrouillant dans la fonction",
            a.fichier, a.fonction
        )),
        Some((p, g)) if a.position < p => fautes.push(format!(
            "(iii) {}::{} : la revérification précède « {} » — elle ne serait pas la \
             dernière acquisition de verrou",
            a.fichier,
            a.fonction,
            g.trim_end()
        )),
        Some(_) => {}
    }
    if let Some(q) = corps.find("audit_log::insert_in_tx(")
        && a.position > a.corps.0 + q
    {
        fautes.push(format!(
            "(iii) {}::{} : la revérification suit « audit_log::insert_in_tx( »",
            a.fichier, a.fonction
        ));
    }
    fautes
}

/// Le diff ensembliste de (ii), dans les deux sens : `(fonctions de G sans
/// statut, entrées du registre hors de G)`.
fn ecarts_fenetre(
    g: &BTreeSet<(String, String)>,
    registre: &[(&str, &str, Fenetre)],
) -> (Vec<String>, Vec<String>) {
    let classes: BTreeSet<(String, String)> = registre
        .iter()
        .map(|(f, n, _)| (f.to_string(), n.to_string()))
        .collect();
    (
        g.difference(&classes)
            .map(|(f, n)| format!("{f}::{n}"))
            .collect(),
        classes
            .difference(g)
            .map(|(f, n)| format!("{f}::{n}"))
            .collect(),
    )
}

/// T29a-5 (Story 29-4a, #651, AC5) — le troisième inventaire, sur les sources
/// de production : aucune faute, et des décomptes anti-muets (`G` à dix
/// sites — les neuf fonctions de la fiche et, depuis le rebase sur la 29-1a,
/// la constante `EXAMINED_ACCOUNTS_SQL` de `closing_checks.rs` —, sept appels
/// de revérification depuis la 29-4b : l'import et le lot, plus les cinq
/// annulations). T29b-4 (Story 29-4b, AC8) : les cinq annulations y sont
/// `Reverifiee(Variante::Clos, Position::EnDernier)`, et leur position est
/// contrôlée par (iii) — [`fautes_en_dernier`].
#[test]
fn every_unlocked_closed_account_guard_is_rechecked_or_justified() {
    let sources = sources_lues();
    let fautes = fautes_de_la_fenetre(&sources, FENETRE);
    assert!(
        fautes.is_empty(),
        "⛔ fenêtre de la clôture (Story 29-4, AC5, AC8) :\n{}",
        fautes.join("\n")
    );
    for (f, n, statut) in FENETRE {
        let raison = match statut {
            Fenetre::Fermee(r) => r,
            Fenetre::Reverifiee(_, Position::Litteraux { apres, .. }) => apres,
            Fenetre::Reverifiee(_, Position::EnDernier) => "en dernier",
        };
        assert!(!raison.trim().is_empty(), "{f}::{n} : raison vide");
    }
    for annulation in ANNULATIONS {
        assert!(
            FENETRE.iter().any(|(f, n, s)| (*f, *n) == annulation
                && matches!(s, Fenetre::Reverifiee(Variante::Clos, Position::EnDernier))),
            "l'annulation {annulation:?} doit être revérifiée en dernier (Variante::Clos)"
        );
    }
    // Anti-muet : un extracteur qui cesserait de voir rendrait un registre
    // « vert » vide.
    assert_eq!(
        gardes_non_verrouillantes(&sources).len(),
        10,
        "G : neuf fonctions au sol de la 29-4a, plus la constante de lecture de \
         closing_checks.rs (29-1a, entrée au rebase)"
    );
    assert_eq!(
        appels_de_reverification(&sources).len(),
        7,
        "sept revérifications depuis la 29-4b (import, lot, cinq annulations)"
    );
}

/// T29a-5, volet inverse — chaque assertion du troisième inventaire rougit sur
/// une source construite : une garde non verrouillante sans entrée (ii), une
/// entrée sans garde (ii), un appel avant l'`INSERT` (iii), une annulation qui
/// appelle la variante « archivé » (i), une annulation déclarée avec la
/// variante « archivé » (i bis), une revérification déclarée et absente (i) ;
/// et — revue de code P1 (B-3 = A-6) — les trois autres fautes de (iii) : une
/// revérification qui **suit** la borne `avant`, un littéral `apres` absent, un
/// littéral `avant` absent, chacune seule sur une source dont le cas de
/// contrôle conforme ne rend **aucune** faute.
/// Sans quoi un extracteur qui ne voit rien ferait passer le registre à vide.
#[test]
fn the_window_inventory_reports_each_fault_on_built_cases() {
    let import = (
        "kesh-db/src/repositories/bank_imports.rs".to_string(),
        r#"
        pub async fn create_with_transactions() {
            super::bank_accounts::ensure_open_unarchived_in_tx(tx, c, b).await?;
            super::bank_accounts::recheck_open_unarchived_locked_in_tx(tx, c, b).await?;
            let _ = "INSERT INTO bank_imports (a) VALUES (?)";
            let _ = "INSERT INTO bank_transactions (a) VALUES (?)";
        }
        fn nouvelle() { bank_accounts::ensure_open_in_tx(tx, c, b).await?; }
        fn lecture() { let _ = "SELECT closed_on FROM bank_accounts \
                               WHERE id = ?"; }
        fn verrouillee() { let _ = "SELECT closed_on FROM bank_accounts WHERE id = ? FOR UPDATE"; }
        #[cfg(test)]
        mod tests { fn t() { bank_accounts::ensure_open_in_tx(tx, c, b); } }
        "#
        .to_string(),
    );
    let annulation = (
        "kesh-db/src/repositories/reconciliation_cancel.rs".to_string(),
        r#"
        pub async fn cancel_in_tx() {
            super::bank_accounts::ensure_open_in_tx(tx, c, b).await?;
            // bank_accounts::recheck_open_locked_in_tx( — commentaire : ignoré
            let _ = "UPDATE bank_transactions SET status = 'pending' WHERE id = ?";
            super::bank_accounts::recheck_open_unarchived_locked_in_tx(tx, c, b).await?;
        }
        "#
        .to_string(),
    );
    let sources = vec![import, annulation];
    let registre = [
        (
            "kesh-db/src/repositories/bank_imports.rs",
            "create_with_transactions",
            Fenetre::Reverifiee(
                Variante::ClosOuArchive,
                Position::Litteraux {
                    apres: "INSERT INTO bank_imports",
                    avant: Some("INSERT INTO bank_transactions"),
                },
            ),
        ),
        (
            "kesh-db/src/repositories/reconciliation_cancel.rs",
            "cancel_in_tx",
            Fenetre::Reverifiee(Variante::Clos, Position::EnDernier),
        ),
        (
            "kesh-db/src/repositories/bank_imports.rs",
            "disparue",
            Fenetre::Fermee("cas construit"),
        ),
    ];

    let g = gardes_non_verrouillantes(&sources);
    let noms: Vec<&str> = g.iter().map(|(_, n)| n.as_str()).collect();
    assert_eq!(
        noms,
        [
            "create_with_transactions",
            "lecture",
            "nouvelle",
            "cancel_in_tx"
        ],
        "G : appels qualifiés et littéral non verrouillant, ni le verrouillé ni le test"
    );

    let fautes = fautes_de_la_fenetre(&sources, &registre);
    let attendues = [
        "(i) kesh-db/src/repositories/reconciliation_cancel.rs::cancel_in_tx appelle la revérification ClosOuArchive",
        "(i) kesh-db/src/repositories/reconciliation_cancel.rs::cancel_in_tx est déclarée revérifiée par Clos",
        "(ii) kesh-db/src/repositories/bank_imports.rs::nouvelle porte une garde",
        "(ii) kesh-db/src/repositories/bank_imports.rs::lecture porte une garde",
        "(ii) kesh-db/src/repositories/bank_imports.rs::disparue figure au registre",
        "(iii) kesh-db/src/repositories/bank_imports.rs::create_with_transactions : la revérification précède « INSERT INTO bank_imports »",
    ];
    for a in attendues {
        assert!(
            fautes.iter().any(|f| f.starts_with(a)),
            "faute attendue absente : {a}\nfautes : {fautes:#?}"
        );
    }
    assert_eq!(fautes.len(), attendues.len(), "fautes : {fautes:#?}");

    // (i bis), sur la déclaration seule.
    let declare_archive = [(
        "kesh-db/src/repositories/reconciliation_cancel.rs",
        "cancel_in_tx",
        Fenetre::Reverifiee(Variante::ClosOuArchive, Position::EnDernier),
    )];
    let fautes = fautes_de_la_fenetre(&sources[1..], &declare_archive);
    assert!(
        fautes.iter().any(|f| f.starts_with("(i bis)")),
        "(i bis) attendue : {fautes:#?}"
    );

    // (iii), les bornes (revue de code P1, B-3 = A-6) : une source par
    // position, une entrée par borne, chaque cas rendant **exactement** la
    // faute attendue — et le cas de contrôle conforme, aucune.
    let fichier = "kesh-db/src/repositories/bank_imports.rs";
    let source = |corps: &str| {
        vec![(
            fichier.to_string(),
            format!(
                "pub async fn create_with_transactions() {{\n\
                 super::bank_accounts::ensure_open_unarchived_in_tx(tx, c, b).await?;\n\
                 {corps}\n}}\n"
            ),
        )]
    };
    let entre = source(
        "let _ = \"INSERT INTO bank_imports (a) VALUES (?)\";\n\
         super::bank_accounts::recheck_open_unarchived_locked_in_tx(tx, c, b).await?;\n\
         let _ = \"INSERT INTO bank_transactions (a) VALUES (?)\";",
    );
    let apres_les_transactions = source(
        "let _ = \"INSERT INTO bank_imports (a) VALUES (?)\";\n\
         let _ = \"INSERT INTO bank_transactions (a) VALUES (?)\";\n\
         super::bank_accounts::recheck_open_unarchived_locked_in_tx(tx, c, b).await?;",
    );
    let entree = |apres: &'static str, avant: Option<&'static str>| {
        [(
            fichier,
            "create_with_transactions",
            Fenetre::Reverifiee(
                Variante::ClosOuArchive,
                Position::Litteraux { apres, avant },
            ),
        )]
    };
    let conforme = || {
        entree(
            "INSERT INTO bank_imports",
            Some("INSERT INTO bank_transactions"),
        )
    };
    let exiger = |nom: &str,
                  sources: &[(String, String)],
                  registre: &[(&str, &str, Fenetre)],
                  attendues: Vec<String>| {
        assert_eq!(
            fautes_de_la_fenetre(sources, registre),
            attendues,
            "cas {nom}"
        );
    };
    exiger("contrôle", &entre, &conforme(), vec![]);
    exiger(
        "suit « avant »",
        &apres_les_transactions,
        &conforme(),
        vec![format!(
            "(iii) {fichier}::create_with_transactions : la revérification suit « INSERT INTO bank_transactions »"
        )],
    );
    exiger(
        "littéral « apres » absent",
        &entre,
        &entree(
            "INSERT INTO lignes_absentes",
            Some("INSERT INTO bank_transactions"),
        ),
        vec![format!(
            "(iii) {fichier}::create_with_transactions : aucun littéral « INSERT INTO lignes_absentes » dans la fonction"
        )],
    );
    exiger(
        "littéral « avant » absent",
        &entre,
        &entree("INSERT INTO bank_imports", Some("DELETE FROM absente")),
        vec![format!(
            "(iii) {fichier}::create_with_transactions : aucun littéral « DELETE FROM absente » dans la fonction"
        )],
    );
}

/// T29b-4, volet inverse (Story 29-4b, #651, AC8) — la règle « en dernière
/// acquisition de verrou » des annulations rougit sur chaque source construite
/// qui la viole, et sur elle seule : une revérification qui précède un
/// **littéral** verrouillant (`DELETE FROM`, la forme de W24 ; `UPDATE `,
/// celle de W28 et W29), un **appel** verrouillant (`reverse_owned_in_tx(`,
/// `cancel_settlement_in_tx(`, la forme de W9 et W25), qui **suit** l'audit,
/// ou un corps sans aucun geste verrouillant ; le cas de contrôle conforme ne
/// rend aucune faute — un commentaire et une chaîne qui nomment un **appel**
/// verrouillant après la revérification n'y comptent pas (copie masquée ; les
/// jetons SQL, eux, se lisent dans les littéraux). Tue, avec T29a-5, M29-14 à
/// M29-18.
#[test]
fn the_last_acquisition_rule_reports_each_fault_on_built_cases() {
    let fichier = "kesh-db/src/repositories/supplier_invoices.rs";
    let source = |corps: &str| {
        vec![(
            fichier.to_string(),
            format!(
                "pub async fn cancel_in_tx() {{\n\
                 super::bank_accounts::ensure_open_in_tx(tx, c, b).await?;\n\
                 {corps}\n}}\n"
            ),
        )]
    };
    let registre = [(
        fichier,
        "cancel_in_tx",
        Fenetre::Reverifiee(Variante::Clos, Position::EnDernier),
    )];
    let faute = |suite: &str| format!("(iii) {fichier}::cancel_in_tx : {suite}");
    let recheck = "super::bank_accounts::recheck_open_locked_in_tx(tx, c, b).await?;";
    let cas: [(&str, String, Vec<String>); 7] = [
        (
            "contrôle",
            format!(
                "let _ = \"SELECT id FROM t WHERE id = ? FOR UPDATE\";\n\
                 journal_entries::reverse_owned_in_tx(tx, c, e, u, x).await?;\n\
                 let _ = \"UPDATE supplier_invoices SET a = 1 WHERE id = ?\";\n\
                 {recheck}\n\
                 // reverse_in_tx( — commentaire : ignoré\n\
                 let _ = \"reverse_owned_in_tx( dans une chaîne : ignoré\";\n\
                 audit_log::insert_in_tx(tx, e).await?;"
            ),
            vec![],
        ),
        (
            "précède un UPDATE",
            format!(
                "journal_entries::reverse_owned_in_tx(tx, c, e, u, x).await?;\n\
                 {recheck}\n\
                 let _ = \"UPDATE supplier_invoices SET a = 1 WHERE id = ?\";"
            ),
            vec![faute(
                "la revérification précède « UPDATE » — elle ne serait pas la dernière \
                 acquisition de verrou",
            )],
        ),
        (
            "précède un DELETE FROM",
            format!(
                "let _ = \"SELECT id FROM t WHERE id = ? FOR UPDATE\";\n\
                 {recheck}\n\
                 let _ = \"DELETE FROM bank_imports WHERE id = ?\";"
            ),
            vec![faute(
                "la revérification précède « DELETE FROM » — elle ne serait pas la dernière \
                 acquisition de verrou",
            )],
        ),
        (
            "précède la contre-passation",
            format!(
                "let _ = \"UPDATE t SET a = 1 WHERE id = ?\";\n\
                 {recheck}\n\
                 journal_entries::reverse_owned_in_tx(tx, c, e, u, x).await?;"
            ),
            vec![faute(
                "la revérification précède « reverse_owned_in_tx( » — elle ne serait pas la \
                 dernière acquisition de verrou",
            )],
        ),
        (
            "précède le règlement annulé",
            format!(
                "let _ = \"UPDATE t SET a = 1 WHERE id = ?\";\n\
                 {recheck}\n\
                 invoice_settlements_write::cancel_settlement_in_tx(tx, c, i, s, u).await?;"
            ),
            vec![faute(
                "la revérification précède « cancel_settlement_in_tx( » — elle ne serait pas \
                 la dernière acquisition de verrou",
            )],
        ),
        (
            "suit l'audit",
            format!(
                "let _ = \"UPDATE t SET a = 1 WHERE id = ?\";\n\
                 audit_log::insert_in_tx(tx, e).await?;\n\
                 {recheck}"
            ),
            vec![faute("la revérification suit « audit_log::insert_in_tx( »")],
        ),
        (
            "aucun geste verrouillant",
            format!("let _ = \"SELECT id FROM t WHERE id = ?\";\n{recheck}"),
            vec![faute("aucun geste verrouillant dans la fonction")],
        ),
    ];
    for (nom, corps, attendues) in cas {
        assert_eq!(
            fautes_de_la_fenetre(&source(&corps), &registre),
            attendues,
            "cas {nom}"
        );
    }
}

/// T29b-4, volet inverse jeton par jeton (Story 29-4b, #651, revue de code P1,
/// B-1) — chacune des dix entrées de [`GESTES_VERROUILLANTS_SQL`] et de
/// [`GESTES_VERROUILLANTS_CODE`] est la cible d'un cas construit : un geste
/// portant ce seul jeton, placé **après** la revérification, doit rendre
/// exactement la faute « précède « <jeton> » ». La table des cas est écrite
/// ici, à part des listes : retirer un jeton d'une liste laisse son cas sans
/// geste reconnu (« aucun geste verrouillant »), donc rouge ; ajouter un jeton
/// sans cas rougit le contrôle de couverture. T29a-5 ne le pouvait pas : chaque
/// revérification de production y est déjà la dernière.
#[test]
fn the_last_acquisition_rule_sees_each_locking_token() {
    let fichier = "kesh-db/src/repositories/supplier_invoices.rs";
    let registre = [(
        fichier,
        "cancel_in_tx",
        Fenetre::Reverifiee(Variante::Clos, Position::EnDernier),
    )];
    let recheck = "super::bank_accounts::recheck_open_locked_in_tx(tx, c, b).await?;";
    // (jeton tel que la faute le nomme, littéral qui le porte seul)
    let litteraux: [(&str, &str); 5] = [
        ("INSERT INTO", "INSERT INTO t (a) VALUES (?)"),
        ("UPDATE", "UPDATE t SET a = 1 WHERE id = ?"),
        ("DELETE FROM", "DELETE FROM t WHERE id = ?"),
        ("FOR UPDATE", "SELECT a FROM t WHERE id = ? FOR UPDATE"),
        (
            "LOCK IN SHARE MODE",
            "SELECT a FROM t WHERE id = ? LOCK IN SHARE MODE",
        ),
    ];
    let appels: [&str; 5] = [
        "reverse_in_tx(",
        "reverse_owned_in_tx(",
        "cancel_settlement_in_tx(",
        "dissolve_invoice_document_group_in_tx(",
        "dissolve_supplier_invoice_document_group_in_tx(",
    ];
    let cas = litteraux
        .iter()
        .map(|(jeton, geste)| (*jeton, format!("let _ = \"{geste}\";")))
        .chain(
            appels
                .iter()
                .map(|appel| (*appel, format!("journal_entries::{appel}tx, c).await?;"))),
        );
    let mut joues = 0;
    for (jeton, geste) in cas {
        let source = vec![(
            fichier.to_string(),
            format!(
                "pub async fn cancel_in_tx() {{\n\
                 super::bank_accounts::ensure_open_in_tx(tx, c, b).await?;\n\
                 let _ = \"SELECT id FROM t WHERE id = ?\";\n\
                 {recheck}\n\
                 {geste}\n}}\n"
            ),
        )];
        assert_eq!(
            fautes_de_la_fenetre(&source, &registre),
            vec![format!(
                "(iii) {fichier}::cancel_in_tx : la revérification précède « {jeton} » — elle \
                 ne serait pas la dernière acquisition de verrou"
            )],
            "jeton « {jeton} »"
        );
        joues += 1;
    }
    assert_eq!(joues, 10, "décompte anti-muet des cas joués");
    // Couverture, après les cas (un jeton retiré rougit d'abord par son cas) :
    // chaque entrée des deux listes a son cas, et réciproquement.
    assert_eq!(
        GESTES_VERROUILLANTS_SQL
            .iter()
            .map(|g| g.trim_end())
            .collect::<BTreeSet<_>>(),
        litteraux.iter().map(|(j, _)| *j).collect::<BTreeSet<_>>(),
        "un jeton SQL sans cas construit (ou un cas sans jeton)"
    );
    assert_eq!(
        GESTES_VERROUILLANTS_CODE
            .iter()
            .copied()
            .collect::<BTreeSet<_>>(),
        appels.iter().copied().collect::<BTreeSet<_>>(),
        "un appel verrouillant sans cas construit (ou un cas sans appel)"
    );
}
