//! Textes par défaut des templates d'e-mail (Epic 20 #224, Story 20-1).
//!
//! Constantes Rust plutôt que fichiers `.ftl` : Fluent ignore silencieusement
//! une variable non fournie (`kesh_i18n::loader` ne traite pas
//! `is_missing_variable_error` comme bloquant), ce qui est incompatible avec
//! la validation stricte au save. La syntaxe `{var}` est aussi volontairement
//! différente de `{ $var }` Fluent pour éviter toute confusion visuelle.
//!
//! Chaque texte n'utilise que des tokens déclarés dans
//! `EmailTemplateType::allowed_variables()` — auto-cohérence vérifiée par
//! un test unitaire (aucun défaut cassé ne doit pouvoir passer inaperçu
//! avant le rendu réel, Story 20-3b).

use super::{EmailTemplateType, Language};

/// Retourne `(subject, body)` par défaut pour `(template_type, language)`.
pub fn default_template(
    template_type: EmailTemplateType,
    language: Language,
) -> (&'static str, &'static str) {
    match (template_type, language) {
        (EmailTemplateType::InvoiceSend, Language::Fr) => (
            "Facture {invoiceNumber} — {companyName}",
            "{salutation},\n\n\
             Veuillez trouver ci-joint la facture {invoiceNumber} d'un montant de {amount}, \
             à régler d'ici au {dueDate}.\n\n\
             Nous vous remercions de votre confiance.\n\n\
             {companyName}",
        ),
        (EmailTemplateType::InvoiceSend, Language::De) => (
            "Rechnung {invoiceNumber} — {companyName}",
            "{salutation}\n\n\
             Anbei erhalten Sie die Rechnung {invoiceNumber} über {amount}, \
             zahlbar bis am {dueDate}.\n\n\
             Wir danken Ihnen für Ihr Vertrauen.\n\n\
             {companyName}",
        ),
        (EmailTemplateType::InvoiceSend, Language::It) => (
            "Fattura {invoiceNumber} — {companyName}",
            "{salutation},\n\n\
             In allegato trova la fattura {invoiceNumber} per un importo di {amount}, \
             da saldare entro il {dueDate}.\n\n\
             La ringraziamo per la fiducia accordataci.\n\n\
             {companyName}",
        ),
        (EmailTemplateType::InvoiceSend, Language::En) => (
            "Invoice {invoiceNumber} — {companyName}",
            "{salutation},\n\n\
             Please find attached invoice {invoiceNumber} for an amount of {amount}, \
             due by {dueDate}.\n\n\
             Thank you for your business.\n\n\
             {companyName}",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Auto-cohérence : chaque défaut ne référence que des tokens déclarés
    /// par `allowed_variables()` de son type (AC #8).
    #[test]
    fn all_defaults_only_use_allowed_variables() {
        for template_type in EmailTemplateType::ALL {
            let allowed = template_type.allowed_variables();
            for language in [Language::Fr, Language::De, Language::It, Language::En] {
                let (subject, body) = default_template(template_type, language);
                let combined = format!("{subject} {body}");
                let tokens = kesh_core::email_template_engine::extract_tokens(&combined);
                for token in &tokens {
                    assert!(
                        allowed.contains(&token.as_str()),
                        "défaut {template_type:?}/{language:?} contient un token non déclaré : {{{token}}}"
                    );
                }
            }
        }
    }
}
