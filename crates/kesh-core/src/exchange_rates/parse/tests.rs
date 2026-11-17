//! Tests de l'analyse des cours (Story 37-2, AC-2) : fixtures synthétiques et
//! réponses hostiles.

use std::time::{Duration, Instant};

use chrono::NaiveDate;
use rust_decimal_macros::dec;

use super::*;

const MENSUEL: &str = include_str!("../../../tests/fixtures/afc_monthly_synthetic.xml");
const JOURNALIER: &str = include_str!("../../../tests/fixtures/afc_daily_synthetic.xml");

const NS_MOIS: &str = "https://www.backend-rates.bazg.admin.ch/avgratesmonth";

fn d(y: i32, m: u32, j: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, j).unwrap()
}

/// Une devise au format de la source.
fn devise(code: &str, waehrung: &str, kurs: &str) -> String {
    format!(
        "<devise code=\"{code}\"><land_de>X</land_de><waehrung>{waehrung}</waehrung>\
         <kurs>{kurs}</kurs></devise>"
    )
}

/// Un document mensuel : `<monat>` puis les devises données.
fn mensuel(monat: &str, devises: &[String]) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><monatsmittelkurs xmlns=\"{NS_MOIS}\">\
         <monat>{monat}</monat>{}</monatsmittelkurs>",
        devises.concat()
    )
}

fn usd() -> String {
    devise("usd", "1 USD", "0.8812")
}

fn codes(r: &MonthlyRates) -> Vec<&str> {
    r.rates.iter().map(|p| p.currency.as_str()).collect()
}

fn ecartes(r: &MonthlyRates) -> Vec<(String, RejectReason)> {
    r.rejected
        .iter()
        .map(|x| (x.code.clone(), x.reason))
        .collect()
}

#[test]
fn fixture_mensuelle_unites_et_cours_par_unite() {
    let r = parse_monthly_rates(MENSUEL.as_bytes()).unwrap();
    assert_eq!(r.month, d(2024, 12, 1));
    assert!(r.rejected.is_empty(), "{:?}", r.rejected);
    assert_eq!(codes(&r), ["USD", "EUR", "JPY", "HUF", "IDR", "ARS"]);
    let par = |c: &str| r.rates.iter().find(|p| p.currency.as_str() == c).unwrap();
    assert_eq!(par("USD").published_unit, 1);
    assert_eq!(par("USD").rate_per_unit, dec!(0.8812));
    assert_eq!(par("JPY").published_unit, 100);
    assert_eq!(par("JPY").published_rate, dec!(0.5733));
    assert_eq!(par("JPY").rate_per_unit, dec!(0.005733));
    assert_eq!(par("HUF").published_unit, 1000);
    assert_eq!(par("HUF").rate_per_unit, dec!(0.0022511));
    assert_eq!(par("IDR").published_unit, 10000);
    assert_eq!(par("IDR").rate_per_unit, dec!(0.00005507));
    assert_eq!(par("ARS").rate_per_unit, dec!(0.0009));
}

#[test]
fn fixture_journaliere_cinq_decimales_et_elements_ignores() {
    let r = parse_daily_rates(JOURNALIER.as_bytes()).unwrap();
    assert_eq!(r.date, d(2024, 6, 28));
    assert!(r.rejected.is_empty(), "{:?}", r.rejected);
    assert_eq!(r.rates.len(), 6);
    let usd = &r.rates[0];
    assert_eq!(usd.currency.as_str(), "USD");
    assert_eq!(usd.published_rate, dec!(0.89731));
    assert_eq!(usd.rate_per_unit, dec!(0.89731));
    let clp = r
        .rates
        .iter()
        .find(|p| p.currency.as_str() == "CLP")
        .unwrap();
    assert_eq!(clp.published_unit, 1000);
    assert_eq!(clp.rate_per_unit, dec!(0.00095013));
    let vnd = r
        .rates
        .iter()
        .find(|p| p.currency.as_str() == "VND")
        .unwrap();
    assert_eq!(vnd.published_unit, 10000);
    assert_eq!(vnd.rate_per_unit, dec!(0.000035217));
}

/// Les deux analyses ne confondent pas leurs racines.
#[test]
fn racine_de_l_autre_document_refusee() {
    assert_eq!(
        parse_daily_rates(MENSUEL.as_bytes()),
        Err(RatesError::Malformed("racine inattendue"))
    );
    assert_eq!(
        parse_monthly_rates(JOURNALIER.as_bytes()),
        Err(RatesError::Malformed("racine inattendue"))
    );
}

#[test]
fn racine_vide_pas_encore_publie() {
    for xml in [
        format!("<monatsmittelkurs xmlns=\"{NS_MOIS}\"/>"),
        format!("<monatsmittelkurs xmlns=\"{NS_MOIS}\">\n  </monatsmittelkurs>"),
        format!("<?xml version=\"1.0\"?><monatsmittelkurs xmlns=\"{NS_MOIS}\"></monatsmittelkurs>"),
    ] {
        assert_eq!(
            parse_monthly_rates(xml.as_bytes()),
            Err(RatesError::NotPublished),
            "{xml}"
        );
    }
    assert_eq!(
        parse_daily_rates(
            b"<wechselkurse xmlns=\"https://www.backend-rates.bazg.admin.ch/xmldaily\"/>"
        ),
        Err(RatesError::NotPublished)
    );
    assert_eq!(RatesError::NotPublished.code(), "not_published");
}

#[test]
fn mois_absent_illisible_ou_hors_bornes_refuse() {
    let sans_mois = format!(
        "<monatsmittelkurs xmlns=\"{NS_MOIS}\">{}</monatsmittelkurs>",
        usd()
    );
    assert_eq!(
        parse_monthly_rates(sans_mois.as_bytes()),
        Err(RatesError::Malformed("date absente"))
    );
    for monat in [
        "2024-13",
        "2024-00",
        "1999-12",
        "2100-01",
        "24-12",
        "2024/12",
        "２０２４-1",
        "",
    ] {
        let xml = mensuel(monat, &[usd()]);
        assert_eq!(
            parse_monthly_rates(xml.as_bytes()),
            Err(RatesError::Malformed("mois illisible")),
            "{monat:?}"
        );
    }
    // Deux <monat> : refus.
    let deux = mensuel("2024-12", &[String::from("<monat>2024-11</monat>"), usd()]);
    assert_eq!(
        parse_monthly_rates(deux.as_bytes()),
        Err(RatesError::Malformed("date répétée"))
    );
    // Blancs de bord admis.
    let r = parse_monthly_rates(mensuel(" 2000-01\n", &[usd()]).as_bytes()).unwrap();
    assert_eq!(r.month, d(2000, 1, 1));
}

#[test]
fn mois_autre_que_demande_refuse_par_l_appelant() {
    let r = parse_monthly_rates(mensuel("2024-11", &[usd()]).as_bytes()).unwrap();
    assert_eq!(check_monthly_month(&r, d(2024, 11, 1)), Ok(()));
    assert_eq!(check_monthly_month(&r, d(2024, 11, 20)), Ok(()));
    let err = check_monthly_month(&r, d(2024, 12, 1)).unwrap_err();
    assert_eq!(
        err,
        RatesError::MonthMismatch {
            expected: d(2024, 12, 1),
            got: d(2024, 11, 1)
        }
    );
    assert_eq!(err.code(), "month_mismatch");
}

fn journalier(datum: &str) -> String {
    format!(
        "<wechselkurse xmlns=\"https://www.backend-rates.bazg.admin.ch/xmldaily\">\
         <datum>{datum}</datum><zeit>07:00:05</zeit><gueltigkeit>01.01.2025</gueltigkeit>{}\
         </wechselkurse>",
        devise("eur", "1 EUR", "0.94947")
    )
}

#[test]
fn datum_lu_valide_et_bornes() {
    assert_eq!(
        parse_daily_rates(journalier("31.12.1999").as_bytes())
            .unwrap()
            .date,
        d(1999, 12, 31)
    );
    assert_eq!(
        parse_daily_rates(journalier("29.02.2024").as_bytes())
            .unwrap()
            .date,
        d(2024, 2, 29)
    );
    for datum in [
        "31.02.2024",
        "29.02.2023",
        "31.12.1998",
        "01.01.2100",
        "2024-12-31",
        "1.1.2024",
        "３1.12.2024",
    ] {
        assert_eq!(
            parse_daily_rates(journalier(datum).as_bytes()),
            Err(RatesError::Malformed("date illisible")),
            "{datum:?}"
        );
    }
}

#[test]
fn datum_controle_par_l_appelant() {
    let r = parse_daily_rates(journalier("28.06.2024").as_bytes()).unwrap();
    // Un dimanche rend le vendredi : accepté.
    assert_eq!(check_daily_date(&r, d(2024, 6, 30)), Ok(()));
    assert_eq!(check_daily_date(&r, d(2024, 6, 28)), Ok(()));
    assert_eq!(check_daily_date(&r, d(2024, 7, 5)), Ok(()));
    // Plus de 7 jours, ou postérieur : refusé.
    let e = check_daily_date(&r, d(2024, 7, 6)).unwrap_err();
    assert_eq!(e.code(), "date_mismatch");
    assert!(check_daily_date(&r, d(2024, 6, 27)).is_err());
}

#[test]
fn devise_ecartee_sans_bloquer_les_autres() {
    let xml = mensuel(
        "2024-12",
        &[
            usd(),
            devise("eur", "1 EUR", "0.0000"),
            devise("gbp", "1 USD", "1.1"),
            devise("jpy", "100", "0.5"),
            devise("nok", "0 NOK", "8.1"),
        ],
    );
    let r = parse_monthly_rates(xml.as_bytes()).unwrap();
    assert_eq!(codes(&r), ["USD"]);
    assert_eq!(
        ecartes(&r),
        [
            ("eur".into(), RejectReason::RateNotPositive),
            ("gbp".into(), RejectReason::CodeMismatch),
            ("jpy".into(), RejectReason::UnitInvalid),
            ("nok".into(), RejectReason::UnitInvalid),
        ]
    );
}

#[test]
fn doublon_ecarte_dans_ses_deux_occurrences() {
    let xml = mensuel(
        "2024-12",
        &[
            usd(),
            devise("eur", "1 EUR", "0.93"),
            devise("EUR", "1 EUR", "0.94"),
        ],
    );
    let r = parse_monthly_rates(xml.as_bytes()).unwrap();
    assert_eq!(codes(&r), ["USD"]);
    assert_eq!(
        ecartes(&r),
        [
            ("eur".into(), RejectReason::Duplicate),
            ("EUR".into(), RejectReason::Duplicate)
        ]
    );
    // Un enfant lu répété dans une devise : écartée.
    let double = "<devise code=\"sek\"><waehrung>1 SEK</waehrung><kurs>0.08</kurs>\
                  <kurs>0.09</kurs></devise>"
        .to_string();
    let r = parse_monthly_rates(mensuel("2024-12", &[usd(), double]).as_bytes()).unwrap();
    assert_eq!(ecartes(&r), [("sek".into(), RejectReason::Duplicate)]);
}

#[test]
fn aucune_devise_valide_refuse_le_mois() {
    let xml = mensuel("2024-12", &[devise("eur", "1 EUR", "0")]);
    let err = parse_monthly_rates(xml.as_bytes()).unwrap_err();
    assert_eq!(err.code(), "no_valid_rate");
    assert_eq!(
        err,
        RatesError::NoValidRate {
            rejected: vec![RejectedRate {
                code: "eur".into(),
                reason: RejectReason::RateNotPositive
            }]
        }
    );
    // Racine avec <monat> mais sans devise : aucune devise valide.
    let xml = mensuel("2024-12", &[]);
    assert_eq!(
        parse_monthly_rates(xml.as_bytes()).unwrap_err().code(),
        "no_valid_rate"
    );
}

/// Nombres énormes ou invalides : devise écartée, les autres écrites.
#[test]
fn nombres_hostiles_ecartes() {
    let enorme = "9".repeat(10_000);
    let mauvais = [
        enorme.as_str(),
        "1e400",
        "-0.5",
        "+0.5",
        "0x10",
        "1,5",
        "NaN",
        "０.５",
        ".5",
        "5.",
        "0.1234567",
        "12345678901234",
        "1 000",
    ];
    for kurs in mauvais {
        let xml = mensuel("2024-12", &[usd(), devise("eur", "1 EUR", kurs)]);
        let r = parse_monthly_rates(xml.as_bytes()).unwrap();
        assert_eq!(codes(&r), ["USD"], "{kurs:.20}");
        let raison = r.rejected[0].reason;
        assert!(
            matches!(raison, RejectReason::RateInvalid | RejectReason::TooLong),
            "{kurs:.20} : {raison:?}"
        );
    }
    // Grammaire tenue mais bornes de prudence : cours ≥ 10⁹, unité > 10⁶.
    let xml = mensuel(
        "2024-12",
        &[
            usd(),
            devise("eur", "1 EUR", "1000000000"),
            devise("gbp", "1000001 GBP", "1.5"),
            devise("sek", "1 SEK", "1000000.5"),
            devise("nok", "1000000 NOK", "0.000001"),
        ],
    );
    let r = parse_monthly_rates(xml.as_bytes()).unwrap();
    assert_eq!(codes(&r), ["USD"]);
    assert_eq!(
        ecartes(&r),
        [
            ("eur".into(), RejectReason::RateTooLarge),
            ("gbp".into(), RejectReason::UnitTooLarge),
            (
                "sek".into(),
                RejectReason::RatePerUnit(RateBoundsError::TooLarge)
            ),
            (
                "nok".into(),
                RejectReason::RatePerUnit(RateBoundsError::NotPositive)
            ),
        ]
    );
    // Juste sous les bornes : admis, arrondi à 10 décimales.
    let xml = mensuel(
        "2024-12",
        &[
            devise("eur", "1 EUR", "999999.999999"),
            devise("nok", "10000 NOK", "0.000001"),
        ],
    );
    let r = parse_monthly_rates(xml.as_bytes()).unwrap();
    assert_eq!(r.rates[0].rate_per_unit, dec!(999999.999999));
    assert_eq!(r.rates[1].rate_per_unit, dec!(0.0000000001));
}

/// Arrondi à 10 décimales, équidistant loin de zéro (et non au pair).
#[test]
fn arrondi_equidistant_loin_de_zero() {
    // 0.000005 / 1000 = 0.000000005 : neuf décimales, exact.
    let xml = mensuel("2024-12", &[devise("huf", "1000 HUF", "0.000005")]);
    let r = parse_monthly_rates(xml.as_bytes()).unwrap();
    assert_eq!(r.rates[0].rate_per_unit, dec!(0.0000000050));
    // 0.000015 / 10000 = 0.0000000015 : exact.
    let xml = mensuel("2024-12", &[devise("idr", "10000 IDR", "0.000015")]);
    assert_eq!(
        parse_monthly_rates(xml.as_bytes()).unwrap().rates[0].rate_per_unit,
        dec!(0.0000000015)
    );
    // 0.000025 / 100000 = 0.00000000025 : équidistant → 0.0000000003 (au pair :
    // 0.0000000002).
    let xml = mensuel("2024-12", &[devise("idr", "100000 IDR", "0.000025")]);
    assert_eq!(
        parse_monthly_rates(xml.as_bytes()).unwrap().rates[0].rate_per_unit,
        dec!(0.0000000003)
    );
}

#[test]
fn codes_non_iso_ecartes() {
    for code in [
        "XXX", "ZZZ", "us", "USDX", "ÜSD", "CHF", "chf", "XAU", "", "1US",
    ] {
        let w = format!(
            "1 {}",
            if code.len() == 3 && code.is_ascii() {
                code
            } else {
                "USD"
            }
        );
        let xml = mensuel("2024-12", &[usd(), devise(code, &w, "1.0")]);
        let r = parse_monthly_rates(xml.as_bytes()).unwrap();
        assert_eq!(codes(&r), ["USD"], "{code:?}");
        assert_eq!(r.rejected.len(), 1, "{code:?}");
        assert_eq!(r.rejected[0].reason, RejectReason::CodeInvalid, "{code:?}");
    }
    // Attribut absent, ou trop long.
    let sans = "<devise><waehrung>1 EUR</waehrung><kurs>0.9</kurs></devise>".to_string();
    let long = devise(&"e".repeat(65), "1 EUR", "0.9");
    let r = parse_monthly_rates(mensuel("2024-12", &[usd(), sans, long]).as_bytes()).unwrap();
    assert_eq!(
        ecartes(&r),
        [
            (String::new(), RejectReason::Missing),
            (String::new(), RejectReason::TooLong)
        ]
    );
    // Casse et blancs de bord admis.
    let r = parse_monthly_rates(mensuel("2024-12", &[devise(" Eur ", "1 eur", "0.9")]).as_bytes())
        .unwrap();
    assert_eq!(codes(&r), ["EUR"]);
}

#[test]
fn enfant_absent_ou_trop_long_ecarte() {
    let sans_kurs = "<devise code=\"eur\"><waehrung>1 EUR</waehrung></devise>".to_string();
    let vide = "<devise code=\"gbp\"/>".to_string();
    let long = devise("sek", "1 SEK", &format!("0.{}", "1".repeat(70)));
    let r = parse_monthly_rates(mensuel("2024-12", &[usd(), sans_kurs, vide, long]).as_bytes())
        .unwrap();
    assert_eq!(
        ecartes(&r),
        [
            ("eur".into(), RejectReason::Missing),
            ("gbp".into(), RejectReason::Missing),
            ("sek".into(), RejectReason::TooLong)
        ]
    );
}

/// « Billion laughs » : DOCTYPE à entités imbriquées → refus, sans expansion
/// et en temps borné.
#[test]
fn billion_laughs_refuse() {
    let xml = format!(
        "<?xml version=\"1.0\"?><!DOCTYPE monatsmittelkurs [\
         <!ENTITY lol \"lol\">\
         <!ENTITY lol2 \"&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;\">\
         <!ENTITY lol3 \"&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;\">\
         <!ENTITY lol9 \"&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;\">\
         ]><monatsmittelkurs xmlns=\"{NS_MOIS}\"><monat>2024-12</monat>\
         <devise code=\"usd\"><waehrung>1 USD</waehrung><kurs>&lol9;</kurs></devise>\
         </monatsmittelkurs>"
    );
    let t = Instant::now();
    assert_eq!(
        parse_monthly_rates(xml.as_bytes()),
        Err(RatesError::Malformed("DOCTYPE refusé"))
    );
    assert!(t.elapsed() < Duration::from_secs(2));
}

#[test]
fn entite_externe_refusee() {
    let xml = format!(
        "<?xml version=\"1.0\"?><!DOCTYPE m [<!ENTITY xxe SYSTEM \"file:///etc/passwd\">]>\
         <monatsmittelkurs xmlns=\"{NS_MOIS}\"><monat>2024-12</monat>\
         <devise code=\"usd\"><waehrung>1 USD</waehrung><kurs>&xxe;</kurs></devise>\
         </monatsmittelkurs>"
    );
    let err = parse_monthly_rates(xml.as_bytes()).unwrap_err();
    assert_eq!(err, RatesError::Malformed("DOCTYPE refusé"));
    assert_eq!(err.code(), "malformed");
}

/// Entité inconnue sans DOCTYPE : refus du document, où qu'elle soit.
#[test]
fn entite_inconnue_refusee() {
    for xml in [
        mensuel("2024-12", &[devise("usd", "1 USD", "&x;")]),
        mensuel(
            "2024-12",
            &[usd(), devise("eur", "1 EUR", "0.9").replace("X", "&nbsp;")],
        ),
        mensuel("2024-12", &[devise("us&x;", "1 USD", "0.9")]),
    ] {
        assert!(
            matches!(
                parse_monthly_rates(xml.as_bytes()),
                Err(RatesError::Malformed(_))
            ),
            "{xml}"
        );
    }
}

/// Références de caractère et entités prédéfinies : admises et résolues.
#[test]
fn references_admises() {
    let xml = mensuel(
        "2024-12",
        &[
            devise("usd", "1&#32;USD", "0&#x2E;88"),
            devise("eur", "1 EUR", "0.93").replace("X", "A &amp; B &lt;&gt;&quot;&apos;"),
        ],
    );
    let r = parse_monthly_rates(xml.as_bytes()).unwrap();
    assert_eq!(codes(&r), ["USD", "EUR"]);
    assert_eq!(r.rates[0].published_rate, dec!(0.88));
}

#[test]
fn cdata_et_pi_dans_un_element_lu_refuses() {
    for kurs in ["<![CDATA[0.9]]>", "0.9<?pi x?>"] {
        let xml = mensuel("2024-12", &[usd(), devise("eur", "1 EUR", kurs)]);
        assert!(
            matches!(
                parse_monthly_rates(xml.as_bytes()),
                Err(RatesError::Malformed(_))
            ),
            "{kurs}"
        );
    }
    let xml = mensuel("<![CDATA[2024-12]]>", &[usd()]);
    assert!(matches!(
        parse_monthly_rates(xml.as_bytes()),
        Err(RatesError::Malformed(_))
    ));
    // Hors d'un élément lu (dans land_de) : ignorés.
    let xml = mensuel(
        "2024-12",
        &[usd().replace(">X<", "><![CDATA[Etats]]><?pi y?><")],
    );
    assert_eq!(
        codes(&parse_monthly_rates(xml.as_bytes()).unwrap()),
        ["USD"]
    );
}

/// 10 000 niveaux d'imbrication : refus, sans débordement de pile.
#[test]
fn imbrication_profonde_refusee() {
    let profond = format!(
        "<monatsmittelkurs xmlns=\"{NS_MOIS}\"><monat>2024-12</monat>{}{}</monatsmittelkurs>",
        "<a>".repeat(10_000),
        "</a>".repeat(10_000)
    );
    assert_eq!(
        parse_monthly_rates(profond.as_bytes()),
        Err(RatesError::Malformed("imbrication trop profonde"))
    );
    // Un quatrième niveau sous une devise suffit.
    let xml = mensuel(
        "2024-12",
        &["<devise code=\"usd\"><kurs><b>0.9</b></kurs></devise>".to_string()],
    );
    assert!(matches!(
        parse_monthly_rates(xml.as_bytes()),
        Err(RatesError::Malformed(_))
    ));
    // Élément vide au quatrième niveau : refusé aussi.
    let xml = mensuel(
        "2024-12",
        &["<devise code=\"usd\"><land_de><b/></land_de></devise>".to_string()],
    );
    assert_eq!(
        parse_monthly_rates(xml.as_bytes()),
        Err(RatesError::Malformed("imbrication trop profonde"))
    );
}

#[test]
fn cinq_cent_une_devises_refusees() {
    let codes_iso: Vec<&str> = crate::currency::ISO_4217_CURRENCIES
        .iter()
        .copied()
        .filter(|c| *c != "CHF")
        .collect();
    let cinq_cents: Vec<String> = (0..MAX_DEVISES)
        .map(|i| {
            let c = codes_iso[i % codes_iso.len()];
            devise(c, &format!("1 {c}"), "1.0")
        })
        .collect();
    // 500 : analysées (les doublons s'écartent, aucune valide ici sauf les
    // codes vus une fois — peu importe : le document n'est pas refusé pour sa
    // taille).
    let r = parse_monthly_rates(mensuel("2024-12", &cinq_cents).as_bytes());
    assert_ne!(r, Err(RatesError::Malformed("trop de devises")));
    let mut cinq_cent_une = cinq_cents.clone();
    cinq_cent_une.push(usd());
    assert_eq!(
        parse_monthly_rates(mensuel("2024-12", &cinq_cent_une).as_bytes()),
        Err(RatesError::Malformed("trop de devises"))
    );
}

#[test]
fn corps_non_utf8_ou_trop_gros_refuse() {
    let mut xml = mensuel("2024-12", &[usd()]).into_bytes();
    xml.insert(xml.len() - 20, 0xFF);
    assert_eq!(
        parse_monthly_rates(&xml),
        Err(RatesError::Malformed("UTF-8 invalide"))
    );
    let latin1 = b"<monatsmittelkurs><monat>2024-12</monat><devise code=\"usd\"><land_de>\xC9tats</land_de></devise></monatsmittelkurs>";
    assert_eq!(
        parse_monthly_rates(latin1),
        Err(RatesError::Malformed("UTF-8 invalide"))
    );
    let gros = vec![b' '; MAX_BODY_BYTES + 1];
    assert_eq!(
        parse_monthly_rates(&gros),
        Err(RatesError::Malformed("corps trop volumineux"))
    );
}

#[test]
fn xml_mal_forme_ou_tronque_refuse() {
    let complet = mensuel("2024-12", &[usd()]);
    let tronque = &complet[..complet.len() - 30];
    assert!(matches!(
        parse_monthly_rates(tronque.as_bytes()),
        Err(RatesError::Malformed(_))
    ));
    let croise =
        format!("<monatsmittelkurs xmlns=\"{NS_MOIS}\"><monat>2024-12</kurs></monatsmittelkurs>");
    assert!(matches!(
        parse_monthly_rates(croise.as_bytes()),
        Err(RatesError::Malformed(_))
    ));
    assert!(matches!(
        parse_monthly_rates(b""),
        Err(RatesError::Malformed("racine absente"))
    ));
    assert!(matches!(
        parse_monthly_rates(b"pas du xml"),
        Err(RatesError::Malformed(_))
    ));
    // Deux racines.
    let deux = format!("{complet}{complet}");
    assert!(matches!(
        parse_monthly_rates(deux.as_bytes()),
        Err(RatesError::Malformed(_))
    ));
    // Un élément dans le texte d'un <monat>.
    let xml = format!(
        "<monatsmittelkurs xmlns=\"{NS_MOIS}\"><monat>2024<x/>-12</monat>{}</monatsmittelkurs>",
        usd()
    );
    assert_eq!(
        parse_monthly_rates(xml.as_bytes()),
        Err(RatesError::Malformed("élément dans un texte lu"))
    );
}

/// Préfixe d'espace de noms : la lecture par nom local l'ignore.
#[test]
fn lecture_par_nom_local() {
    let xml = "<m:monatsmittelkurs xmlns:m=\"urn:x\"><m:monat>2024-12</m:monat>\
               <m:devise code=\"usd\"><m:waehrung>1 USD</m:waehrung><m:kurs>0.88</m:kurs>\
               </m:devise></m:monatsmittelkurs>";
    assert_eq!(
        codes(&parse_monthly_rates(xml.as_bytes()).unwrap()),
        ["USD"]
    );
}

#[test]
fn codes_des_motifs_d_ecart() {
    assert_eq!(RejectReason::Duplicate.as_str(), "duplicate");
    assert_eq!(
        RejectReason::RatePerUnit(RateBoundsError::TooLarge).as_str(),
        "rate_per_unit_out_of_bounds"
    );
    assert_eq!(RatesError::Malformed("x").code(), "malformed");
}
