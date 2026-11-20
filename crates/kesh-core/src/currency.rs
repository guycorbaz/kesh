//! Devises : la devise principale et les codes ISO 4217 admis pour un compte
//! (Story 37-1, Epic 37 multi-devises), et ceux admis pour un **cours de
//! change** (correctif `fix-cours-afc-bgn`).
//!
//! Deux listes **embarquées** (aucune dépendance réseau) :
//!
//! - [`ISO_4217_CURRENCIES`] ne porte que des **monnaies** actives : les codes
//!   réservés (`XXX`, `XTS`), les métaux précieux (`XAU`, `XAG`, `XPT`,
//!   `XPD`), les droits de tirage spéciaux (`XDR`), les unités de compte et de
//!   fonds (`XBA` à `XBD`, `XSU`, `XUA`, `BOV`, `CHE`, `CHW`, `CLF`, `COU`,
//!   `MXV`, `USN`, `UYI`, `UYW`) et les codes retirés en sont exclus. `CHF` y
//!   figure : la devise principale se reconnaît, puis se normalise en « aucune
//!   devise » ([`normalize_foreign_currency`]). C'est la liste d'un **compte** :
//!   [`CurrencyCode::parse`].
//! - [`WITHDRAWN_CURRENCIES`] porte des monnaies **retirées** (lev bulgare,
//!   kuna croate, monnaies remplacées par l'euro…) dont l'AFC publie les cours
//!   pour les périodes où elles avaient cours. Elles ne s'admettent que pour un
//!   **cours** : [`CurrencyCode::parse_rate_currency`], [`is_rate_currency`].
//!   Un compte ne s'ouvre jamais dans une monnaie retirée.
//!
//! Les listes ne valident qu'**à l'écriture** : une devise relue en base n'est
//! jamais recontrôlée, de sorte qu'un code retiré plus tard de la liste ne
//! fasse jamais échouer la lecture d'un compte ou d'un cours.

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

/// Codes ISO 4217 **retirés** dont l'AFC publie des cours, triés (recherche
/// dichotomique) et disjoints de [`ISO_4217_CURRENCIES`].
///
/// Admis pour un **cours de change** seulement ([`is_rate_currency`],
/// [`CurrencyCode::parse_rate_currency`]), jamais pour un compte : un cours
/// d'une monnaie retirée sert à reprendre ou à vérifier des livres d'une
/// période où elle avait cours, il ne crée aucun compte.
///
/// **Liste close, relevée** le 2026-11-18 sur les publications de l'AFC de
/// 2000 à 2026 (cours mensuels de janvier, mars, juin, septembre et décembre
/// de chaque année ; cours journaliers de chaque fin de trimestre) : ce sont
/// tous les codes publiés absents de [`ISO_4217_CURRENCIES`]. Monnaies
/// remplacées par l'euro, avec l'année de l'euro et la dernière publication
/// relevée : `ATS`, `BEF`, `DEM`, `ESP`, `FIM`, `FRF`, `IEP`, `ITL`, `NLG`,
/// `PTE` (1999) et `GRD` (2001), publiées jusqu'à fin 2001 ; `SIT` (2007,
/// fin 2006) ; `CYP`, `MTL` (2008, janvier 2008) ; `SKK` (2009, janvier
/// 2009) ; `EEK` (2011, fin 2010) ; `LVL` (2014, janvier 2014) ; `LTL`
/// (2015, janvier 2015) ; `HRK` (2023 ; cours journaliers jusqu'en
/// septembre 2025) ; `BGN` (2026 ; cours mensuels jusqu'en mars 2026). Et
/// `VEF`, le bolívar fort vénézuélien (remplacé en 2018, publié jusqu'à fin
/// 2018). L'AFC publie une monnaie retirée encore des mois après son
/// remplacement : la date n'est donc pas contrôlée, l'AFC fait foi. Un code
/// que l'AFC publierait demain et qu'aucune des deux listes ne porte reste
/// écarté (`code_invalid` au journal du serveur) : c'est le signal pour
/// l'ajouter ici.
pub const WITHDRAWN_CURRENCIES: &[&str] = &[
    "ATS", "BEF", "BGN", "CYP", "DEM", "EEK", "ESP", "FIM", "FRF", "GRD", "HRK", "IEP", "ITL",
    "LTL", "LVL", "MTL", "NLG", "PTE", "SIT", "SKK", "VEF",
];

/// `true` si `code`, lu **tel quel** (trois lettres majuscules, sans
/// normalisation : `usd` ne l'est pas), est admis pour un **cours de change** :
/// monnaie active ([`ISO_4217_CURRENCIES`], `CHF` compris) ou retirée
/// ([`WITHDRAWN_CURRENCIES`]).
///
/// ```
/// use kesh_core::currency::is_rate_currency;
/// assert!(is_rate_currency("USD"));
/// assert!(is_rate_currency("BGN"));
/// assert!(!is_rate_currency("bgn"));
/// assert!(!is_rate_currency("XAU"));
/// ```
pub fn is_rate_currency(code: &str) -> bool {
    ISO_4217_CURRENCIES.binary_search(&code).is_ok()
        || WITHDRAWN_CURRENCIES.binary_search(&code).is_ok()
}

/// Une saisie de devise qui n'est pas un code de la liste ISO 4217 embarquée
/// (vide, non ASCII, autre longueur, ou code inconnu). La valeur reçue est
/// gardée telle quelle, pour le message.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("Devise inconnue : {0:?}")]
pub struct UnknownCurrency(pub String);

/// Code ISO 4217 validé : trois lettres majuscules ASCII, de
/// [`ISO_4217_CURRENCIES`] ([`Self::parse`], devise d'un compte) ou, pour un
/// cours de change seulement, de [`WITHDRAWN_CURRENCIES`]
/// ([`Self::parse_rate_currency`]). Aucune instance invalide ne se construit.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(into = "String")]
pub struct CurrencyCode(String);

impl CurrencyCode {
    /// Valide une saisie : `trim`, puis refus de tout caractère non ASCII,
    /// puis **majuscules ASCII** (`to_ascii_uppercase`, jamais Unicode), puis
    /// recherche dans [`ISO_4217_CURRENCIES`] (monnaies actives) : la devise
    /// d'un **compte**. Une monnaie retirée est refusée.
    ///
    /// ```
    /// use kesh_core::currency::CurrencyCode;
    /// assert_eq!(CurrencyCode::parse(" usd ").unwrap().as_str(), "USD");
    /// assert!(CurrencyCode::parse("XAU").is_err());
    /// ```
    pub fn parse(input: &str) -> Result<Self, UnknownCurrency> {
        Self::parse_against(input, |c| ISO_4217_CURRENCIES.binary_search(&c).is_ok())
    }

    /// Valide la devise d'un **cours de change** : même normalisation que
    /// [`Self::parse`], mais une monnaie **retirée** ([`WITHDRAWN_CURRENCIES`])
    /// est admise en plus des monnaies actives ([`is_rate_currency`]). À ne
    /// jamais employer pour la devise d'un compte.
    ///
    /// ```
    /// use kesh_core::currency::CurrencyCode;
    /// assert_eq!(CurrencyCode::parse_rate_currency(" bgn ").unwrap().as_str(), "BGN");
    /// assert!(CurrencyCode::parse("BGN").is_err());
    /// assert!(CurrencyCode::parse_rate_currency("XAU").is_err());
    /// ```
    pub fn parse_rate_currency(input: &str) -> Result<Self, UnknownCurrency> {
        Self::parse_against(input, is_rate_currency)
    }

    /// `trim`, refus du non-ASCII, majuscules ASCII, puis `admis` sur le code
    /// obtenu — la normalisation commune à [`Self::parse`] et à
    /// [`Self::parse_rate_currency`].
    fn parse_against(input: &str, admis: impl Fn(&str) -> bool) -> Result<Self, UnknownCurrency> {
        let trimmed = input.trim();
        if !trimmed.is_ascii() {
            return Err(UnknownCurrency(input.to_string()));
        }
        let upper = trimmed.to_ascii_uppercase();
        if admis(&upper) {
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
    fn monnaies_retirees_triees_et_disjointes_des_actives() {
        for paire in WITHDRAWN_CURRENCIES.windows(2) {
            assert!(paire[0] < paire[1], "ordre ou doublon : {paire:?}");
        }
        for code in WITHDRAWN_CURRENCIES {
            assert_eq!(code.len(), 3, "{code}");
            assert!(code.bytes().all(|b| b.is_ascii_uppercase()), "{code}");
            assert!(
                ISO_4217_CURRENCIES.binary_search(code).is_err(),
                "{code} est dans les deux listes"
            );
        }
    }

    /// Correctif `fix-cours-afc-bgn` : une monnaie retirée s'admet pour un
    /// cours, jamais pour un compte.
    #[test]
    fn monnaie_retiree_admise_pour_un_cours_jamais_pour_un_compte() {
        for code in WITHDRAWN_CURRENCIES {
            assert!(is_rate_currency(code), "{code}");
            assert_eq!(
                CurrencyCode::parse_rate_currency(code).unwrap().as_str(),
                *code
            );
            assert_eq!(
                CurrencyCode::parse(code),
                Err(UnknownCurrency(code.to_string())),
                "{code} ne s'admet pas pour un compte"
            );
            assert!(normalize_foreign_currency(code).is_err(), "{code}");
        }
        // Même normalisation que `parse`.
        assert_eq!(
            CurrencyCode::parse_rate_currency("  bgn\t")
                .unwrap()
                .as_str(),
            "BGN"
        );
        assert!(CurrencyCode::parse_rate_currency("bgı").is_err());
        // Les monnaies actives restent admises, CHF compris (le parseur des
        // cours l'écarte lui-même).
        for code in ["USD", "EUR", "CHF", "RON", "TRY"] {
            assert!(is_rate_currency(code), "{code}");
            assert!(CurrencyCode::parse_rate_currency(code).is_ok(), "{code}");
        }
        // Ni métaux, ni unités, ni codes retirés que l'AFC ne publie pas.
        for code in [
            "XAU", "XDR", "XXX", "CHE", "CUC", "SLL", "ZWL", "LUF", "ZZZ", "",
        ] {
            assert!(!is_rate_currency(code), "{code:?}");
            assert!(CurrencyCode::parse_rate_currency(code).is_err(), "{code:?}");
        }
        // `is_rate_currency` lit le code tel quel.
        assert!(!is_rate_currency("bgn"));
        assert!(!is_rate_currency(" BGN"));
    }

    #[test]
    fn base_currency_est_dans_la_liste() {
        assert!(ISO_4217_CURRENCIES.binary_search(&BASE_CURRENCY).is_ok());
    }
}
