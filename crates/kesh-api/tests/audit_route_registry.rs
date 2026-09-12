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
//! entre marqueurs ; les 105 routes sont réparties dans tout le fichier.
//!
//! # Deux fichiers, deux volets
//!
//! L'ensemble clos de l'INVENTAIRE est celui de `lib.rs` (105 routes) ; celui du
//! REGISTRE est plus large (108), car trois routes mutantes vivent dans
//! `routes/test_endpoints.rs` et sont montées par un `nest()`. D'où :
//!
//! - volet « route absente du registre » → sur les **deux** fichiers, faute de
//!   quoi une quatrième route de `test_endpoints.rs` n'alerterait personne ;
//! - volet « entrée du registre absente du code » → sur les deux également,
//!   chaque entrée nommant le fichier dont elle provient.

use std::collections::BTreeSet;

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
    ("post", "invoices::settle_invoice_handler", Traced),
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

/// Extrait les routes mutantes d'un source par leur identité.
fn extract(source: &str, pattern_prefix: &str) -> BTreeSet<(String, String)> {
    let mut out = BTreeSet::new();
    for verb in ["post", "put", "delete", "patch"] {
        let needle = format!("{verb}(");
        let mut rest = source;
        while let Some(pos) = rest.find(&needle) {
            let after = &rest[pos + needle.len()..];
            if let Some(close) = after.find(')') {
                let arg = &after[..close];
                if arg.starts_with(pattern_prefix) {
                    out.insert((
                        verb.to_string(),
                        arg.trim_start_matches(pattern_prefix).to_string(),
                    ));
                }
            }
            rest = &rest[pos + needle.len()..];
        }
    }
    out
}

#[test]
fn every_mutating_route_of_lib_is_in_the_registry() {
    // ⚠️ On coupe au bloc d'exemple commenté de fin de fichier : il porte des
    // constructeurs de route qui ne sont pas des routes.
    let source = include_str!("../src/lib.rs");
    let source = source
        .split("// let app = build_router")
        .next()
        .expect("le fichier n'est pas vide");

    let in_code = extract(source, "routes::");
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

    assert_eq!(LIB_ROUTES.len(), 105, "l'inventaire porte sur 105 routes");
    assert_eq!(traced + exempt + no_matter, LIB_ROUTES.len());
    assert_eq!(traced, 87, "73 tracées avant la story, plus ses 14");
    assert_eq!(
        exempt, 15,
        "11 routes d'onboarding (#434) + 4 d'auth (#435)"
    );
    assert_eq!(no_matter, 3, "trois routes mutantes qui ne mutent rien");
    assert_eq!(
        LIB_ROUTES.len() + TEST_ENDPOINT_ROUTES.len(),
        108,
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
