//! Registre des routes mutantes et sa garde — Story 25-1b (AC 11).
//!
//! # Ce que ce test établit, et ce qu'il n'établit PAS
//!
//! ⛔ **Il ne vérifie PAS qu'une route est réellement tracée.** Le traçage se
//! fait au repository, parfois à trois appels de distance du handler, et aucune
//! analyse statique raisonnable ne le suit. Le contenu des traces est l'affaire
//! des tests par route (`users_e2e`, `contact_persons_e2e`, …).
//!
//! ✅ **Il vérifie que toute route mutante a été EXAMINÉE.** C'est une propriété
//! plus faible, et la seule qui soit décidable — mais elle ferme le mode
//! d'échec le plus coûteux : une route ajoutée demain, qui n'écrirait aucune
//! trace **sans que rien ne le signale**.
//!
//! # Pourquoi un diff ensembliste et non un compteur
//!
//! ⚠️ Comparer des NOMBRES ne détecterait pas une route retirée pendant qu'une
//! autre est ajoutée : le compte resterait égal et la dérive invisible. Le test
//! `admin_pat_denied_e2e` peut compter, lui, parce qu'il opère sur un bloc clos
//! entre marqueurs ; les 109 routes sont réparties dans tout le fichier.
//!
//! # Deux fichiers, deux volets
//!
//! L'ensemble clos de l'INVENTAIRE est celui de `lib.rs` (109 routes) ; celui du
//! REGISTRE est plus large (112), car trois routes mutantes vivent dans
//! `routes/test_endpoints.rs` et sont montées par un `nest()`. D'où :
//!
//! - volet « route absente du registre » → sur les **deux** fichiers, faute de
//!   quoi une quatrième route de `test_endpoints.rs` n'alerterait personne ;
//! - volet « entrée du registre absente du code » → sur les deux également,
//!   chaque entrée nommant le fichier dont elle provient.

use std::collections::{BTreeMap, BTreeSet};

/// Statut d'examen d'une route mutante au regard du journal d'audit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    /// Une entrée d'audit est écrite sur son chemin — dans la route ou dans le
    /// repository, parfois plus bas.
    Traced,
    /// Non tracée, et c'est un **angle mort assumé** : la justification porte le
    /// numéro de l'issue qui le suit. *Une justification sans suivi est un
    /// abandon déguisé.*
    Exempt(&'static str),
    /// Route mutante qui ne mute rien — rien à auditer.
    NoMatter(&'static str),
}

use Status::{Exempt, NoMatter, Traced};

/// Les routes mutantes de `crates/kesh-api/src/lib.rs`, par identité
/// `(verbe, module::handler)`.
#[rustfmt::skip]
const LIB_ROUTES: &[(&str, &str, Status)] = &[
    ("post", "users::create_user", Traced),
    ("put", "users::update_user", Traced),
    ("put", "users::disable_user", Traced),
    ("put", "users::reset_password", Traced),
    ("put", "company_invoice_settings::update_invoice_settings", Traced),
    ("post", "vat::create_vat_rate", Traced),
    ("put", "vat::update_vat_rate", Traced),
    ("delete", "vat::deactivate_vat_rate", Traced),
    ("put", "company_dunning_settings::update_dunning_settings", Traced),
    ("post", "dunning_levels::create_dunning_level", Traced),
    ("put", "dunning_levels::update_dunning_level", Traced),
    ("delete", "dunning_levels::delete_dunning_level", Traced),
    ("delete", "invoices::delete_invoice", Traced),
    ("post", "dunning_reminders::cancel_reminder", Traced),
    ("post", "admin::full_import", Traced),
    ("put", "email_templates::update_email_template", Traced),
    ("delete", "email_templates::restore_email_template_default", Traced),
    ("put", "companies::update_company_email", Traced),
    ("put", "companies::update_company_contact_details", Traced),
    ("post", "fiscal_years::reopen_fiscal_year", Traced),
    ("post", "companies::unlock_company_books", Traced),
    ("post", "onboarding::reset", Exempt("issue #434 — toute la séquence d'installation est non tracée")),
    ("post", "accounts::create_account", Traced),
    ("put", "accounts::update_account", Traced),
    ("put", "accounts::archive_account", Traced),
    ("put", "accounts::reactivate_account", Traced),
    ("post", "journal_entries::create_journal_entry", Traced),
    ("put", "journal_entries::update_journal_entry", NoMatter("ne mute rien : le corps n'est pas désérialisé, le handler rend 404 ou 409 ENTRY_IS_POSTED")),
    ("delete", "journal_entries::delete_journal_entry", Traced),
    ("post", "companies::lock_company_books", Traced),
    ("post", "journal_entries::reverse_journal_entry", Traced),
    ("post", "opening_balances::generate_opening_balances", Traced),
    ("post", "contacts::create_contact", Traced),
    ("put", "contacts::update_contact", Traced),
    ("put", "contacts::archive_contact", Traced),
    ("post", "contact_persons::create_person", Traced),
    ("put", "contact_persons::update_person", Traced),
    ("delete", "contact_persons::delete_person", Traced),
    ("post", "products::create_product", Traced),
    ("put", "products::update_product", Traced),
    ("put", "products::archive_product", Traced),
    ("post", "invoices::create_invoice", Traced),
    ("put", "invoices::update_invoice", Traced),
    ("post", "credit_notes::create_credit_note", Traced),
    ("post", "supplier_invoices::create_supplier_invoice", Traced),
    ("post", "supplier_invoices::pay_supplier_invoice", Traced),
    ("post", "supplier_invoices::cancel_supplier_invoice", Traced),
    ("post", "supplier_invoices::scan_qr_supplier_invoice", NoMatter("parsing pur du payload SPC, aucun accès base")),
    ("post", "imported_supplier_invoices::post_inbox_import", Traced),
    ("post", "imported_supplier_invoices::complete_import", Traced),
    ("post", "imported_supplier_invoices::discard_import", Traced),
    ("post", "payment_batches::create_payment_batch", Traced),
    ("post", "payment_batches::confirm_payment_batch", Traced),
    ("post", "payment_batches::cancel_payment_batch", Traced),
    ("post", "invoices::validate_invoice_handler", Traced),
    ("post", "invoices::unvalidate_invoice_handler", Traced),
    ("post", "invoices::settle_invoice_handler", Traced),
    ("post", "invoices::cancel_invoice_settlement_handler", Traced),
    ("post", "supplier_invoices::cancel_supplier_invoice_settlement", Traced),
    ("put", "dunning_reminders::pause_dunning", Traced),
    ("put", "dunning_reminders::resume_dunning", Traced),
    ("post", "dunning_reminders::record_manual_reminder", Traced),
    ("post", "invoice_email::send_reminder", Traced),
    ("post", "invoice_email::send_reminder_batch", Traced),
    ("post", "invoice_email::send_invoice_email", Traced),
    ("post", "fiscal_years::create_fiscal_year", Traced),
    ("put", "fiscal_years::update_fiscal_year", Traced),
    ("post", "fiscal_years::close_fiscal_year", Traced),
    ("post", "bank_imports::preview", NoMatter("parse et valide sans persistance — aucune mutation")),
    ("post", "bank_imports::create", Traced),
    ("post", "bank_profiles::create", Traced),
    ("put", "bank_profiles::update", Traced),
    ("delete", "bank_profiles::delete", Traced),
    ("post", "reconciliation::post_accept", Traced),
    ("post", "reconciliation::post_reject", Traced),
    ("post", "reconciliation::post_manual", Traced),
    ("post", "reconciliation::post_split", Traced),
    // Story 25-3-b (#418) — `reconciliation.cancelled`.
    ("post", "reconciliation::post_cancel_reconciliation", Traced),
    ("post", "bank_accounts::create_bank_account", Traced),
    ("patch", "bank_accounts::patch_bank_account_journal_link", Traced),
    ("put", "bank_accounts::update_bank_account", Traced),
    ("delete", "bank_accounts::archive_bank_account", Traced),
    ("post", "reconciliation_rules::post_create", Traced),
    ("patch", "reconciliation_rules::patch", Traced),
    ("delete", "reconciliation_rules::delete", Traced),
    ("post", "api_keys::create", Traced),
    ("delete", "api_keys::revoke", Traced),
    ("post", "projects::create_project", Traced),
    ("put", "projects::update_project", Traced),
    ("post", "projects::archive_project", Traced),
    ("post", "projects::unarchive_project", Traced),
    ("put", "auth::change_password", Exempt("issue #435 — change_password, login, logout, refresh")),
    ("put", "profile::set_mode", Traced),
    ("post", "onboarding::set_language", Exempt("issue #434 — toute la séquence d'installation est non tracée")),
    ("post", "onboarding::set_mode", Exempt("issue #434 — toute la séquence d'installation est non tracée")),
    ("post", "onboarding::seed_demo", Exempt("issue #434 — toute la séquence d'installation est non tracée")),
    ("post", "onboarding::start_production", Exempt("issue #434 — toute la séquence d'installation est non tracée")),
    ("post", "onboarding::set_org_type", Exempt("issue #434 — toute la séquence d'installation est non tracée")),
    ("post", "onboarding::set_accounting_language", Exempt("issue #434 — toute la séquence d'installation est non tracée")),
    ("post", "onboarding::set_coordinates", Exempt("issue #434 — toute la séquence d'installation est non tracée")),
    ("post", "onboarding::set_bank_account", Exempt("issue #434 — toute la séquence d'installation est non tracée")),
    ("post", "onboarding::skip_bank", Exempt("issue #434 — toute la séquence d'installation est non tracée")),
    ("post", "onboarding::finalize", Exempt("issue #434 — toute la séquence d'installation est non tracée")),
    ("post", "auth::login", Exempt("issue #435 — change_password, login, logout, refresh")),
    ("post", "auth::logout", Exempt("issue #435 — change_password, login, logout, refresh")),
    ("post", "auth::refresh", Exempt("issue #435 — change_password, login, logout, refresh")),
    ("post", "setup::create_admin", Traced),
    ("post", "auth::forgot_password", Traced),
    ("post", "auth::reset_password", Traced),];

/// Les routes mutantes de `crates/kesh-api/src/routes/test_endpoints.rs`,
/// montées par le `nest("/api/v1/_test", …)` de `lib.rs` et conditionnées au
/// mode test.
#[rustfmt::skip]
const TEST_ENDPOINT_ROUTES: &[(&str, &str, Status)] = &[
    ("post", "/seed", Exempt("chemin de test, jamais monté en production — efface delibérément la base")),
    ("post", "/reset", Exempt("chemin de test, jamais monté en production")),
    ("post", "/password-reset-token", Exempt("chemin de test, jamais monté en production")),
];

/// Extrait les routes mutantes d'un source par leur identité, **avec le nombre
/// d'occurrences de chacune**.
///
/// ⛔ **Le compte n'est pas décoratif** : deux routes différentes peuvent
/// partager le même couple `(verbe, handler)` — un alias. L'ensemble extrait
/// serait alors identique à celui d'avant l'ajout, et le diff resterait **vert
/// sur une route mutante que personne n'a examinée**. C'est le faux vert que
/// l'AC 11 nommait, et il a été vérifié empiriquement en passe 1 de revue.
fn extract_counted(source: &str, pattern_prefix: &str) -> BTreeMap<(String, String), usize> {
    let mut out: BTreeMap<(String, String), usize> = BTreeMap::new();
    for verb in ["post", "put", "delete", "patch"] {
        let needle = format!("{verb}(");
        let mut rest = source;
        while let Some(pos) = rest.find(&needle) {
            let after = &rest[pos + needle.len()..];
            if let Some(close) = after.find(')') {
                let arg = &after[..close];
                if arg.starts_with(pattern_prefix) {
                    *out.entry((
                        verb.to_string(),
                        arg.trim_start_matches(pattern_prefix).to_string(),
                    ))
                    .or_insert(0) += 1;
                }
            }
            rest = &rest[pos + needle.len()..];
        }
    }
    out
}

#[test]
fn every_mutating_route_of_lib_is_in_the_registry() {
    // ⚠️ **La coupe tombe au DÉBUT du bloc d'exemple commenté**, non à sa fin.
    //
    // Le marqueur employé d'abord — `// let app = build_router` — est la
    // DERNIÈRE ligne de ce bloc : les lignes précédentes, dont un
    // `.route(…, post(...))` d'exemple, restaient dans la source balayée. Elles
    // n'étaient inoffensives que grâce au filtre de préfixe, c'est-à-dire par
    // accident. On coupe donc à l'en-tête du bloc, ce qui rend l'inventaire des
    // sites non lus **vide** plutôt que d'avoir à y accueillir un faux site.
    let source = include_str!("../src/lib.rs");
    let source = source
        .split("// NOTE: les stories futures ajouteront leurs routes protégées")
        .next()
        .expect("le fichier n'est pas vide");
    assert!(
        !source.contains("// let app = build_router"),
        "⛔ la coupe doit précéder le bloc d'exemple commenté en entier — si ce \
         marqueur subsiste, l'en-tête du bloc a été réécrit et la coupe est \
         retombée trop tard"
    );

    // ⛔ **L'INVENTAIRE DES SITES QUE L'EXTRACTEUR NE SAIT PAS LIRE.**
    //
    // L'extraction ci-dessous énumère **une forme qui marche** —
    // `post(routes::module::handler)`. Une route écrite autrement, par exemple
    // `post(users::create_widget)` après un `use crate::routes::users;`, ne
    // serait **pas extraite** : trois ensembles vides, test vert, et personne
    // n'aurait examiné la route. La passe 1 croyait fermer ce faux vert en
    // traitant les alias ; elle l'avait seulement déplacé.
    //
    // La § *Inventorier les sites NON RÉSOLUS* du `CLAUDE.md` dit quoi faire :
    // ne pas énumérer les formes qui marchent, mais inventorier **l'ensemble
    // clos de celles qui ne résolvent pas**, et exiger que chacune soit soit
    // résolue, soit écrite comme angle mort assumé.
    let non_resolus: Vec<(String, String)> = extract_counted(source, "")
        .into_keys()
        .filter(|(_, arg)| !arg.starts_with("routes::"))
        .collect();
    assert_eq!(
        non_resolus,
        Vec::<(String, String)>::new(),
        "⛔ Site(s) de montage de route que l'extracteur du registre NE SAIT PAS \
         lire : {non_resolus:?}\n\n\
         Soit la route est écrite `post(routes::module::handler)` comme toutes les \
         autres, soit cet inventaire doit l'accueillir explicitement comme angle \
         mort assumé. ⚠️ Ne PAS élargir le filtre d'extraction sans élargir aussi \
         cet inventaire : c'est lui qui empêche une forme imprévue de passer."
    );

    let counted = extract_counted(source, "routes::");

    // ⛔ Un alias — deux routes partageant verbe et handler — rendrait l'ensemble
    // extrait identique à celui d'avant l'ajout, et ce test vert sur une route
    // mutante jamais examinée. On échoue donc BRUYAMMENT, au lieu de compter sur
    // une unicité que rien n'impose.
    let aliases: Vec<_> = counted.iter().filter(|(_, n)| **n > 1).collect();
    assert!(
        aliases.is_empty(),
        "⛔ Deux routes de lib.rs partagent le même couple (verbe, handler) : \
         {aliases:?}\n\n\
         L'identité du registre cesse alors d'être discriminante, et une route \
         mutante pourrait s'ajouter sans que ce test ne rougisse. Donner un \
         handler distinct à chaque route, ou enrichir l'identité du registre \
         (par exemple du chemin HTTP)."
    );

    let in_code: BTreeSet<(String, String)> = counted.into_keys().collect();
    let in_registry: BTreeSet<(String, String)> = LIB_ROUTES
        .iter()
        .map(|(v, h, _)| (v.to_string(), h.to_string()))
        .collect();

    let missing: Vec<_> = in_code.difference(&in_registry).collect();
    assert!(
        missing.is_empty(),
        "⛔ Route(s) mutante(s) ajoutée(s) à lib.rs sans entrée au registre \
         d'audit : {missing:?}\n\n\
         Ajouter chacune à LIB_ROUTES avec son statut : `Traced` si son chemin \
         écrit une entrée d'audit, `Exempt(\"issue #NNN — motif\")` sinon. \
         ⚠️ Une exemption SANS numéro d'issue est un abandon déguisé."
    );

    let stale: Vec<_> = in_registry.difference(&in_code).collect();
    assert!(
        stale.is_empty(),
        "⛔ Entrée(s) du registre qui ne correspondent plus à aucune route de \
         lib.rs : {stale:?} — les retirer."
    );
}

#[test]
fn every_mutating_route_of_test_endpoints_is_in_the_registry() {
    // ⚠️ Ce second volet existe parce que le patch d'une passe de revue avait
    // exclu ces trois routes du diff ENTIER : une quatrième route ajoutée ici
    // serait alors restée aussi invisible qu'avant la story.
    let source = include_str!("../src/routes/test_endpoints.rs");

    // ⚠️ La forme diffère de `lib.rs` : ici le chemin est porté par `.route(`,
    // et le verbe vient APRÈS. Un extracteur recopié tel quel rendait un
    // ensemble vide — et un ensemble vide se compare sans rien prouver.
    let mut in_code: BTreeSet<(String, String)> = BTreeSet::new();
    for line in source.lines() {
        let Some(rest) = line.trim().strip_prefix(".route(\"") else {
            continue;
        };
        let Some((path, tail)) = rest.split_once('"') else {
            continue;
        };
        for verb in ["post", "put", "delete", "patch"] {
            if tail.contains(&format!("{verb}(")) {
                in_code.insert((verb.to_string(), path.to_string()));
            }
        }
    }

    let in_registry: BTreeSet<(String, String)> = TEST_ENDPOINT_ROUTES
        .iter()
        .map(|(v, h, _)| (v.to_string(), h.to_string()))
        .collect();

    assert!(
        !in_code.is_empty(),
        "⛔ l'extracteur n'a trouvé AUCUNE route dans test_endpoints.rs — c'est \
         le détecteur qui est cassé, pas le code : un ensemble vide se compare \
         sans rien prouver"
    );

    assert_eq!(
        in_code, in_registry,
        "⛔ Les routes mutantes de test_endpoints.rs ne correspondent plus au \
         registre. Code : {in_code:?} — registre : {in_registry:?}"
    );
}

/// ⛔ **Le registre couvre DEUX fichiers ; rien n'exigeait qu'un troisième le
/// soit.** Un futur sous-routeur externe échapperait aux deux volets — ses
/// routes vivant hors de `lib.rs` et hors de `test_endpoints.rs`.
///
/// ⚠️ **Ce test inventorie les sites NON RÉSOLUS, il n'énumère pas une forme
/// qui marche.** Une première rédaction cherchait la sous-chaîne `::router()` :
/// un module exposant `routes()`, `mount()` ou `build()` l'aurait contournée en
/// silence. *C'est exactement la faute que la passe 2 avait relevée sur l'autre
/// garde de ce fichier, refaite un cran plus loin — et relevée de nouveau en
/// passe 3.*
///
/// La propriété décidable est : **tout appel à une fonction d'un module de
/// `routes::` qui n'est PAS un handler de route doit être connu.** Un handler
/// apparaît toujours comme argument d'un constructeur de méthode
/// (`post(routes::m::h)`) ; tout autre usage construit ou monte quelque chose,
/// et c'est cela qu'il faut inventorier.
/// Les appels à un module de `routes::` qui **ne sont pas des handlers** —
/// donc les sites qui construisent ou montent quelque chose.
///
/// Fonction pure, pour que la garde ci-dessous soit **éprouvable sur une source
/// factice** : muter `lib.rs` pour la tester ne compile pas, et un test qui ne
/// compile pas ne rougit pas — il se tait.
fn appels_non_handlers(source: &str) -> Vec<String> {
    let mut appels: Vec<String> = Vec::new();
    let mut reste = source;
    while let Some(pos) = reste.find("routes::") {
        let apres = &reste[pos..];
        if let Some(paren) = apres.find('(') {
            let chemin = &apres[..paren];
            // Un chemin de module tient sur une ligne et n'a ni espace ni virgule.
            if !chemin.contains(char::is_whitespace) && !chemin.contains(',') {
                // Handler ? Le constructeur de méthode le précède immédiatement.
                let avant = &reste[..pos];
                let est_handler = ["post(", "put(", "delete(", "patch(", "get("]
                    .iter()
                    .any(|c| avant.ends_with(c));
                if !est_handler {
                    appels.push(chemin.to_string());
                }
            }
        }
        reste = &reste[pos + "routes::".len()..];
    }
    appels.sort();
    appels.dedup();
    appels
}

#[test]
fn no_third_route_file_escapes_the_registry() {
    let source = include_str!("../src/lib.rs");
    let source = source
        .split("// NOTE: les stories futures ajouteront leurs routes protégées")
        .next()
        .expect("le fichier n'est pas vide");

    let appels = appels_non_handlers(source);
    assert_eq!(
        appels,
        vec!["routes::test_endpoints::router".to_string()],
        "⛔ Appel(s) à un module de `routes::` qui ne sont pas des handlers et \
         que le registre ne connaît pas : {appels:?}\n\n\
         Si c'est un sous-routeur externe, ses routes mutantes échappent aux \
         DEUX volets du registre : ajouter un volet pour son fichier, sur le \
         modèle de `every_mutating_route_of_test_endpoints_is_in_the_registry`, \
         puis inscrire ses routes. ⚠️ Ne PAS se contenter d'ajouter le nom ici."
    );
}

/// ⛔ **La garde ci-dessus est éprouvée sur une source factice**, parce que la
/// muter dans `lib.rs` ne compilerait pas — et *un test qui ne compile pas ne
/// rougit pas : il se tait.*
///
/// Les trois formes ci-dessous sont celles qui contournaient la rédaction
/// précédente, qui cherchait la sous-chaîne `::router()`.
#[test]
fn the_third_file_guard_sees_forms_that_are_not_named_router() {
    let handler_seul = r#"        .route("/api/v1/users", post(routes::users::create_user))"#;
    assert!(
        appels_non_handlers(handler_seul).is_empty(),
        "un handler n'est pas un montage — il ne doit pas être inventorié"
    );

    for forme in [
        r#"        .nest("/api/v1/x", routes::widgets::router())"#,
        r#"        .nest("/api/v1/x", routes::widgets::mount())"#,
        r#"        let sous = routes::widgets::build();"#,
    ] {
        assert_eq!(
            appels_non_handlers(forme).len(),
            1,
            "⛔ la garde doit voir CE montage, quel que soit le nom de la \
             fonction : {forme}"
        );
    }
}

#[test]
fn the_registry_partition_is_what_the_story_declares() {
    // ⚠️ Ce test recompte la ventilation DEPUIS la source plutôt que de la
    // croire : un total doit être cohérent avec sa propre ventilation.
    let traced = LIB_ROUTES.iter().filter(|(_, _, s)| *s == Traced).count();
    let exempt = LIB_ROUTES
        .iter()
        .filter(|(_, _, s)| matches!(s, Exempt(_)))
        .count();
    let no_matter = LIB_ROUTES
        .iter()
        .filter(|(_, _, s)| matches!(s, NoMatter(_)))
        .count();

    assert_eq!(LIB_ROUTES.len(), 109, "l'inventaire porte sur 109 routes");
    assert_eq!(traced + exempt + no_matter, LIB_ROUTES.len());
    assert_eq!(
        traced, 91,
        "73 tracées avant la 25-1b, plus ses 14, plus la dévalidation (25-2-b-1, #440), \
         plus l'annulation d'un règlement client (25-3-a-1) et fournisseur (25-3-a-2, #414), \
         plus l'annulation d'un rapprochement (25-3-b, #418)"
    );
    assert_eq!(
        exempt, 15,
        "11 routes d'onboarding (#434) + 4 d'auth (#435)"
    );
    assert_eq!(no_matter, 3, "trois routes mutantes qui ne mutent rien");
    assert_eq!(
        LIB_ROUTES.len() + TEST_ENDPOINT_ROUTES.len(),
        112,
        "le registre est plus large que l'inventaire, et c'est voulu"
    );
}

#[test]
fn every_exemption_names_the_issue_that_follows_it() {
    // ⛔ C'est la garde qui empêche l'exemption de devenir la sortie la moins
    // coûteuse : écrire une justification est plus rapide que tracer une route,
    // et une justification sans suivi ne se distingue pas d'un oubli.
    for (verb, handler, status) in LIB_ROUTES.iter().chain(TEST_ENDPOINT_ROUTES) {
        if let Exempt(motif) = status {
            assert!(
                motif.contains("issue #") || motif.contains("jamais monté en production"),
                "l'exemption de `{verb} {handler}` doit nommer l'issue qui la \
                 suit, ou dire pourquoi aucune n'est nécessaire — trouvé : {motif:?}"
            );
        }
    }
}
