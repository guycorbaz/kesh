//! Whitelist partagée des taux TVA suisses (v0.1, depuis 01.01.2024).
//!
//! Extraite depuis `products.rs` (Story 4.2) pour être réutilisée par
//! `invoices.rs` (Story 5.1) — DRY strict, une seule source de vérité.
//!
//! Note : `rust_decimal::Decimal::eq` ignore le scale (`8.1 == 8.10 == 8.100`),
//! donc la comparaison est robuste aux variations de représentation client.

use std::str::FromStr;
use std::sync::LazyLock;

use rust_decimal::Decimal;

static ALLOWED_VAT_RATES: LazyLock<[Decimal; 4]> = LazyLock::new(|| {
    [
        Decimal::from_str("0.00").expect("VAT whitelist literal must parse"),
        Decimal::from_str("2.60").expect("VAT whitelist literal must parse"),
        Decimal::from_str("3.80").expect("VAT whitelist literal must parse"),
        Decimal::from_str("8.10").expect("VAT whitelist literal must parse"),
    ]
});

/// Liste des taux TVA autorisés (Decimal, comparable par égalité sans tenir compte du scale).
pub fn allowed_vat_rates() -> &'static [Decimal] {
    &*ALLOWED_VAT_RATES
}

/// Retourne `true` si le taux fourni est dans la whitelist.
pub fn validate_vat_rate(rate: &Decimal) -> bool {
    ALLOWED_VAT_RATES.iter().any(|r| r == rate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn accepts_all_swiss_rates() {
        for s in ["0.00", "2.60", "3.80", "8.10"] {
            assert!(validate_vat_rate(&Decimal::from_str(s).unwrap()));
        }
    }

    #[test]
    fn rejects_unknown_rates() {
        assert!(!validate_vat_rate(&dec!(7.70))); // ancien taux pré-2024
        assert!(!validate_vat_rate(&dec!(99.99)));
        assert!(!validate_vat_rate(&dec!(-1.00)));
    }

    #[test]
    fn scale_invariant() {
        // 8.1 == 8.10 == 8.100 côté Decimal::eq.
        assert!(validate_vat_rate(&Decimal::from_str("8.1").unwrap()));
        assert!(validate_vat_rate(&Decimal::from_str("8.100").unwrap()));
    }
}
