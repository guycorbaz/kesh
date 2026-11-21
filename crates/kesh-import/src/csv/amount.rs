//! Montants à **forme fermée** des exports de prestataires de paiement
//! (Story 38-11a ; repris par Wise et Stripe, Epics 39 et 40).
//!
//! Les exports PayPal (et ceux des autres prestataires lus en anglais) écrivent
//! leurs montants à la manière anglo-saxonne : point décimal, virgule des
//! milliers facultative (`1,234.56`). Le lecteur du CSV bancaire, lui, retire
//! toute virgule sans la contrôler (`parse_amount` de `csv/parser.rs`) : sous
//! cette règle, `1,50` — montant d'un compte à virgule décimale — se lirait
//! `150`, erreur de facteur 100 que des contrôles de cohérence entre brut,
//! frais et net ne verraient pas, les trois étant faussés ensemble.
//!
//! D'où une forme fermée, décrite par une expression régulière : la virgule
//! n'est admise que comme séparateur de milliers (groupes de trois chiffres
//! exactement), puis retirée ; toute autre écriture est refusée.

use std::sync::LazyLock;

use regex::Regex;
use rust_decimal::Decimal;

/// `-?`, puis des chiffres nus ou groupés par milliers à la virgule, puis au
/// plus deux décimales au point. `[0-9]` et non `\d`, qui admettrait les
/// chiffres non latins d'Unicode.
static DOT_DECIMAL_AMOUNT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^-?([0-9]+|[0-9]{1,3}(,[0-9]{3})+)(\.[0-9]{1,2})?$")
        .expect("expression régulière des montants valide")
});

/// Lit un montant à point décimal et virgule des milliers facultative, après
/// `trim` : `12`, `-3.2`, `1,234.56`, `-1,000,000.00`.
///
/// Rend `None` pour toute autre forme : virgule décimale (`1,50`),
/// séparateurs européens (`1.500,00`), espace ou apostrophe des milliers,
/// signe `+`, plus de deux décimales, cellule vide.
pub fn parse_dot_decimal_amount(raw: &str) -> Option<Decimal> {
    let raw = raw.trim();
    if !DOT_DECIMAL_AMOUNT.is_match(raw) {
        return None;
    }
    Decimal::from_str_exact(&raw.replace(',', "")).ok()
}

/// Lit un montant **d'une API** de prestataire (PayPal, puis Wise et Stripe
/// s'ils écrivent leurs montants en chaînes) : la forme de
/// [`parse_dot_decimal_amount`], **sans virgule des milliers** — une API
/// n'en écrit jamais ; une valeur qui en porte serait un défaut du prestataire,
/// refusé plutôt que lu sans alerte (revue P1 de la 38-12a, F13).
pub fn parse_api_amount(raw: &str) -> Option<Decimal> {
    if raw.contains(',') {
        return None;
    }
    parse_dot_decimal_amount(raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn accepts_the_closed_form() {
        assert_eq!(parse_dot_decimal_amount("12"), Some(dec!(12)));
        assert_eq!(parse_dot_decimal_amount(" -3.2 "), Some(dec!(-3.2)));
        assert_eq!(parse_dot_decimal_amount("0.00"), Some(dec!(0.00)));
        assert_eq!(parse_dot_decimal_amount("1234.56"), Some(dec!(1234.56)));
        assert_eq!(parse_dot_decimal_amount("1,234.56"), Some(dec!(1234.56)));
        assert_eq!(
            parse_dot_decimal_amount("-1,000,000.00"),
            Some(dec!(-1000000.00))
        );
    }

    #[test]
    fn refuses_every_other_form() {
        for raw in [
            "",
            " ",
            "1,50",
            "1.500,00",
            "1 500.00",
            "1'500.00",
            "+1.00",
            "1.234",
            "1,2345.00",
            "12,34",
            ",123",
            "1,",
            "1.",
            ".5",
            "--1",
            "1e3",
            "١٢",
            "CHF 1.00",
        ] {
            assert_eq!(
                parse_dot_decimal_amount(raw),
                None,
                "{raw:?} doit être refusé"
            );
        }
    }

    #[test]
    fn api_amount_refuses_the_thousands_comma() {
        assert_eq!(parse_api_amount("1234.56"), Some(dec!(1234.56)));
        assert_eq!(parse_api_amount("-0.50"), Some(dec!(-0.50)));
        assert_eq!(parse_api_amount("1,234"), None);
        assert_eq!(parse_api_amount("1,234.56"), None);
        assert_eq!(parse_api_amount("1.234,5"), None);
        // Le reste de la forme fermée tient toujours.
        assert_eq!(parse_api_amount("1.234"), None);
        assert_eq!(parse_api_amount("+1.00"), None);
    }
}
