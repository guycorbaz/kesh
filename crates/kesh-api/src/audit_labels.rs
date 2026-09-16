//! Les libellés du journal d'audit — **source unique** (Story 25-1c-a).
//!
//! Le journal stocke des **codes** : `invoice.validated`, `contact`. Ce module
//! est le seul endroit qui les traduit, et il sert la liste JSON, la route de
//! vocabulaire et l'export CSV. L'écran, lui, ne traduit rien : il affiche ce
//! que la route lui rend (25-1c-b1).
//!
//! ⚠️ **Module PUBLIC, délibérément** : la garde
//! `tests/audit_label_registry.rs` est une **crate externe** et ne voit que le
//! `pub` de `kesh_api`. Sans cela, elle devrait recopier les deux listes et la
//! dérivation des clés — et ne verrait plus une mutation de la vraie.
//!
//! ⛔ **La langue est celle de l'INSTALLATION** (`state.config.locale`,
//! `KESH_LANG`), jamais la `accounting_language` de la société : c'est la langue
//! que l'interface affiche, et le journal se lit à l'écran.

use kesh_i18n::{I18nBundle, Locale};

/// Les types d'entité écrits par le code de production, **triés par code**.
///
/// ⚠️ Les codes au pluriel (`bank_imports`, `bank_profiles`,
/// `reconciliation_rules`) sont les codes **réels** de la base : ne pas les
/// « corriger ». Leur libellé, lui, est au singulier — il désigne une entrée.
pub const ENTITY_TYPES: &[&str] = &[
    "account",
    "api_key",
    "bank_account",
    "bank_imports",
    "bank_profiles",
    "bank_transaction",
    "company",
    "company_dunning_settings",
    "company_invoice_settings",
    "contact",
    "contact_person",
    "credit_note",
    "dunning_level",
    "email_template",
    "export",
    "fiscal_year",
    "imported_supplier_invoice",
    "installation",
    "invoice",
    "journal_entry",
    "payment_batch",
    "product",
    "project",
    "reconciliation_rules",
    "report",
    "supplier_invoice",
    "user",
    "vat_rate",
];

/// Les actions écrites par le code de production, **triées par code**.
///
/// ⚠️ **Dix d'entre elles ne s'écrivent pas en toutes lettres** au site
/// d'appel : elles passent par une variable conditionnelle
/// (`user.role_changed`, `invoice.paid`…) ou par le helper `build_audit_entry`
/// des exercices comptables. Un inventaire par « littéraux » en manquait dix,
/// dont `invoice.paid`. C'est la garde de `tests/audit_label_registry.rs` qui
/// fixe cette liste, pas la lecture d'un humain.
///
/// ⚠️ `admin_break_glass_reset` **n'a pas de point** et reste une action.
pub const ACTIONS: &[&str] = &[
    "account.archived",
    "account.created",
    "account.reactivated",
    "account.updated",
    "admin.full_export",
    "admin.full_import",
    "admin_break_glass_reset",
    "api_key.created",
    "api_key.revoked",
    "auth.password_reset_completed",
    "auth.password_reset_requested",
    "bank_account.archived",
    "bank_account.created",
    "bank_account.updated",
    "bank_import.created",
    "bank_profile.created",
    "bank_profile.deleted",
    "bank_profile.updated",
    "books.locked",
    "books.restored",
    "books.unlocked",
    "company.updated",
    "company_dunning_settings.updated",
    "company_invoice_settings.updated",
    "contact.archived",
    "contact.created",
    "contact.updated",
    "contact_person.archived",
    "contact_person.created",
    "contact_person.updated",
    "credit_note.created",
    "dunning_level.created",
    "dunning_level.deleted",
    "dunning_level.updated",
    "email_template.restored_default",
    "email_template.updated",
    "exports.global",
    "fiscal_year.closed",
    "fiscal_year.created",
    "fiscal_year.reopened",
    "fiscal_year.updated",
    "imported_supplier_invoice.completed",
    "imported_supplier_invoice.created",
    "imported_supplier_invoice.discarded",
    "imported_supplier_invoice.reactivated",
    "installation.ui_mode_changed",
    "invoice.cancelled",
    "invoice.created",
    "invoice.deleted",
    "invoice.dunning_paused",
    "invoice.dunning_resumed",
    "invoice.emailed",
    "invoice.paid",
    "invoice.partially_settled",
    "invoice.reminder_cancelled",
    "invoice.reminder_sent",
    "invoice.updated",
    "invoice.validated",
    "journal_entry.created",
    "journal_entry.deleted",
    "journal_entry.reversed",
    "payment_batch.cancelled",
    "payment_batch.confirmed",
    "payment_batch.generated",
    "product.archived",
    "product.created",
    "product.updated",
    "project.archived",
    "project.created",
    "project.unarchived",
    "project.updated",
    "reconciliation.accepted",
    "reconciliation.manual_matched",
    "reconciliation.rejected",
    "reconciliation.split_applied",
    "reconciliation_rule.applied",
    "reconciliation_rule.created",
    "reconciliation_rule.deleted",
    "reconciliation_rule.updated",
    "report.exported",
    "report.generated",
    "supplier_invoice.cancelled",
    "supplier_invoice.created",
    "supplier_invoice.paid",
    "user.created",
    "user.disabled",
    "user.password_reset",
    "user.role_changed",
    "user.updated",
    "vat_rate.created",
    "vat_rate.deactivated",
    "vat_rate.updated",
];

/// Préfixe de clé des types d'entité.
pub const PREFIX_ENTITY: &str = "entity";
/// Préfixe de clé des actions.
pub const PREFIX_ACTION: &str = "action";
/// Préfixe de clé des types d'auteur.
pub const PREFIX_ACTOR_TYPE: &str = "actor-type";

/// Dérive la clé i18n d'un code : `invoice.reminder_sent` →
/// `audit-log-action-invoice-reminder-sent`.
///
/// ⛔ **Une seule dérivation dans tout le dépôt.** La garde de
/// `tests/audit_label_registry.rs` appelle cette fonction ; si elle recopiait la
/// règle, une mutation de la vraie ne la ferait plus rougir.
pub fn message_key(prefix: &str, code: &str) -> String {
    format!("audit-log-{prefix}-{}", code.replace(['.', '_'], "-"))
}

/// Traduit un code, ou **rend le code** s'il est inconnu.
///
/// ⛔ **Le test d'appartenance n'est pas décoratif.** `I18nBundle::format` replie
/// sur le français puis rend **la clé brute** : appelé sur un code hors liste, il
/// afficherait `audit-log-action-foo-bar` à l'écran et dans le CSV.
///
/// Motif du repli sur le code : le journal conserve les codes des versions
/// antérieures (`journal_entry.updated`, qu'aucun site n'écrit plus), et un code
/// futur ne doit rien casser.
fn label(i18n: &I18nBundle, locale: &Locale, prefix: &str, connus: &[&str], code: &str) -> String {
    if connus.contains(&code) {
        i18n.format(locale, &message_key(prefix, code), None)
    } else {
        code.to_string()
    }
}

/// Libellé d'un type d'entité, dans la langue de l'interface.
pub fn entity_type_label(i18n: &I18nBundle, locale: &Locale, code: &str) -> String {
    label(i18n, locale, PREFIX_ENTITY, ENTITY_TYPES, code)
}

/// Libellé d'une action, dans la langue de l'interface.
pub fn action_label(i18n: &I18nBundle, locale: &Locale, code: &str) -> String {
    label(i18n, locale, PREFIX_ACTION, ACTIONS, code)
}

/// Libellé d'un type d'auteur — `user` ou `api_key`, les deux seules valeurs
/// que la colonne `actor_type` peut porter.
pub fn actor_type_label(i18n: &I18nBundle, locale: &Locale, code: &str) -> String {
    label(i18n, locale, PREFIX_ACTOR_TYPE, ACTOR_TYPES, code)
}

/// Les deux valeurs de la colonne `actor_type`.
pub const ACTOR_TYPES: &[&str] = &["api_key", "user"];

#[cfg(test)]
mod tests {
    use super::*;

    /// Le bundle réel du dépôt — pas un double : ce qu'on veut savoir, c'est si
    /// les vrais catalogues répondent.
    fn bundle() -> I18nBundle {
        I18nBundle::load(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .expect("crates/kesh-api a un parent")
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("charger les catalogues du dépôt")
    }

    #[test]
    fn les_trois_listes_sont_triees_et_sans_doublon() {
        // Le tri n'est pas cosmétique : il rend le diff d'un ajout lisible, et
        // c'est ce qui permet de relire la liste contre la sortie de la garde.
        for (nom, liste) in [
            ("ENTITY_TYPES", ENTITY_TYPES),
            ("ACTIONS", ACTIONS),
            ("ACTOR_TYPES", ACTOR_TYPES),
        ] {
            let mut trie = liste.to_vec();
            trie.sort_unstable();
            assert_eq!(trie, liste.to_vec(), "{nom} n'est pas trié");
            trie.dedup();
            assert_eq!(trie.len(), liste.len(), "{nom} contient un doublon");
        }
    }

    #[test]
    fn la_cle_derive_du_code_en_remplacant_points_et_soulignes() {
        assert_eq!(
            message_key(PREFIX_ACTION, "invoice.reminder_sent"),
            "audit-log-action-invoice-reminder-sent"
        );
        // Sans point : le code le plus atypique du lot.
        assert_eq!(
            message_key(PREFIX_ACTION, "admin_break_glass_reset"),
            "audit-log-action-admin-break-glass-reset"
        );
        assert_eq!(
            message_key(PREFIX_ENTITY, "imported_supplier_invoice"),
            "audit-log-entity-imported-supplier-invoice"
        );
    }

    #[test]
    fn un_code_inconnu_rend_le_code_et_jamais_la_cle() {
        // ⛔ Le cœur du module. Sans le test d'appartenance, `format` rendrait
        // `audit-log-action-parti-en-vacances` — affiché tel quel à l'écran et
        // dans le CSV. Le journal porte des codes de versions antérieures, donc
        // ce chemin est emprunté pour de vrai, pas seulement en théorie.
        let b = bundle();
        for rendu in [
            action_label(&b, &Locale::FrCh, "parti.en_vacances"),
            entity_type_label(&b, &Locale::DeCh, "licorne"),
            actor_type_label(&b, &Locale::ItCh, "robot"),
        ] {
            assert!(
                !rendu.starts_with("audit-log-"),
                "le repli a laissé fuir une clé brute : {rendu}"
            );
        }
        assert_eq!(
            action_label(&b, &Locale::FrCh, "parti.en_vacances"),
            "parti.en_vacances"
        );
    }

    #[test]
    fn chaque_code_connu_se_traduit_dans_les_quatre_locales() {
        // ⚠️ Ce test ne remplace PAS la garde `tests/audit_label_registry.rs`.
        // `I18nBundle::format` replie sur le français : une clé présente en
        // `fr-CH` et absente en `de-CH` rendrait ici le texte français et
        // passerait. Seule la garde, qui lit les quatre `.ftl` directement, voit
        // une locale en retard.
        let b = bundle();
        for locale in Locale::ALL {
            for (prefixe, liste, traduire) in [
                (
                    PREFIX_ENTITY,
                    ENTITY_TYPES,
                    entity_type_label as fn(&I18nBundle, &Locale, &str) -> String,
                ),
                (PREFIX_ACTION, ACTIONS, action_label),
                (PREFIX_ACTOR_TYPE, ACTOR_TYPES, actor_type_label),
            ] {
                for &code in liste {
                    let rendu = traduire(&b, &locale, code);
                    assert_ne!(
                        rendu,
                        message_key(prefixe, code),
                        "aucune traduction pour {code} en {locale}"
                    );
                    assert!(!rendu.is_empty(), "libellé vide pour {code} en {locale}");
                }
            }
        }
    }
}
