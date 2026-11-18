//! Devises : la devise principale et les codes ISO 4217 admis pour un compte
//! (Story 37-1, Epic 37 multi-devises).
//!
//! La liste est **embarquée** (aucune dépendance réseau) et ne porte que des
//! **monnaies** actives : les codes réservés (`XXX`, `XTS`), les métaux
//! précieux (`XAU`, `XAG`, `XPT`, `XPD`), les droits de tirage spéciaux
//! (`XDR`), les unités de compte et de fonds (`XBA` à `XBD`, `XSU`, `XUA`,
//! `BOV`, `CHE`, `CHW`, `CLF`, `COU`, `MXV`, `USN`, `UYI`, `UYW`) et les codes
//! retirés en sont exclus. `CHF` y figure : la devise principale se reconnaît,
//! puis se normalise en « aucune devise » ([`normalize_foreign_currency`]).
//!
//! La liste ne valide qu'**à l'écriture** : une devise relue en base n'est
//! jamais recontrôlée, de sorte qu'un code retiré plus tard de la liste ne
//! fasse jamais échouer la lecture d'un compte.

use std::fmt;

use serde::Serialize;
use thiserror::Error;

/// Devise principale de la comptabilité. Elle ne change pas : un compte dans
/// cette devise ne porte **aucune** devise en base (`NULL`), une seule graphie.
pub const BASE_CURRENCY: &str = "CHF";

/// Codes ISO 4217 des monnaies actives, triés (recherche dichotomique).
///
/// Exclusions : voir la doc du module. Le test
/// `liste_triee_sans_doublon_ni_code_exclu` garde l'ordre et les exclusions.
pub const ISO_4217_CURRENCIES: &[&str] = &[
    "AED", "AFN", "ALL", "AMD", "AOA", "ARS", "AUD", "AWG", "AZN", "BAM", "BBD", "BDT", "BHD",
    "BIF", "BMD", "BND", "BOB", "BRL", "BSD", "BTN", "BWP", "BYN", "BZD", "CAD", "CDF", "CHF",
    "CLP", "CNY", "COP", "CRC", "CUP", "CVE", "CZK", "DJF", "DKK", "DOP", "DZD", "EGP", "ERN",
    "ETB", "EUR", "FJD", "FKP", "GBP", "GEL", "GHS", "GIP", "GMD", "GNF", "GTQ", "GYD", "HKD",
    "HNL", "HTG", "HUF", "IDR", "ILS", "INR", "IQD", "IRR", "ISK", "JMD", "JOD", "JPY", "KES",
    "KGS", "KHR", "KMF", "KPW", "KRW", "KWD", "KYD", "KZT", "LAK", "LBP", "LKR", "LRD", "LSL",
    "LYD", "MAD", "MDL", "MGA", "MKD", "MMK", "MNT", "MOP", "MRU", "MUR", "MVR", "MWK", "MXN",
    "MYR", "MZN", "NAD", "NGN", "NIO", "NOK", "NPR", "NZD", "OMR", "PAB", "PEN", "PGK", "PHP",
    "PKR", "PLN", "PYG", "QAR", "RON", "RSD", "RUB", "RWF", "SAR", "SBD", "SCR", "SDG", "SEK",
    "SGD", "SHP", "SLE", "SOS", "SRD", "SSP", "STN", "SVC", "SYP", "SZL", "THB", "TJS", "TMT",
    "TND", "TOP", "TRY", "TTD", "TWD", "TZS", "UAH", "UGX", "USD", "UYU", "UZS", "VED", "VES",
    "VND", "VUV", "WST", "XAF", "XCD", "XCG", "XOF", "XPF", "YER", "ZAR", "ZMW", "ZWG",
];

/// Une saisie de devise qui n'est pas un code de la liste ISO 4217 embarquée
/// (vide, non ASCII, autre longueur, ou code inconnu). La valeur reçue est
/// gardée telle quelle, pour le message.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("Devise inconnue : {0:?}")]
pub struct UnknownCurrency(pub String);

/// Code ISO 4217 validé contre [`ISO_4217_CURRENCIES`] : trois lettres
/// majuscules ASCII. Aucune instance invalide ne se construit.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(into = "String")]
pub struct CurrencyCode(String);

impl CurrencyCode {
    /// Valide une saisie : `trim`, puis refus de tout caractère non ASCII,
    /// puis **majuscules ASCII** (`to_ascii_uppercase`, jamais Unicode), puis
    /// recherche dans la liste embarquée.
    ///
    /// ```
    /// use kesh_core::currency::CurrencyCode;
    /// assert_eq!(CurrencyCode::parse(" usd ").unwrap().as_str(), "USD");
    /// assert!(CurrencyCode::parse("XAU").is_err());
    /// ```
    pub fn parse(input: &str) -> Result<Self, UnknownCurrency> {
        let trimmed = input.trim();
        if !trimmed.is_ascii() {
            return Err(UnknownCurrency(input.to_string()));
        }
        let upper = trimmed.to_ascii_uppercase();
        if ISO_4217_CURRENCIES.binary_search(&upper.as_str()).is_ok() {
            Ok(Self(upper))
        } else {
            Err(UnknownCurrency(input.to_string()))
        }
    }

    /// Le code, trois lettres majuscules.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// `true` pour la devise principale ([`BASE_CURRENCY`]).
    pub fn is_base(&self) -> bool {
        self.0 == BASE_CURRENCY
    }
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<CurrencyCode> for String {
    fn from(c: CurrencyCode) -> Self {
        c.0
    }
}

/// Devise d'un compte, telle qu'elle s'écrit en base : `Ok(None)` pour la
/// devise principale (`"CHF"`, `" chf "`…), `Ok(Some(code))` pour une autre
/// devise de la liste, `Err` sinon.
///
/// C'est la normalisation qui passe **avant tout autre contrôle** : `CHF`
/// n'est jamais « une devise donnée » (en mono-devise, il n'est donc jamais
/// refusé).
///
/// ```
/// use kesh_core::currency::normalize_foreign_currency;
/// assert_eq!(normalize_foreign_currency("chf").unwrap(), None);
/// assert_eq!(normalize_foreign_currency("eur").unwrap().unwrap().as_str(), "EUR");
/// ```
pub fn normalize_foreign_currency(input: &str) -> Result<Option<CurrencyCode>, UnknownCurrency> {
    let code = CurrencyCode::parse(input)?;
    Ok(if code.is_base() { None } else { Some(code) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn liste_triee_sans_doublon_ni_code_exclu() {
        for paire in ISO_4217_CURRENCIES.windows(2) {
            assert!(paire[0] < paire[1], "ordre ou doublon : {paire:?}");
        }
        for code in ISO_4217_CURRENCIES {
            assert_eq!(code.len(), 3, "{code}");
            assert!(code.bytes().all(|b| b.is_ascii_uppercase()), "{code}");
        }
        for exclu in [
            "XXX", "XTS", "XAU", "XAG", "XPT", "XPD", "XDR", "XBA", "XBB", "XBC", "XBD", "XSU",
            "XUA", "BOV", "CHE", "CHW", "CLF", "COU", "MXV", "USN", "UYI", "UYW", "HRK", "CUC",
            "SLL", "ZWL", "MRO", "STD",
        ] {
            assert!(
                CurrencyCode::parse(exclu).is_err(),
                "{exclu} ne doit pas être admis"
            );
        }
    }

    #[test]
    fn parse_normalise_casse_et_espaces() {
        assert_eq!(CurrencyCode::parse("usd").unwrap().as_str(), "USD");
        assert_eq!(CurrencyCode::parse("  Eur\t").unwrap().as_str(), "EUR");
        assert_eq!(CurrencyCode::parse("CHF").unwrap().as_str(), "CHF");
        assert!(CurrencyCode::parse("CHF").unwrap().is_base());
        assert!(!CurrencyCode::parse("JPY").unwrap().is_base());
    }

    #[test]
    fn parse_refuse_le_reste() {
        for mauvais in ["", " ", "US", "USDD", "ZZZ", "U$D", "123"] {
            assert_eq!(
                CurrencyCode::parse(mauvais),
                Err(UnknownCurrency(mauvais.to_string())),
                "{mauvais:?}"
            );
        }
        // Non ASCII : refusé, même si la majuscule Unicode donnerait un code
        // connu (ﬀ, ı turc, lettres pleine chasse).
        for mauvais in ["ＵＳＤ", "usdı", "eür", "ﬀr"] {
            assert!(CurrencyCode::parse(mauvais).is_err(), "{mauvais:?}");
        }
    }

    #[test]
    fn normalize_foreign_currency_ramene_chf_a_none() {
        assert_eq!(normalize_foreign_currency("CHF").unwrap(), None);
        assert_eq!(normalize_foreign_currency(" chf ").unwrap(), None);
        assert_eq!(
            normalize_foreign_currency("usd").unwrap().map(String::from),
            Some("USD".to_string())
        );
        assert!(normalize_foreign_currency("xau").is_err());
    }

    #[test]
    fn display_et_serialisation() {
        let c = CurrencyCode::parse("gbp").unwrap();
        assert_eq!(c.to_string(), "GBP");
        assert_eq!(serde_json::to_string(&c).unwrap(), "\"GBP\"");
    }

    #[test]
    fn base_currency_est_dans_la_liste() {
        assert!(ISO_4217_CURRENCIES.binary_search(&BASE_CURRENCY).is_ok());
    }
}
