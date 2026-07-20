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

/// Retourne `(subject, body)` par défaut pour `(template_type, language, level_number)`.
/// `level_number` n'est significatif que pour `InvoiceReminder` (0/≥4 = générique ;
/// 1..3 = ton en escalade). Pour `InvoiceSend`, `level_number` est ignoré (toujours 0).
pub fn default_template(
    template_type: EmailTemplateType,
    language: Language,
    level_number: i16,
) -> (&'static str, &'static str) {
    match template_type {
        EmailTemplateType::InvoiceSend => invoice_send_default(language),
        EmailTemplateType::InvoiceReminder => reminder_default(language, level_number),
    }
}

fn invoice_send_default(language: Language) -> (&'static str, &'static str) {
    match language {
        Language::Fr => (
            "Facture {invoiceNumber} — {companyName}",
            "{salutation},\n\n\
             Veuillez trouver ci-joint la facture {invoiceNumber} d'un montant de {amount}, \
             à régler d'ici au {dueDate}.\n\n\
             Nous vous remercions de votre confiance.\n\n\
             {companyName}",
        ),
        Language::De => (
            "Rechnung {invoiceNumber} — {companyName}",
            "{salutation}\n\n\
             Anbei erhalten Sie die Rechnung {invoiceNumber} über {amount}, \
             zahlbar bis am {dueDate}.\n\n\
             Wir danken Ihnen für Ihr Vertrauen.\n\n\
             {companyName}",
        ),
        Language::It => (
            "Fattura {invoiceNumber} — {companyName}",
            "{salutation},\n\n\
             In allegato trova la fattura {invoiceNumber} per un importo di {amount}, \
             da saldare entro il {dueDate}.\n\n\
             La ringraziamo per la fiducia accordataci.\n\n\
             {companyName}",
        ),
        Language::En => (
            "Invoice {invoiceNumber} — {companyName}",
            "{salutation},\n\n\
             Please find attached invoice {invoiceNumber} for an amount of {amount}, \
             due by {dueDate}.\n\n\
             Thank you for your business.\n\n\
             {companyName}",
        ),
    }
}

/// Défauts de rappel par langue et niveau. Niveaux 1/2/3 = ton en escalade
/// (courtois → ferme → mise en demeure avant poursuite) ; le bras `_` (niveau 0
/// générique ou ≥ 4) est un rappel neutre-ferme. N'utilise que des tokens déclarés
/// dans `allowed_variables(InvoiceReminder)` (`reminderLevel` volontairement non utilisé).
fn reminder_default(language: Language, level_number: i16) -> (&'static str, &'static str) {
    match (language, level_number) {
        // ---- Français ----
        (Language::Fr, 1) => (
            "Rappel de paiement — facture {invoiceNumber}",
            "{salutation},\n\n\
             Sauf erreur de notre part, la facture {invoiceNumber} d'un montant de {amount}, \
             échue le {dueDate}, demeure impayée à ce jour ({daysOverdue} jours de retard).\n\n\
             Nous vous remercions de bien vouloir régler le montant dû de {totalDue} dans les meilleurs délais.\n\n\
             Avec nos meilleures salutations,\n{companyName}",
        ),
        (Language::Fr, 2) => (
            "2e rappel — facture {invoiceNumber}",
            "{salutation},\n\n\
             Malgré notre premier rappel, la facture {invoiceNumber} de {amount}, échue le {dueDate}, \
             reste impayée ({daysOverdue} jours de retard).\n\n\
             Des frais de rappel de {reminderFee} ont été ajoutés ; le montant total dû s'élève désormais à {totalDue}. \
             Nous vous invitons à le régler sous huitaine.\n\n\
             Avec nos salutations,\n{companyName}",
        ),
        (Language::Fr, 3) => (
            "Dernier rappel avant poursuite — facture {invoiceNumber}",
            "{salutation},\n\n\
             La facture {invoiceNumber} de {amount}, échue le {dueDate}, demeure impayée malgré nos rappels \
             ({daysOverdue} jours de retard).\n\n\
             Nous vous mettons en demeure de régler le montant total dû de {totalDue} \
             (frais de rappel de {reminderFee} inclus) sans délai. À défaut de paiement, \
             nous engagerons une procédure de recouvrement sans autre avis.\n\n\
             {companyName}",
        ),
        (Language::Fr, _) => (
            "Rappel de paiement — facture {invoiceNumber}",
            "{salutation},\n\n\
             La facture {invoiceNumber} d'un montant de {amount}, échue le {dueDate}, \
             reste impayée ({daysOverdue} jours de retard).\n\n\
             Nous vous prions de régler le montant total dû de {totalDue} dans les meilleurs délais.\n\n\
             Avec nos meilleures salutations,\n{companyName}",
        ),
        // ---- Deutsch ----
        (Language::De, 1) => (
            "Zahlungserinnerung — Rechnung {invoiceNumber}",
            "{salutation}\n\n\
             Sofern sich unsere Angaben mit den Ihren decken, ist die Rechnung {invoiceNumber} über {amount}, \
             fällig am {dueDate}, bis heute unbeglichen ({daysOverdue} Tage überfällig).\n\n\
             Wir bitten Sie, den offenen Betrag von {totalDue} baldmöglichst zu begleichen.\n\n\
             Freundliche Grüsse\n{companyName}",
        ),
        (Language::De, 2) => (
            "2. Mahnung — Rechnung {invoiceNumber}",
            "{salutation}\n\n\
             Trotz unserer ersten Erinnerung ist die Rechnung {invoiceNumber} über {amount}, \
             fällig am {dueDate}, weiterhin offen ({daysOverdue} Tage überfällig).\n\n\
             Wir haben Mahngebühren von {reminderFee} erhoben; der Gesamtbetrag beläuft sich nun auf {totalDue}. \
             Wir bitten um Begleichung innert acht Tagen.\n\n\
             Freundliche Grüsse\n{companyName}",
        ),
        (Language::De, 3) => (
            "Letzte Mahnung vor Betreibung — Rechnung {invoiceNumber}",
            "{salutation}\n\n\
             Die Rechnung {invoiceNumber} über {amount}, fällig am {dueDate}, ist trotz mehrfacher Mahnung \
             unbeglichen ({daysOverdue} Tage überfällig).\n\n\
             Wir fordern Sie letztmals auf, den Gesamtbetrag von {totalDue} (inkl. Mahngebühren von {reminderFee}) \
             unverzüglich zu begleichen. Andernfalls leiten wir ohne weitere Ankündigung die Betreibung ein.\n\n\
             {companyName}",
        ),
        (Language::De, _) => (
            "Mahnung — Rechnung {invoiceNumber}",
            "{salutation}\n\n\
             Die Rechnung {invoiceNumber} über {amount}, fällig am {dueDate}, ist offen \
             ({daysOverdue} Tage überfällig).\n\n\
             Wir bitten Sie, den Gesamtbetrag von {totalDue} baldmöglichst zu begleichen.\n\n\
             Freundliche Grüsse\n{companyName}",
        ),
        // ---- Italiano ----
        (Language::It, 1) => (
            "Sollecito di pagamento — fattura {invoiceNumber}",
            "{salutation},\n\n\
             Salvo errore da parte nostra, la fattura {invoiceNumber} di {amount}, scaduta il {dueDate}, \
             risulta ancora non saldata ({daysOverdue} giorni di ritardo).\n\n\
             La preghiamo di saldare l'importo dovuto di {totalDue} al più presto.\n\n\
             Distinti saluti,\n{companyName}",
        ),
        (Language::It, 2) => (
            "2° sollecito — fattura {invoiceNumber}",
            "{salutation},\n\n\
             Nonostante il nostro primo sollecito, la fattura {invoiceNumber} di {amount}, scaduta il {dueDate}, \
             risulta ancora non saldata ({daysOverdue} giorni di ritardo).\n\n\
             Sono state applicate spese di sollecito di {reminderFee}; l'importo totale dovuto ammonta ora a {totalDue}. \
             La invitiamo a saldarlo entro otto giorni.\n\n\
             Distinti saluti,\n{companyName}",
        ),
        (Language::It, 3) => (
            "Ultimo sollecito prima dell'esecuzione — fattura {invoiceNumber}",
            "{salutation},\n\n\
             La fattura {invoiceNumber} di {amount}, scaduta il {dueDate}, risulta non saldata nonostante i nostri solleciti \
             ({daysOverdue} giorni di ritardo).\n\n\
             La diffidiamo a saldare l'importo totale dovuto di {totalDue} (spese di sollecito di {reminderFee} incluse) \
             senza indugio. In mancanza di pagamento, avvieremo una procedura esecutiva senza ulteriore avviso.\n\n\
             {companyName}",
        ),
        (Language::It, _) => (
            "Sollecito — fattura {invoiceNumber}",
            "{salutation},\n\n\
             La fattura {invoiceNumber} di {amount}, scaduta il {dueDate}, risulta non saldata \
             ({daysOverdue} giorni di ritardo).\n\n\
             La preghiamo di saldare l'importo totale dovuto di {totalDue} al più presto.\n\n\
             Distinti saluti,\n{companyName}",
        ),
        // ---- English ----
        (Language::En, 1) => (
            "Payment reminder — invoice {invoiceNumber}",
            "{salutation},\n\n\
             Unless our records are mistaken, invoice {invoiceNumber} for {amount}, due on {dueDate}, \
             remains unpaid ({daysOverdue} days overdue).\n\n\
             We kindly ask you to settle the amount due of {totalDue} at your earliest convenience.\n\n\
             Kind regards,\n{companyName}",
        ),
        (Language::En, 2) => (
            "Second reminder — invoice {invoiceNumber}",
            "{salutation},\n\n\
             Despite our first reminder, invoice {invoiceNumber} for {amount}, due on {dueDate}, \
             remains unpaid ({daysOverdue} days overdue).\n\n\
             A reminder fee of {reminderFee} has been added; the total amount due is now {totalDue}. \
             Please settle it within eight days.\n\n\
             Kind regards,\n{companyName}",
        ),
        (Language::En, 3) => (
            "Final reminder before debt collection — invoice {invoiceNumber}",
            "{salutation},\n\n\
             Invoice {invoiceNumber} for {amount}, due on {dueDate}, remains unpaid despite our reminders \
             ({daysOverdue} days overdue).\n\n\
             We formally request payment of the total amount due of {totalDue} (including a reminder fee of {reminderFee}) \
             without delay. Failing payment, we will initiate debt-collection proceedings without further notice.\n\n\
             {companyName}",
        ),
        (Language::En, _) => (
            "Payment reminder — invoice {invoiceNumber}",
            "{salutation},\n\n\
             Invoice {invoiceNumber} for {amount}, due on {dueDate}, remains unpaid \
             ({daysOverdue} days overdue).\n\n\
             Please settle the total amount due of {totalDue} at your earliest convenience.\n\n\
             Kind regards,\n{companyName}",
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
            // Niveaux à couvrir : 0 pour invoice_send ; 0..=4 pour invoice_reminder
            // (0/4 = générique, 1..3 = spécifiques) — le bras `_` doit aussi être sain.
            let levels: &[i16] = match template_type {
                EmailTemplateType::InvoiceSend => &[0],
                EmailTemplateType::InvoiceReminder => &[0, 1, 2, 3, 4],
            };
            for language in [Language::Fr, Language::De, Language::It, Language::En] {
                for &level in levels {
                    let (subject, body) = default_template(template_type, language, level);
                    let combined = format!("{subject} {body}");
                    let tokens = kesh_core::email_template_engine::extract_tokens(&combined);
                    for token in &tokens {
                        assert!(
                            allowed.contains(&token.as_str()),
                            "défaut {template_type:?}/{language:?}/niv{level} contient un token non déclaré : {{{token}}}"
                        );
                    }
                }
            }
        }
    }
}
