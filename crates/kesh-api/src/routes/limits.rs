//! Plafonds et helpers de validation partagés entre routes (Story 5.1).
//!
//! Extrait depuis `products.rs` pour être réutilisé par `invoices.rs`.
//! Ces limites sont anti-DoS / anti-troncature et s'appliquent au niveau
//! handler (pas DB), cohérent avec le pattern déjà établi.

use std::str::FromStr;
use std::sync::LazyLock;

use rust_decimal::Decimal;

/// Plafond `unit_price` : 1 milliard CHF. Anti-overflow (ligne = prix × qty).
pub static MAX_UNIT_PRICE: LazyLock<Decimal> =
    LazyLock::new(|| Decimal::from_str("1000000000").expect("MAX_UNIT_PRICE literal must parse"));

/// Plafond `quantity` : 1 million. Anti-overflow `qty × unit_price`.
pub static MAX_QUANTITY: LazyLock<Decimal> =
    LazyLock::new(|| Decimal::from_str("1000000").expect("MAX_QUANTITY literal must parse"));

/// Plafond `line_total` (= quantity × unit_price) : 10¹². Anti-overflow DECIMAL(19,4)
/// sur l'agrégation `total_amount = Σ line_total` sur N lignes (N ≤ MAX_LINES).
/// 10¹² × 200 lignes ≈ 2·10¹⁴, reste sous le max DECIMAL(19,4) (~10¹⁵).
pub static MAX_LINE_TOTAL: LazyLock<Decimal> = LazyLock::new(|| {
    Decimal::from_str("1000000000000").expect("MAX_LINE_TOTAL literal must parse")
});

/// Scale maximal pour `DECIMAL(19,4)` — évite une troncature silencieuse.
pub const MAX_DECIMAL_SCALE: u32 = 4;

/// Retourne `true` si le scale (nombre de décimales) est ≤ `max_scale`.
pub fn scale_within(value: &Decimal, max_scale: u32) -> bool {
    value.scale() <= max_scale
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn scale_within_accepts_up_to_max() {
        assert!(scale_within(&dec!(1.0), 4));
        assert!(scale_within(&dec!(1.1234), 4));
        assert!(!scale_within(&dec!(1.12345), 4));
    }

    #[test]
    fn caps_are_sane() {
        assert_eq!(*MAX_UNIT_PRICE, dec!(1000000000));
        assert_eq!(*MAX_QUANTITY, dec!(1000000));
    }
}
