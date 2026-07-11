//! Moteur de substitution `{var}` pour les templates d'e-mail (Epic 20 #224,
//! Story 20-1).
//!
//! Pur : aucune connaissance de `EmailTemplateType`/`Language` (définis dans
//! `kesh-db`, qui dépend de ce crate — pas l'inverse). Le caller passe la
//! liste des variables autorisées (`&[&str]`) et la map de valeurs déjà
//! résolues/formatées (`&HashMap<String, String>`).
//!
//! **Invariant single-pass** : toutes les fonctions ci-dessous scannent le
//! texte source une seule fois et n'avancent jamais que sur ce texte source.
//! [`render`] ne réanalyse jamais le texte déjà produit — si une valeur
//! substituée contient elle-même une séquence `{token}`, cette séquence
//! n'est jamais interprétée (anti-injection).

use std::collections::{HashMap, HashSet};

/// Scanne `text` une seule fois et retourne les noms de tokens `{nom}`
/// rencontrés, dédupliqués (comparaison exacte, sensible à la casse), dans
/// l'ordre d'apparition.
pub fn extract_tokens(text: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for name in ScanTokens::new(text) {
        if seen.insert(name) {
            out.push(name.to_string());
        }
    }
    out
}

/// Valide que tous les tokens `{nom}` de `subject`+`body` (scannés dans cet
/// ordre) appartiennent à `allowed`. `Err(unknown)` liste les tokens hors
/// `allowed`, dédupliqués, dans l'ordre d'apparition. `Ok(())` si aucun.
pub fn validate_tokens(subject: &str, body: &str, allowed: &[&str]) -> Result<(), Vec<String>> {
    let mut seen = HashSet::new();
    let mut unknown = Vec::new();
    for name in ScanTokens::new(subject).chain(ScanTokens::new(body)) {
        if !allowed.contains(&name) && seen.insert(name) {
            unknown.push(name.to_string());
        }
    }
    if unknown.is_empty() {
        Ok(())
    } else {
        Err(unknown)
    }
}

/// Substitue les tokens `{nom}` connus dans `vars` par leur valeur. Un token
/// inconnu est laissé **littéral** (jamais d'erreur — rendu infaillible).
///
/// Single-pass strict sur `template` : l'avancement du scan (`rest`) ne
/// porte jamais sur `result` (le texte déjà produit, y compris les valeurs
/// substituées) — une valeur qui contient elle-même `{token}` n'est donc
/// jamais réinterprétée.
pub fn render(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = String::with_capacity(template.len());
    let mut rest = template;
    loop {
        let Some(open) = rest.find('{') else {
            result.push_str(rest);
            break;
        };
        result.push_str(&rest[..open]);
        let after_open = &rest[open + 1..];
        match after_open.find('}') {
            Some(close) => {
                let name = &after_open[..close];
                match vars.get(name) {
                    Some(value) => result.push_str(value),
                    None => {
                        result.push('{');
                        result.push_str(name);
                        result.push('}');
                    }
                }
                rest = &after_open[close + 1..];
            }
            None => {
                // '{' sans '}' fermant correspondant : littéral, fin du scan.
                result.push('{');
                result.push_str(after_open);
                break;
            }
        }
    }
    result
}

/// Itérateur single-pass sur les noms de tokens `{nom}` d'un texte.
/// `'{' '/' '}'` sont ASCII (1 octet) : les points de découpe sont toujours
/// des frontières UTF-8 valides quel que soit le contenu entre eux (accents
/// FR/DE/IT sans risque de panic sur slicing).
struct ScanTokens<'a> {
    rest: &'a str,
}

impl<'a> ScanTokens<'a> {
    fn new(text: &'a str) -> Self {
        Self { rest: text }
    }
}

impl<'a> Iterator for ScanTokens<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        let open = self.rest.find('{')?;
        let after_open = &self.rest[open + 1..];
        match after_open.find('}') {
            Some(close) => {
                let name = &after_open[..close];
                self.rest = &after_open[close + 1..];
                Some(name)
            }
            None => {
                self.rest = "";
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_substitutes_known_token() {
        let mut vars = HashMap::new();
        vars.insert("amount".to_string(), "CHF 100.00".to_string());
        assert_eq!(
            render("Montant dû : {amount}.", &vars),
            "Montant dû : CHF 100.00."
        );
    }

    #[test]
    fn render_leaves_unknown_token_literal() {
        let vars = HashMap::new();
        assert_eq!(
            render("Bonjour {contactName}.", &vars),
            "Bonjour {contactName}."
        );
    }

    #[test]
    fn render_never_rescans_substituted_value() {
        // Anti-injection : la valeur substituée contient elle-même un token
        // qui existe dans `vars` — il ne doit JAMAIS être réinterprété.
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "{other}".to_string());
        vars.insert("other".to_string(), "REMPLACÉ".to_string());
        assert_eq!(render("Salut {name}", &vars), "Salut {other}");
    }

    #[test]
    fn render_handles_unclosed_brace_literally() {
        let vars = HashMap::new();
        assert_eq!(render("Texte { non fermé", &vars), "Texte { non fermé");
    }

    #[test]
    fn extract_tokens_dedupes_preserving_order() {
        let tokens = extract_tokens("{amount} et encore {amount}, puis {dueDate}");
        assert_eq!(tokens, vec!["amount".to_string(), "dueDate".to_string()]);
    }

    #[test]
    fn extract_tokens_is_case_sensitive() {
        let tokens = extract_tokens("{Amount} {amount}");
        assert_eq!(tokens, vec!["Amount".to_string(), "amount".to_string()]);
    }

    #[test]
    fn validate_tokens_accepts_only_allowed() {
        let allowed = &["amount", "dueDate"];
        assert!(
            validate_tokens("Facture", "Montant {amount}, échéance {dueDate}", allowed).is_ok()
        );
    }

    #[test]
    fn validate_tokens_rejects_unknown_and_lists_them_deduped() {
        let allowed = &["amount"];
        let err = validate_tokens("{foo}", "{amount} {bar} {foo}", allowed).unwrap_err();
        assert_eq!(err, vec!["foo".to_string(), "bar".to_string()]);
    }
}
