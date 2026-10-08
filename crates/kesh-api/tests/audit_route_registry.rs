//! Registre des routes mutantes et sa garde — Story 25-1b (AC 11), étendu au
//! rejeu sur interblocage par la Story 15-5e1 (AC1, AC4 test 6 ; choix C58).
//!
//! Chaque route mutante porte **deux statuts** : son statut d'audit
//! ([`Status`]) et son statut de rejeu ([`Rejeu`]). Une route ajoutée demain
//! s'examine **une fois**, pour les deux propriétés.
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
//! entre marqueurs ; les 112 routes sont réparties dans tout le fichier.
//!
//! # Deux fichiers, deux volets
//!
//! L'ensemble clos de l'INVENTAIRE est celui de `lib.rs` (112 routes) ; celui du
//! REGISTRE est plus large (115), car trois routes mutantes vivent dans
//! `routes/test_endpoints.rs` et sont montées par un `nest()`. D'où :
//!
//! - volet « route absente du registre » → sur les **deux** fichiers, faute de
//!   quoi une quatrième route de `test_endpoints.rs` n'alerterait personne ;
//! - volet « entrée du registre absente du code » → sur les deux également,
//!   chaque entrée nommant le fichier dont elle provient.
//!
//! # Le rejeu sur interblocage : ce que le registre établit, et ce qu'il n'établit PAS
//!
//! ✅ La colonne [`Rejeu`] grave l'**inventaire fermé des routes qui écrivent au
//! journal** (Story 15-5e1, AC1 — remonté depuis les primitives d'écriture au
//! journal jusqu'aux handlers) : toute route mutante y est classée, et le volet
//! (c) ([`every_replayed_route_calls_an_envelope`]) échoue sur une route
//! `Rejouee` dont le handler n'appelle aucune enveloppe.
//!
//! ⛔ Il n'établit PAS :
//!
//! - **(i)** qu'une route classée `SansEcritureAuJournal` n'écrit pas au
//!   journal : l'écriture se fait au dépôt, parfois à trois appels du handler.
//!   C'est la classification de l'inventaire de l'AC1, recontrôlée en revue.
//! - **(ii)** que le 1213 **atteint** le prédicat : le volet (c) est statique —
//!   il voit l'appel, pas le chemin de l'erreur ; une conversion faite dans la
//!   fermeture le passerait. Seuls les tests 2 à 5 et 7 de
//!   `rejeu_interblocage_e2e.rs` le prouvent dynamiquement, pour cinq routes de
//!   la famille `DbError` (quatre qui écrivent au journal, et l'enregistrement
//!   des réglages de facturation). Pour la famille `AppError`, le test 1 éprouve
//!   le prédicat et l'enveloppe, et le chemin propre à chaque route relève de la
//!   revue fichier par fichier.
//! - **(iii)** une enveloppe appelée par une fonction auxiliaire du handler : le
//!   nom doit figurer dans le corps du handler lui-même — c'est la forme exigée.
//! - **(iii bis)** — **angle mort assumé** (revue P1, B-4 = E-2) — que
//!   l'enveloppe **enveloppe l'écriture** : le volet (c) est vrai dès qu'un
//!   appel de ce nom figure dans le corps. Un handler `Rejouee` qui
//!   envelopperait une lecture et ferait l'écriture hors de la fermeture
//!   resterait vert. Les tests 2 à 5 et 7 de `rejeu_interblocage_e2e.rs` le
//!   prouvent dynamiquement pour leurs cinq routes, et
//!   `accept_replays_the_batch_when_it_is_the_deadlock_victim`
//!   (`reconciliation_e2e.rs`) pour `reconciliation::accept`,
//!   `the_put_replays_a_deadlock_it_lost` (`journal_entry_reversal_e2e.rs`,
//!   15-8a) pour `journal_entries::update` ; pour
//!   `invoices::write_off`, `reconciliation::cancel`,
//!   `opening_balances::complete` et `journal_entries::delete` (15-8b, rejouée
//!   par uniformité avec le `PUT`, sans cycle connu — choix C-15-8b-6), c'est la
//!   revue fichier par fichier.
//! - **(iv)** — **angle mort assumé** — qu'une route `SansEcritureAuJournal` qui
//!   prend un verrou ne soit pas la **victime** d'un cycle avec un flux qui écrit
//!   au journal. P. ex. `accept_batch` tient un verrou partagé sur la ligne
//!   `companies` dès qu'une proposition a inséré son écriture
//!   (`fk_journal_entries_company`), puis demande la sentinelle `FOR UPDATE` de
//!   cette ligne pour une règle à projet ; une route qui attend entre-temps un
//!   verrou exclusif sur `companies` (`companies::lock_books`, `unlock_books`, la
//!   sentinelle des routes de `projects`, `bank_accounts`, `vat`,
//!   `dunning_levels`) ferme le cycle et, plus légère, rend 500. Rare, et hors de
//!   la promesse (« les routes qui écrivent au journal ») : le registre établit
//!   que **toute route mutante a été examinée**.
//! - **(v)** quoi que ce soit d'une route `GET` : l'extracteur ne balaie que
//!   `post`, `put`, `delete` et `patch`. Aucune route `GET` n'écrit au journal
//!   aujourd'hui (remontée de l'AC1 : des `POST` et un `DELETE`) ; l'angle mort
//!   est du même ordre que celui de l'audit (`GET /invoices/{id}/pdf`, plus bas).
//! - **(vi)** **deux routes `SansEcritureAuJournal` sont rejouées quand même** :
//!   `onboarding::finalize` (`retry_with`) et
//!   `company_invoice_settings::update_invoice_settings` (enveloppe `DbError`,
//!   exposée par l'avance des réglages de la saisie fournisseur — choix C70). La
//!   colonne dit l'inventaire de l'AC1, pas la présence d'une enveloppe, et le
//!   volet (c) ne les examine pas : la seconde est tenue par le test 7 de
//!   `rejeu_interblocage_e2e.rs`, la première par sa revue.

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

/// Statut de rejeu sur interblocage d'une route mutante (Story 15-5e1, AC1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rejeu {
    /// Écrit au journal et rejouée : son handler appelle une enveloppe
    /// (`retry_on_deadlock`, `retry_on_deadlock_with`, `retry_app_on_deadlock`
    /// ou, jusqu'à la 15-5e2, `retry_with`) — contrôlé par le volet (c).
    Rejouee,
    /// Écrit au journal, **pas encore** rejouée : statut transitoire, qui nomme
    /// la story qui la rejoue et que celle-ci retire.
    ARejouer(&'static str),
    /// N'écrit pas au journal (inventaire de l'AC1). Ne dit rien de la présence
    /// d'une enveloppe — cf. point (vi) du doc-comment du module.
    SansEcritureAuJournal,
    /// Écrit au journal et n'est pas rejouée, raison écrite.
    Exemptee(&'static str),
}

use Rejeu::{ARejouer, Exemptee, Rejouee, SansEcritureAuJournal};

/// Les routes mutantes de `crates/kesh-api/src/lib.rs`, par identité
/// `(verbe, module::handler)`.
#[rustfmt::skip]
const LIB_ROUTES: &[(&str, &str, Status, Rejeu)] = &[
    ("post", "users::create_user", Traced, SansEcritureAuJournal),
    ("put", "users::update_user", Traced, SansEcritureAuJournal),
    ("put", "users::disable_user", Traced, SansEcritureAuJournal),
    ("put", "users::reset_password", Traced, SansEcritureAuJournal),
    ("put", "company_invoice_settings::update_invoice_settings", Traced, SansEcritureAuJournal),
    ("post", "vat::create_vat_rate", Traced, SansEcritureAuJournal),
    ("put", "vat::update_vat_rate", Traced, SansEcritureAuJournal),
    ("delete", "vat::deactivate_vat_rate", Traced, SansEcritureAuJournal),
    ("put", "company_dunning_settings::update_dunning_settings", Traced, SansEcritureAuJournal),
    ("post", "dunning_levels::create_dunning_level", Traced, SansEcritureAuJournal),
    ("put", "dunning_levels::update_dunning_level", Traced, SansEcritureAuJournal),
    ("delete", "dunning_levels::delete_dunning_level", Traced, SansEcritureAuJournal),
    ("delete", "invoices::delete_invoice", Traced, SansEcritureAuJournal),
    ("post", "dunning_reminders::cancel_reminder", Traced, SansEcritureAuJournal),
    ("post", "admin::full_import", Traced, Exemptee("restauration d'instance : geste d'administration exclusif, hors exploitation — un 1213 annule sa transaction unique et la relance manuelle est sûre")),
    ("put", "email_templates::update_email_template", Traced, SansEcritureAuJournal),
    ("delete", "email_templates::restore_email_template_default", Traced, SansEcritureAuJournal),
    ("put", "companies::update_company_email", Traced, SansEcritureAuJournal),
    ("put", "companies::update_company_contact_details", Traced, SansEcritureAuJournal),
    ("post", "fiscal_years::reopen_fiscal_year", Traced, SansEcritureAuJournal),
    ("post", "companies::unlock_company_books", Traced, SansEcritureAuJournal),
    ("post", "onboarding::reset", Exempt("issue #434 — toute la séquence d'installation est non tracée"), Exemptee("effacement de la démo : geste d'administration exclusif, hors exploitation — un 1213 annule sa transaction unique et la relance manuelle est sûre")),
    ("post", "accounts::create_account", Traced, SansEcritureAuJournal),
    ("put", "accounts::update_account", Traced, SansEcritureAuJournal),
    ("put", "accounts::archive_account", Traced, SansEcritureAuJournal),
    ("put", "accounts::reactivate_account", Traced, SansEcritureAuJournal),
    ("post", "journal_entries::create_journal_entry", Traced, ARejouer("15-5e2")),
    // Story 15-8a (#532) : l'audit `journal_entry.updated` vient de `journal_entries::update`.
    // Rejouée par la 15-8a (`retry_with`, C-15-8-19), nommée à l'intégration de la 15-5e1.
    ("put", "journal_entries::update_journal_entry", Traced, Rejouee),
    // Story 15-8b (#532) : l'audit `journal_entry.deleted` vient de `journal_entries::delete_in_tx`.
    // Rejouée par la 15-8b (`retry_with`, par uniformité avec le `PUT`), nommée à son intégration.
    ("delete", "journal_entries::delete_journal_entry", Traced, Rejouee),
    ("post", "companies::lock_company_books", Traced, SansEcritureAuJournal),
    ("post", "journal_entries::reverse_journal_entry", Traced, ARejouer("15-5e2")),
    ("post", "opening_balances::generate_opening_balances", Traced, ARejouer("15-5e2")),
    // Story 25-7 (#445) : l'audit vient de `create_in_tx` (`journal_entry.created`).
    ("post", "opening_balances::complete_opening_balances", Traced, Rejouee),
    ("post", "contacts::create_contact", Traced, SansEcritureAuJournal),
    ("put", "contacts::update_contact", Traced, SansEcritureAuJournal),
    ("put", "contacts::archive_contact", Traced, SansEcritureAuJournal),
    ("post", "contact_persons::create_person", Traced, SansEcritureAuJournal),
    ("put", "contact_persons::update_person", Traced, SansEcritureAuJournal),
    ("delete", "contact_persons::delete_person", Traced, SansEcritureAuJournal),
    ("post", "products::create_product", Traced, SansEcritureAuJournal),
    ("put", "products::update_product", Traced, SansEcritureAuJournal),
    ("put", "products::archive_product", Traced, SansEcritureAuJournal),
    ("post", "invoices::create_invoice", Traced, SansEcritureAuJournal),
    ("put", "invoices::update_invoice", Traced, SansEcritureAuJournal),
    ("post", "credit_notes::create_credit_note", Traced, ARejouer("15-5e2")),
    ("post", "supplier_invoices::create_supplier_invoice", Traced, Rejouee),
    ("post", "supplier_invoices::pay_supplier_invoice", Traced, ARejouer("15-5e2")),
    ("post", "supplier_invoices::cancel_supplier_invoice", Traced, ARejouer("15-5e2")),
    ("post", "supplier_invoices::scan_qr_supplier_invoice", NoMatter("parsing pur du payload SPC, aucun accès base"), SansEcritureAuJournal),
    ("post", "imported_supplier_invoices::post_inbox_import", Traced, SansEcritureAuJournal),
    ("post", "imported_supplier_invoices::complete_import", Traced, ARejouer("15-5e2")),
    ("post", "imported_supplier_invoices::discard_import", Traced, SansEcritureAuJournal),
    ("post", "payment_batches::create_payment_batch", Traced, SansEcritureAuJournal),
    ("post", "payment_batches::confirm_payment_batch", Traced, ARejouer("15-5e2")),
    ("post", "payment_batches::cancel_payment_batch", Traced, SansEcritureAuJournal),
    ("post", "invoices::validate_invoice_handler", Traced, Rejouee),
    ("post", "invoices::unvalidate_invoice_handler", Traced, ARejouer("15-5e2")),
    // Story 25-6-b (#387) — refiger le PDF d'une facture (audit
    // `invoice.pdf_refrozen`, dans la transaction de `invoices::refreeze_pdf`).
    //
    // ⚠️ ANGLE MORT ASSUMÉ : `GET /api/v1/invoices/{id}/pdf`
    // (`invoice_pdf::get_invoice_pdf`) ÉCRIT désormais — le premier rendu fige
    // le document et trace `invoice.pdf_frozen` —, et `HEAD` aussi, axum le
    // servant par le handler `get`. Ce registre ne balaie que les méthodes
    // d'écriture : cette route mutante lui échappe. Son audit est tenu par
    // `invoices::freeze_pdf`, qui ne pose rien sans sa trace.
    ("post", "invoice_pdf::refreeze_invoice_pdf", Traced, SansEcritureAuJournal),
    ("post", "invoices::settle_invoice_handler", Traced, Rejouee),
    ("post", "invoices::write_off_invoice_handler", Traced, Rejouee),
    ("post", "invoices::cancel_invoice_settlement_handler", Traced, Rejouee),
    ("post", "supplier_invoices::cancel_supplier_invoice_settlement", Traced, ARejouer("15-5e2")),
    ("put", "dunning_reminders::pause_dunning", Traced, SansEcritureAuJournal),
    ("put", "dunning_reminders::resume_dunning", Traced, SansEcritureAuJournal),
    ("post", "dunning_reminders::record_manual_reminder", Traced, SansEcritureAuJournal),
    ("post", "invoice_email::send_reminder", Traced, SansEcritureAuJournal),
    ("post", "invoice_email::send_reminder_batch", Traced, SansEcritureAuJournal),
    ("post", "invoice_email::send_invoice_email", Traced, SansEcritureAuJournal),
    ("post", "fiscal_years::create_fiscal_year", Traced, SansEcritureAuJournal),
    ("put", "fiscal_years::update_fiscal_year", Traced, SansEcritureAuJournal),
    ("post", "fiscal_years::close_fiscal_year", Traced, SansEcritureAuJournal),
    ("post", "bank_imports::preview", NoMatter("parse et valide sans persistance — aucune mutation"), SansEcritureAuJournal),
    ("post", "bank_imports::create", Traced, SansEcritureAuJournal),
    ("post", "bank_profiles::create", Traced, SansEcritureAuJournal),
    ("put", "bank_profiles::update", Traced, SansEcritureAuJournal),
    ("delete", "bank_profiles::delete", Traced, SansEcritureAuJournal),
    ("post", "reconciliation::post_accept", Traced, Rejouee),
    ("post", "reconciliation::post_reject", Traced, SansEcritureAuJournal),
    ("post", "reconciliation::post_manual", Traced, ARejouer("15-5e2")),
    ("post", "reconciliation::post_split", Traced, ARejouer("15-5e2")),
    // Story 25-3-b (#418) — `reconciliation.cancelled`.
    ("post", "reconciliation::post_cancel_reconciliation", Traced, Rejouee),
    ("post", "bank_accounts::create_bank_account", Traced, SansEcritureAuJournal),
    ("patch", "bank_accounts::patch_bank_account_journal_link", Traced, SansEcritureAuJournal),
    ("put", "bank_accounts::update_bank_account", Traced, SansEcritureAuJournal),
    ("delete", "bank_accounts::archive_bank_account", Traced, SansEcritureAuJournal),
    ("post", "reconciliation_rules::post_create", Traced, SansEcritureAuJournal),
    ("patch", "reconciliation_rules::patch", Traced, SansEcritureAuJournal),
    ("delete", "reconciliation_rules::delete", Traced, SansEcritureAuJournal),
    ("post", "api_keys::create", Traced, SansEcritureAuJournal),
    ("delete", "api_keys::revoke", Traced, SansEcritureAuJournal),
    ("post", "projects::create_project", Traced, SansEcritureAuJournal),
    ("put", "projects::update_project", Traced, SansEcritureAuJournal),
    ("post", "projects::archive_project", Traced, SansEcritureAuJournal),
    ("post", "projects::unarchive_project", Traced, SansEcritureAuJournal),
    ("put", "auth::change_password", Exempt("issue #435 — change_password, login, logout, refresh"), SansEcritureAuJournal),
    ("put", "profile::set_mode", Traced, SansEcritureAuJournal),
    ("post", "onboarding::set_language", Exempt("issue #434 — toute la séquence d'installation est non tracée"), SansEcritureAuJournal),
    ("post", "onboarding::set_mode", Exempt("issue #434 — toute la séquence d'installation est non tracée"), SansEcritureAuJournal),
    ("post", "onboarding::seed_demo", Exempt("issue #434 — toute la séquence d'installation est non tracée"), SansEcritureAuJournal),
    ("post", "onboarding::start_production", Exempt("issue #434 — toute la séquence d'installation est non tracée"), SansEcritureAuJournal),
    ("post", "onboarding::set_org_type", Exempt("issue #434 — toute la séquence d'installation est non tracée"), SansEcritureAuJournal),
    ("post", "onboarding::set_accounting_language", Exempt("issue #434 — toute la séquence d'installation est non tracée"), SansEcritureAuJournal),
    ("post", "onboarding::set_coordinates", Exempt("issue #434 — toute la séquence d'installation est non tracée"), SansEcritureAuJournal),
    ("post", "onboarding::set_bank_account", Exempt("issue #434 — toute la séquence d'installation est non tracée"), SansEcritureAuJournal),
    ("post", "onboarding::skip_bank", Exempt("issue #434 — toute la séquence d'installation est non tracée"), SansEcritureAuJournal),
    ("post", "onboarding::finalize", Exempt("issue #434 — toute la séquence d'installation est non tracée"), SansEcritureAuJournal),
    ("post", "auth::login", Exempt("issue #435 — change_password, login, logout, refresh"), SansEcritureAuJournal),
    ("post", "auth::logout", Exempt("issue #435 — change_password, login, logout, refresh"), SansEcritureAuJournal),
    ("post", "auth::refresh", Exempt("issue #435 — change_password, login, logout, refresh"), SansEcritureAuJournal),
    ("post", "setup::create_admin", Traced, SansEcritureAuJournal),
    ("post", "auth::forgot_password", Traced, SansEcritureAuJournal),
    ("post", "auth::reset_password", Traced, SansEcritureAuJournal),];

/// Les routes mutantes de `crates/kesh-api/src/routes/test_endpoints.rs`,
/// montées par le `nest("/api/v1/_test", …)` de `lib.rs` et conditionnées au
/// mode test.
#[rustfmt::skip]
const TEST_ENDPOINT_ROUTES: &[(&str, &str, Status, Rejeu)] = &[
    ("post", "/seed", Exempt("chemin de test, jamais monté en production — efface delibérément la base"), Exemptee("mode test, jamais monté en production")),
    ("post", "/reset", Exempt("chemin de test, jamais monté en production"), Exemptee("mode test, jamais monté en production")),
    ("post", "/password-reset-token", Exempt("chemin de test, jamais monté en production"), SansEcritureAuJournal),
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
        .map(|(v, h, _, _)| (v.to_string(), h.to_string()))
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
        .map(|(v, h, _, _)| (v.to_string(), h.to_string()))
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
    let traced = LIB_ROUTES
        .iter()
        .filter(|(_, _, s, _)| *s == Traced)
        .count();
    let exempt = LIB_ROUTES
        .iter()
        .filter(|(_, _, s, _)| matches!(s, Exempt(_)))
        .count();
    let no_matter = LIB_ROUTES
        .iter()
        .filter(|(_, _, s, _)| matches!(s, NoMatter(_)))
        .count();

    assert_eq!(LIB_ROUTES.len(), 112, "l'inventaire porte sur 112 routes");
    assert_eq!(traced + exempt + no_matter, LIB_ROUTES.len());
    assert_eq!(
        traced, 95,
        "73 tracées avant la 25-1b, plus ses 14, plus la dévalidation (25-2-b-1, #440), \
         plus l'annulation d'un règlement client (25-3-a-1) et fournisseur (25-3-a-2, #414), \
         plus l'annulation d'un rapprochement (25-3-b, #418), plus le solde du reste \
         (25-4-d2a, #384), plus le refigeage du PDF d'une facture (25-6-b, #387), plus le \
         complément des soldes de départ (25-7, #445), plus la modification d'une écriture \
         (15-8a, #532 — le `PUT` gelé par la 24-4b ne mutait rien)"
    );
    assert_eq!(
        exempt, 15,
        "11 routes d'onboarding (#434) + 4 d'auth (#435)"
    );
    assert_eq!(
        no_matter, 2,
        "deux routes mutantes qui ne mutent rien (le `PUT` des écritures en est sorti, 15-8a)"
    );
    assert_eq!(
        LIB_ROUTES.len() + TEST_ENDPOINT_ROUTES.len(),
        115,
        "le registre est plus large que l'inventaire, et c'est voulu"
    );
}

#[test]
fn every_exemption_names_the_issue_that_follows_it() {
    // ⛔ C'est la garde qui empêche l'exemption de devenir la sortie la moins
    // coûteuse : écrire une justification est plus rapide que tracer une route,
    // et une justification sans suivi ne se distingue pas d'un oubli.
    for (verb, handler, status, _) in LIB_ROUTES.iter().chain(TEST_ENDPOINT_ROUTES) {
        if let Exempt(motif) = status {
            assert!(
                motif.contains("issue #") || motif.contains("jamais monté en production"),
                "l'exemption de `{verb} {handler}` doit nommer l'issue qui la \
                 suit, ou dire pourquoi aucune n'est nécessaire — trouvé : {motif:?}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Rejeu sur interblocage — Story 15-5e1 (AC4, test 6)
// ---------------------------------------------------------------------------

/// Les enveloppes de rejeu, cherchées par le **dernier segment** du chemin
/// appelé. `retry_with` n'est accepté que jusqu'à la 15-5e2, qui le restreint à
/// `post_accept`.
const ENVELOPPES: &[&str] = &[
    "retry_on_deadlock",
    "retry_on_deadlock_with",
    "retry_app_on_deadlock",
    "retry_with",
];

/// Visiteur `syn` du **corps** d'un handler : vrai dès qu'il rencontre un appel
/// à une enveloppe — `ExprCall` dont la fonction est un chemin terminé par l'un
/// des noms de [`ENVELOPPES`], ou `ExprMethodCall` de ce nom.
///
/// ⚠️ Les deux surcharges **rappellent** l'implémentation par défaut : sans ce
/// rappel, le visiteur ne descendrait plus sous un appel et manquerait
/// `Ok(retry_with(…).await.map_err(…)?)`. Il ne descend pas, en revanche, dans
/// les items déclarés dans le corps (`fn`, `impl`, `mod` imbriqués).
#[derive(Default)]
struct ChercheEnveloppe {
    trouve: bool,
}

impl<'ast> syn::visit::Visit<'ast> for ChercheEnveloppe {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(chemin) = &*node.func
            && let Some(dernier) = chemin.path.segments.last()
            && ENVELOPPES.iter().any(|e| dernier.ident == e)
        {
            self.trouve = true;
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if ENVELOPPES.iter().any(|e| node.method == e) {
            self.trouve = true;
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_item(&mut self, _node: &'ast syn::Item) {
        // Un item déclaré dans le corps n'est pas le corps du handler.
    }
}

/// Retrouve **par son nom** les `fn` libres d'un fichier source.
struct CherchePlusieursFn<'a> {
    nom: &'a str,
    corps: Vec<syn::Block>,
}

impl<'ast> syn::visit::Visit<'ast> for CherchePlusieursFn<'_> {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if node.sig.ident == self.nom {
            self.corps.push((*node.block).clone());
        }
        syn::visit::visit_item_fn(self, node);
    }
}

/// Le corps du handler `nom` appelle-t-il une enveloppe ? `Err` (message qui
/// nomme la cause) si le source ne se parse pas, ou si aucun `fn` — ou plus
/// d'un — ne porte ce nom : une route `Rejouee` n'est jamais sautée.
fn handler_appelle_une_enveloppe(source: &str, nom: &str) -> Result<bool, String> {
    use syn::visit::Visit;
    let fichier = syn::parse_file(source).map_err(|e| format!("source non analysable : {e}"))?;
    let mut cherche = CherchePlusieursFn {
        nom,
        corps: Vec::new(),
    };
    cherche.visit_file(&fichier);
    match cherche.corps.as_slice() {
        [corps] => {
            let mut visiteur = ChercheEnveloppe::default();
            visiteur.visit_block(corps);
            Ok(visiteur.trouve)
        }
        [] => Err(format!("aucun `fn {nom}` dans le fichier")),
        plusieurs => Err(format!("{} `fn {nom}` dans le fichier", plusieurs.len())),
    }
}

/// **Volet (c)** — toute route `Rejouee` appelle une enveloppe **dans le corps
/// de son handler**, vérifié par l'analyseur de Rust (`syn`) : un commentaire,
/// un doc-comment, un littéral ne sont pas des appels.
///
/// Limite écrite : un appel placé dans une macro (`tokio::join!(…)`) n'est pas
/// analysé — il rend « absente », un faux rouge, jamais un faux vert.
#[test]
fn every_replayed_route_calls_an_envelope() {
    let racine = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/routes");
    let mut examinees = 0;
    for (verbe, identite, _, rejeu) in LIB_ROUTES {
        if *rejeu != Rejouee {
            continue;
        }
        let (module, handler) = identite
            .split_once("::")
            .unwrap_or_else(|| panic!("identité sans module : {identite}"));
        let chemin = racine.join(format!("{module}.rs"));
        let source = std::fs::read_to_string(&chemin).unwrap_or_else(|e| {
            panic!(
                "⛔ route `{verbe} {identite}` (Rejouee) : fichier {} illisible ({e}) — \
                 module en répertoire ? Une route Rejouee n'est jamais sautée.",
                chemin.display()
            )
        });
        match handler_appelle_une_enveloppe(&source, handler) {
            Ok(true) => examinees += 1,
            Ok(false) => panic!(
                "⛔ route `{verbe} {identite}` classée Rejouee, mais le corps de son \
                 handler n'appelle aucune enveloppe ({ENVELOPPES:?}). Envelopper son \
                 écriture (Story 15-5e1, AC2) ou corriger son statut de rejeu."
            ),
            Err(cause) => panic!(
                "⛔ route `{verbe} {identite}` (Rejouee) : handler introuvable dans {} — \
                 {cause}",
                chemin.display()
            ),
        }
    }
    assert_eq!(examinees, 10, "les dix routes Rejouee ont été examinées");
}

/// Le visiteur du volet (c), éprouvé sur un **source synthétique** : muter les
/// fichiers de routes pour l'éprouver ne laisserait rien dans le dépôt.
#[test]
fn the_envelope_visitor_sees_calls_and_only_calls() {
    let source = r##"
        /// Le handler suivant n'appelle pas retry_with(…) : ce doc-comment est
        /// placé au-dessus de `doc_du_suivant`, pas dans le corps de `nu`.
        pub async fn nu(s: &'static str, t: &'_ str) -> Result<(), E> {
            let _octet = b';';
            let _brut = r#"} retry_with( {"#;
            retry_on_deadlock("x", || async { Ok(()) }).await
        }

        /// … retry_with(…)
        pub async fn doc_du_suivant() -> Result<(), E> {
            // retry_with(1, p, f)
            /* retry_on_deadlock("x", f) */
            let _ = "retry_with(";
            mon_retry_with(1, p, f).await
        }

        pub async fn qualifie() -> Result<(), E> {
            kesh_db::retry::retry_with("x", 3, p, f).await
        }

        pub async fn turbofish() -> Result<(), E> {
            retry_with::<_, _, (), E, _>("x", 3, p, f).await
        }

        pub async fn methode() -> Result<(), E> {
            enveloppe.retry_app_on_deadlock("x", f).await
        }

        pub async fn imbrique<'a>(x: &'a str) -> Result<i64, E> {
            Ok(retry_on_deadlock_with("x", 3, || async { Ok(1) }).await.map_err(conv)?)
        }

        pub async fn dans_une_fermeture() -> Result<(), E> {
            let f = || async move { retry_app_on_deadlock("x", g).await };
            f().await
        }

        pub async fn dans_un_bloc() -> Result<(), E> {
            {
                { retry_with("x", 3, p, f).await }
            }
        }

        pub async fn ailleurs() -> Result<(), E> {
            aide().await
        }

        async fn aide() -> Result<(), E> {
            retry_with("x", 3, p, f).await
        }

        pub async fn item_imbrique() -> Result<(), E> {
            fn interne() { retry_with("x", 3, p, f); }
            Ok(())
        }

        pub async fn deux() {}
        mod m { pub async fn deux() {} }
    "##;
    for trouve in [
        "nu",
        "qualifie",
        "turbofish",
        "methode",
        "imbrique",
        "dans_une_fermeture",
        "dans_un_bloc",
        "aide",
    ] {
        assert_eq!(
            handler_appelle_une_enveloppe(source, trouve),
            Ok(true),
            "l'appel de `{trouve}` doit être trouvé"
        );
    }
    for absent in ["doc_du_suivant", "ailleurs", "item_imbrique"] {
        assert_eq!(
            handler_appelle_une_enveloppe(source, absent),
            Ok(false),
            "`{absent}` n'appelle aucune enveloppe dans son corps"
        );
    }
    assert!(
        handler_appelle_une_enveloppe(source, "introuvable").is_err(),
        "un handler absent est une erreur, pas un « non »"
    );
    assert!(
        handler_appelle_une_enveloppe(source, "deux").is_err(),
        "deux `fn` du même nom sont une erreur, pas un choix"
    );
    assert!(
        handler_appelle_une_enveloppe("fn cassé( {", "x").is_err(),
        "un source que syn ne parse pas est une erreur"
    );
}

/// **Volet (d)** — la partition de rejeu, recomptée depuis la source, avec
/// son total (Story 15-5e1, AC4 test 6).
#[test]
fn the_replay_partition_is_what_the_story_declares() {
    let tout: Vec<_> = LIB_ROUTES.iter().chain(TEST_ENDPOINT_ROUTES).collect();
    let rejouees = tout.iter().filter(|(_, _, _, r)| *r == Rejouee).count();
    let a_rejouer = tout
        .iter()
        .filter(|(_, _, _, r)| matches!(r, ARejouer(_)))
        .count();
    let exemptees = tout
        .iter()
        .filter(|(_, _, _, r)| matches!(r, Exemptee(_)))
        .count();
    let sans_ecriture = tout
        .iter()
        .filter(|(_, _, _, r)| *r == SansEcritureAuJournal)
        .count();

    assert_eq!(
        rejouees, 10,
        "5 rejouées avant la 15-5e1 (write_off, accept, cancel du rapprochement, \
         complément des soldes de départ, modification d'une écriture — 15-8a) + 4 par \
         elle (validation, règlement, annulation de règlement, saisie fournisseur) + la \
         suppression d'une écriture (15-8b)"
    );
    assert_eq!(a_rejouer, 12, "12 routes rejouées par la 15-5e2");
    assert_eq!(
        exemptees, 4,
        "full_import, onboarding::reset, /seed, /reset"
    );
    assert_eq!(
        sans_ecriture, 89,
        "88 routes de lib.rs, plus /password-reset-token de test_endpoints.rs"
    );
    assert_eq!(rejouees + a_rejouer + exemptees + sans_ecriture, tout.len());
    assert_eq!(tout.len(), 115);
    for (verbe, identite, _, rejeu) in &tout {
        if let ARejouer(fiche) = rejeu {
            assert_eq!(
                *fiche, "15-5e2",
                "`{verbe} {identite}` : le statut transitoire nomme la story qui le retire"
            );
        }
    }
}
