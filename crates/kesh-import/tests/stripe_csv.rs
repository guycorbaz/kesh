//! Lecteur du CSV Stripe sur la fixture fictive (Story 40-7a, AC-1 à AC-7).
//!
//! `tests/fixtures/stripe-itemized-payout-fictif.csv` est inventée de toutes
//! pièces (catalogue : `tests/fixtures/README.md`) : treize colonnes, neuf
//! lignes de données en 2020, quatre versements, trois devises.

use chrono::NaiveDate;
use kesh_import::{
    MAX_CSV_LINE_ERRORS, StripeCsvError, StripeLineErrorCode, parse_stripe_payout_csv,
};
use rust_decimal_macros::dec;

const FIXTURE: &str = include_str!("fixtures/stripe-itemized-payout-fictif.csv");
const SANS_SOURCE: &[u8] = include_bytes!("fixtures/stripe-itemized-payout-sans-source-fictif.csv");
const HEADER: &str = "automatic_payout_id,automatic_payout_effective_at,balance_transaction_id,created,available_on,currency,gross,fee,net,reporting_category,description,source_id,charge_id";

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// Une ligne valide, dont on remplace une cellule (indice dans [`HEADER`]).
fn line_with(idx: usize, value: &str) -> String {
    let mut cells: Vec<&str> = "po_T1,2020-02-03 00:00:00,txn_T1,2020-01-29 10:15:04,2020-02-03,chf,100.00,3.20,96.80,charge,d,ch_T1,ch_T1"
        .split(',')
        .collect();
    cells[idx] = value;
    cells.join(",")
}

fn parse(text: &str) -> Result<kesh_import::StripePayoutCsv, StripeCsvError> {
    parse_stripe_payout_csv(text.as_bytes())
}

fn lines_err(text: &str) -> Vec<kesh_import::StripeLineError> {
    match parse(text) {
        Err(StripeCsvError::Lines(l)) => l,
        other => panic!("attendu Lines, obtenu {other:?}"),
    }
}

#[test]
fn reads_every_line_of_the_fixture() {
    let out = parse(FIXTURE).unwrap();
    assert_eq!(out.rows.len(), 9);
    // (ligne, versement, catégorie, devise, brut, frais, net, objet lié)
    let expected = [
        (
            2,
            "po_TEST0001",
            "charge",
            "CHF",
            dec!(100.00),
            dec!(3.20),
            dec!(96.80),
            Some("ch_TEST0001"),
        ),
        (
            3,
            "po_TEST0001",
            "charge",
            "CHF",
            dec!(50.00),
            dec!(1.75),
            dec!(48.25),
            Some("ch_TEST0002"),
        ),
        (
            4,
            "po_TEST0001",
            "refund",
            "CHF",
            dec!(-20.00),
            dec!(-0.60),
            dec!(-19.40),
            Some("re_TEST0001"),
        ),
        (
            5,
            "po_TEST0001",
            "fee",
            "CHF",
            dec!(-0.50),
            dec!(0.00),
            dec!(-0.50),
            None,
        ),
        (
            6,
            "po_TEST0002",
            "charge",
            "EUR",
            dec!(200.00),
            dec!(6.00),
            dec!(194.00),
            Some("ch_TEST0003"),
        ),
        (
            7,
            "po_TEST0002",
            "charge",
            "EUR",
            dec!(80.00),
            dec!(2.50),
            dec!(77.50),
            Some("ch_TEST0099"),
        ),
        (
            8,
            "po_TEST0002",
            "fee",
            "EUR",
            dec!(-1.00),
            dec!(0.20),
            dec!(-1.20),
            Some("fee_TEST0001"),
        ),
        (
            9,
            "po_TEST0003",
            "charge",
            "USD",
            dec!(40.00),
            dec!(1.46),
            dec!(38.54),
            Some("ch_TEST0004"),
        ),
        (
            10,
            "po_TEST0004",
            "payout_reversal",
            "USD",
            dec!(-38.54),
            dec!(0.00),
            dec!(-38.54),
            Some("po_TEST0003"),
        ),
    ];
    for (row, e) in out.rows.iter().zip(expected) {
        assert_eq!(
            (
                row.line,
                row.payout_id.as_str(),
                row.category.as_str(),
                row.currency.as_str(),
                row.gross,
                row.fee,
                row.net,
                row.source_id.as_deref()
            ),
            e
        );
    }
    let first = &out.rows[0];
    assert_eq!(first.payout_date, date(2020, 2, 3));
    assert_eq!(first.date, date(2020, 1, 29));
    assert_eq!(first.available_on, Some(date(2020, 2, 3)));
    assert_eq!(first.balance_transaction_id, "txn_TEST0001");
    assert_eq!(first.charge_id.as_deref(), Some("ch_TEST0001"));
    assert_eq!(first.description.as_deref(), Some("Vente fictive A"));
    // Le net EUR du versement po_TEST0002 : valeur tenue par le catalogue.
    let eur: rust_decimal::Decimal = out
        .rows
        .iter()
        .filter(|r| r.payout_id == "po_TEST0002")
        .map(|r| r.net)
        .sum();
    assert_eq!(eur, dec!(270.30));
}

#[test]
fn fixture_is_synthetic_and_without_bom() {
    assert!(!FIXTURE.starts_with('\u{feff}'));
    assert_eq!(FIXTURE.lines().next().unwrap(), HEADER);
    assert!(
        FIXTURE
            .lines()
            .skip(1)
            .all(|l| l.contains("_TEST") && l.split(',').nth(3).unwrap().starts_with("2020-"))
    );
}

#[test]
fn columns_in_another_order_and_created_utc() {
    // Colonnes inversées, `created_utc` et `available_on_utc` au lieu de `created` et `available_on`.
    let header: Vec<&str> = HEADER.split(',').collect();
    let first: Vec<&str> = FIXTURE.lines().nth(1).unwrap().split(',').collect();
    let rename = |h: &str| match h {
        "created" => "created_utc".to_string(),
        "available_on" => "available_on_utc".to_string(),
        o => o.to_string(),
    };
    let h: Vec<String> = header.iter().rev().map(|h| rename(h)).collect();
    let r: Vec<&str> = first.iter().rev().copied().collect();
    let out = parse(&format!("{}\n{}\n", h.join(","), r.join(","))).unwrap();
    assert_eq!(out.rows.len(), 1);
    assert_eq!(out.rows[0].date, date(2020, 1, 29));
    assert_eq!(out.rows[0].available_on, Some(date(2020, 2, 3)));
    assert_eq!(out.rows[0].net, dec!(96.80));
}

#[test]
fn report_without_linked_object_column_is_refused() {
    assert_eq!(
        parse_stripe_payout_csv(SANS_SOURCE),
        Err(StripeCsvError::MissingColumns(vec![
            "source_id|charge_id".to_string()
        ]))
    );
}

#[test]
fn either_linked_column_is_enough() {
    let h = HEADER.replace(",source_id", "");
    let l = "po_T1,2020-02-03,txn_T1,2020-01-29,2020-02-03,chf,1.00,0.00,1.00,charge,d,ch_T1";
    let out = parse(&format!("{h}\n{l}\n")).unwrap();
    assert_eq!(out.rows[0].source_id, None);
    assert_eq!(out.rows[0].charge_id.as_deref(), Some("ch_T1"));
}

#[test]
fn all_missing_required_columns_are_named_in_order() {
    let err = parse("net,gross,fee,x\n1,1,0,y\n").unwrap_err();
    assert_eq!(
        err,
        StripeCsvError::MissingColumns(
            [
                "automatic_payout_id",
                "automatic_payout_effective_at",
                "balance_transaction_id",
                "created|created_utc",
                "currency",
                "reporting_category",
                "source_id|charge_id"
            ]
            .map(String::from)
            .to_vec()
        )
    );
    assert_eq!(err.code(), "STRIPE_CSV_MISSING_COLUMNS");
}

#[test]
fn duplicate_columns_are_refused() {
    let err = parse(&format!("{HEADER},net\n")).unwrap_err();
    assert_eq!(err, StripeCsvError::DuplicateColumn("net".to_string()));
    let err = parse(&format!("{HEADER},created_utc\n")).unwrap_err();
    assert_eq!(err, StripeCsvError::DuplicateColumn("created".to_string()));
}

#[test]
fn net_mismatch_is_a_line_error() {
    let l = lines_err(&format!("{HEADER}\n{}\n", line_with(8, "96.81")));
    assert_eq!(
        (l[0].line, l[0].field, l[0].code),
        (2, Some("net"), StripeLineErrorCode::NetMismatch)
    );
}

#[test]
fn payout_with_two_currencies_is_refused_on_the_second_line() {
    let text = format!(
        "{HEADER}\n{}\n{}\n",
        line_with(5, "chf"),
        line_with(5, "eur")
    );
    let l = lines_err(&text);
    assert_eq!(l.len(), 1);
    assert_eq!(
        (l[0].line, l[0].field, l[0].code),
        (
            3,
            Some("currency"),
            StripeLineErrorCode::PayoutCurrencyMixed
        )
    );
}

#[test]
fn decimal_comma_is_refused_never_read_as_hundred_times() {
    // Frais nuls : brut et net restent égaux, seul le contrôle de forme voit l'erreur.
    let row = "po_T1,2020-02-03,txn_T1,2020-01-29,2020-02-03,chf,\"12,50\",0.00,\"12,50\",charge,d,ch_T1,ch_T1";
    let l = lines_err(&format!("{HEADER}\n{row}\n"));
    assert!(l.iter().all(|e| e.code == StripeLineErrorCode::Amount));
    assert_eq!(
        l.iter().filter_map(|e| e.field).collect::<Vec<_>>(),
        ["gross", "net"]
    );
    // Milliers à la virgule : refusés aussi (Stripe n'en écrit pas).
    let row = "po_T1,2020-02-03,txn_T1,2020-01-29,2020-02-03,chf,\"1,234.50\",0.00,\"1,234.50\",charge,d,ch_T1,ch_T1";
    assert_eq!(lines_err(&format!("{HEADER}\n{row}\n")).len(), 2);
}

#[test]
fn three_decimals_and_other_amount_forms_are_refused() {
    for bad in ["1.234", "+1.00", "1 000.00", "1'000.00", "", "abc"] {
        let l = lines_err(&format!("{HEADER}\n{}\n", line_with(6, bad)));
        assert!(
            l.iter()
                .any(|e| e.field == Some("gross") && e.code == StripeLineErrorCode::Amount),
            "{bad:?}"
        );
    }
}

#[test]
fn bom_is_removed_and_latin1_decoded() {
    let mut bytes = b"\xEF\xBB\xBF".to_vec();
    bytes.extend_from_slice(FIXTURE.as_bytes());
    assert_eq!(parse_stripe_payout_csv(&bytes).unwrap().rows.len(), 9);

    let text = format!("{HEADER}\n{}\n", line_with(10, "d\u{e9}j\u{e0}"));
    let latin1: Vec<u8> = text.chars().map(|c| c as u8).collect();
    let out = parse_stripe_payout_csv(&latin1).unwrap();
    assert_eq!(out.rows[0].description.as_deref(), Some("d\u{e9}j\u{e0}"));
}

#[test]
fn wrong_cell_count_is_a_field_count_error() {
    let text = format!("{HEADER}\npo_T1,2020-02-03,txn_T1\n");
    let l = lines_err(&text);
    assert_eq!(
        (l[0].line, l[0].field, l[0].code),
        (2, None, StripeLineErrorCode::FieldCount)
    );
}

#[test]
fn ids_currency_and_dates_are_checked() {
    let code = |idx, v: &str| lines_err(&format!("{HEADER}\n{}\n", line_with(idx, v)))[0].code;
    assert_eq!(code(0, ""), StripeLineErrorCode::Id);
    assert_eq!(code(2, ""), StripeLineErrorCode::Id);
    assert_eq!(code(2, "txn:1"), StripeLineErrorCode::Id);
    assert_eq!(code(5, "eu"), StripeLineErrorCode::Currency);
    assert_eq!(code(3, "29/01/2020"), StripeLineErrorCode::Date);
    assert_eq!(code(3, "0999-01-01"), StripeLineErrorCode::Date);
    assert_eq!(code(1, "2020-13-01"), StripeLineErrorCode::Date);
    assert_eq!(code(4, "n'importe quoi"), StripeLineErrorCode::Date);
}

#[test]
fn date_only_form_is_accepted_and_category_lowercased() {
    let out = parse(&format!("{HEADER}\n{}\n", line_with(9, "CHARGE"))).unwrap();
    assert_eq!(out.rows[0].category, "charge");
    assert_eq!(out.rows[0].available_on, Some(date(2020, 2, 3)));
}

#[test]
fn empty_inputs() {
    for text in [
        "",
        "\n\n",
        &format!("{HEADER}\n"),
        &format!("{HEADER}\n\n,,,,,,,,,,,,\n"),
    ] {
        assert_eq!(parse(text), Err(StripeCsvError::Empty), "{text:?}");
    }
    assert_eq!(StripeCsvError::Empty.code(), "STRIPE_CSV_EMPTY");
}

#[test]
fn error_collection_is_capped() {
    let mut text = format!("{HEADER}\n");
    for _ in 0..(MAX_CSV_LINE_ERRORS + 20) {
        text.push_str(&line_with(8, "0.00"));
        text.push('\n');
    }
    assert_eq!(lines_err(&text).len(), MAX_CSV_LINE_ERRORS);
}

#[test]
fn extreme_amounts_never_panic_on_the_net_check() {
    // `Decimal::MAX - (-1)` déborderait : le lecteur de montants partagé refuse
    // désormais plus de 15 chiffres (38-12a), donc la ligne est refusée dès le
    // montant brut — erreur de ligne, jamais une panique.
    let max = "79228162514264337593543950335";
    let row = line_with(6, max)
        .replace("3.20", "-1")
        .replace("96.80", "0");
    let l = lines_err(&format!("{HEADER}\n{row}\n"));
    assert_eq!(
        (l[0].line, l[0].field, l[0].code),
        (2, Some("gross"), StripeLineErrorCode::Amount)
    );
}

#[test]
fn several_line_errors_come_out_sorted_by_line() {
    // Ligne 2 : erreur de contenu ; ligne 3 : nombre de cellules. Le second
    // type est collecté avant le premier, le tri rétablit l'ordre du fichier.
    let text = format!(
        "{HEADER}\n{}\npo_T1,2020-02-03,txn_T1\n{}\n",
        line_with(8, "96.81"),
        line_with(8, "96.82")
    );
    let l = lines_err(&text);
    let got: Vec<_> = l.iter().map(|e| (e.line, e.code)).collect();
    assert_eq!(
        got,
        vec![
            (2, StripeLineErrorCode::NetMismatch),
            (3, StripeLineErrorCode::FieldCount),
            (4, StripeLineErrorCode::NetMismatch),
        ]
    );
}

#[test]
fn line_after_a_blank_line_takes_the_blank_line_number() {
    // Limite du crate `csv`, partagée avec PayPal et Wise : un enregistrement
    // qui suit une ligne vide au milieu du fichier porte le numéro de cette
    // ligne vide (ici 3, alors que sa ligne dans le fichier est la 4).
    let text = format!(
        "{HEADER}\n{}\n\n{}\n",
        line_with(8, "96.80"),
        line_with(8, "96.81")
    );
    let l = lines_err(&text);
    assert_eq!(
        (l[0].line, l[0].code),
        (3, StripeLineErrorCode::NetMismatch)
    );
}
