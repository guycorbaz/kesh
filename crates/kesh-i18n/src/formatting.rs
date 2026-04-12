//! Formatage suisse : montants (apostrophe U+2019) et dates (dd.mm.yyyy).

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

/// Apostrophe typographique (U+2019) — séparateur de milliers suisse.
const THOUSANDS_SEP: char = '\u{2019}';

/// Formate un montant en convention suisse : `1'234.56`.
///
/// - 2 décimales toujours (centimes)
/// - Apostrophe typographique (U+2019) comme séparateur de milliers
/// - Signe `-` préfixé pour les négatifs
/// - Arrondi au centime (2 décimales)
pub fn format_money(amount: &Decimal) -> String {
    let rounded = amount.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero);
    let is_negative = rounded.is_sign_negative() && !rounded.is_zero();
    let abs = if is_negative { -rounded } else { rounded };

    // Séparer partie entière et décimale
    let s = format!("{:.2}", abs);
    let parts: Vec<&str> = s.split('.').collect();
    let integer_part = parts[0];
    let decimal_part = parts[1];

    // Insérer les séparateurs de milliers
    let formatted_int = insert_thousands_sep(integer_part);

    if is_negative {
        format!("-{formatted_int}.{decimal_part}")
    } else {
        format!("{formatted_int}.{decimal_part}")
    }
}

/// Formate une date en convention suisse : `dd.mm.yyyy`.
pub fn format_date(date: &NaiveDate) -> String {
    date.format("%d.%m.%Y").to_string()
}

/// Formate un datetime en convention suisse : `dd.mm.yyyy HH:MM`.
pub fn format_datetime(dt: &NaiveDateTime) -> String {
    dt.format("%d.%m.%Y %H:%M").to_string()
}

/// Insère l'apostrophe typographique tous les 3 chiffres depuis la droite.
fn insert_thousands_sep(integer: &str) -> String {
    let digits: Vec<char> = integer.chars().collect();
    let len = digits.len();
    if len <= 3 {
        return integer.to_string();
    }

    let mut result = String::with_capacity(len + len / 3);
    for (i, ch) in digits.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(THOUSANDS_SEP);
        }
        result.push(*ch);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    // --- format_money ---

    #[test]
    fn money_simple() {
        assert_eq!(format_money(&dec!(1234.56)), "1\u{2019}234.56");
    }

    #[test]
    fn money_negative() {
        assert_eq!(format_money(&dec!(-1234.56)), "-1\u{2019}234.56");
    }

    #[test]
    fn money_zero() {
        assert_eq!(format_money(&dec!(0)), "0.00");
    }

    #[test]
    fn money_below_thousand() {
        assert_eq!(format_money(&dec!(999.00)), "999.00");
    }

    #[test]
    fn money_large() {
        assert_eq!(
            format_money(&dec!(1234567.89)),
            "1\u{2019}234\u{2019}567.89"
        );
    }

    #[test]
    fn money_centimes_only() {
        assert_eq!(format_money(&dec!(0.50)), "0.50");
    }

    #[test]
    fn money_rounding() {
        assert_eq!(format_money(&dec!(1234.555)), "1\u{2019}234.56");
    }

    #[test]
    fn money_exact_thousand() {
        assert_eq!(format_money(&dec!(1000)), "1\u{2019}000.00");
    }

    #[test]
    fn money_million() {
        assert_eq!(format_money(&dec!(1000000)), "1\u{2019}000\u{2019}000.00");
    }

    // --- format_date ---

    #[test]
    fn date_standard() {
        let d = NaiveDate::from_ymd_opt(2026, 4, 3).unwrap();
        assert_eq!(format_date(&d), "03.04.2026");
    }

    #[test]
    fn date_new_year() {
        let d = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        assert_eq!(format_date(&d), "01.01.2026");
    }

    #[test]
    fn date_end_year() {
        let d = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
        assert_eq!(format_date(&d), "31.12.2026");
    }

    // --- format_datetime ---

    #[test]
    fn datetime_standard() {
        let dt = NaiveDate::from_ymd_opt(2026, 4, 3)
            .unwrap()
            .and_hms_opt(14, 30, 0)
            .unwrap();
        assert_eq!(format_datetime(&dt), "03.04.2026 14:30");
    }

    #[test]
    fn datetime_midnight() {
        let dt = NaiveDate::from_ymd_opt(2026, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        assert_eq!(format_datetime(&dt), "01.01.2026 00:00");
    }
}
