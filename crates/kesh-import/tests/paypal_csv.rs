//! Lecteur du CSV PayPal sur la fixture fictive (Story 38-11a, AC-1 à AC-7).
//!
//! `tests/fixtures/paypal-balance-affecting-fictif.csv` est inventée de toutes
//! pièces (catalogue : `tests/fixtures/README.md`) : 41 colonnes dans l'ordre
//! PayPal, BOM UTF-8, 14 lignes de données en 2020.

use chrono::NaiveDate;
use kesh_import::paypal::{REQUIRED_COLUMNS, parse_paypal_csv};
use kesh_import::{BalanceImpact, DateOrder, PaypalCsvError, PaypalLineErrorCode};
use rust_decimal_macros::dec;

const FIXTURE: &[u8] = include_bytes!("fixtures/paypal-balance-affecting-fictif.csv");

/// La fixture, sans son BOM, en texte.
fn fixture_text() -> String {
    String::from_utf8(FIXTURE[3..].to_vec()).unwrap()
}

/// Récrit la fixture par le crate `csv` après transformation de chaque
/// enregistrement (en-tête compris) — les guillemets et la note multiligne
/// restent justes.
fn rewrite(f: impl Fn(Vec<String>) -> Vec<String>) -> Vec<u8> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(FIXTURE[3..].as_ref());
    let mut writer = csv::WriterBuilder::new()
        .flexible(true)
        .from_writer(Vec::new());
    for record in reader.records() {
        let cells: Vec<String> = record.unwrap().iter().map(str::to_string).collect();
        writer.write_record(f(cells)).unwrap();
    }
    writer.into_inner().unwrap()
}

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

#[test]
fn fixture_shape_is_the_paypal_one() {
    assert_eq!(&FIXTURE[..3], b"\xEF\xBB\xBF", "BOM UTF-8");
    let text = fixture_text();
    let header = text.lines().next().unwrap();
    assert_eq!(header.split(',').count(), 41);
    assert!(header.starts_with("\"Date\",\"Time\",\"TimeZone\",\"Name\""));
    assert!(header.ends_with("\"Country Code\",\"Balance Impact\""));
    assert!(
        !text.contains('@')
            || text.matches('@').count() == text.matches("@example.invalid").count()
    );
}

#[test]
fn reads_every_line_of_the_fixture() {
    let out = parse_paypal_csv(FIXTURE).unwrap();
    assert_eq!(out.date_order, DateOrder::DayFirst);
    assert_eq!(out.rows.len(), 14);

    // (ligne, date, type, statut, devise, brut, frais, net, sens, lié)
    let expected = [
        (
            2,
            date(2020, 1, 3),
            "Website Payment",
            "Completed",
            "EUR",
            dec!(100.00),
            dec!(-3.20),
            dec!(96.80),
            BalanceImpact::Credit,
            None,
        ),
        (
            3,
            date(2020, 1, 15),
            "Subscription Payment",
            "Completed",
            "USD",
            dec!(20.00),
            dec!(-1.08),
            dec!(18.92),
            BalanceImpact::Credit,
            None,
        ),
        (
            4,
            date(2020, 1, 20),
            "General Payment",
            "Completed",
            "EUR",
            dec!(-50.00),
            dec!(-0.50),
            dec!(-50.50),
            BalanceImpact::Debit,
            None,
        ),
        (
            5,
            date(2020, 2, 5),
            "User Initiated Currency Conversion",
            "Completed",
            "EUR",
            dec!(-40.00),
            dec!(0.00),
            dec!(-40.00),
            BalanceImpact::Debit,
            Some("0TEST00000000005E"),
        ),
        (
            6,
            date(2020, 2, 5),
            "User Initiated Currency Conversion",
            "Completed",
            "USD",
            dec!(44.12),
            dec!(0.00),
            dec!(44.12),
            BalanceImpact::Credit,
            Some("0TEST00000000004D"),
        ),
        (
            7,
            date(2020, 2, 14),
            "Website Payment",
            "Completed",
            "CHF",
            dec!(1234.56),
            dec!(-40.00),
            dec!(1194.56),
            BalanceImpact::Credit,
            None,
        ),
        (
            8,
            date(2020, 2, 28),
            "General Withdrawal",
            "Completed",
            "CHF",
            dec!(-1000.00),
            dec!(0.00),
            dec!(-1000.00),
            BalanceImpact::Debit,
            None,
        ),
        (
            9,
            date(2020, 3, 10),
            "Payment Review Hold",
            "Completed",
            "EUR",
            dec!(-96.80),
            dec!(0.00),
            dec!(-96.80),
            BalanceImpact::Debit,
            Some("0TEST00000000001A"),
        ),
        (
            10,
            date(2020, 3, 12),
            "Payment Review Release",
            "Completed",
            "EUR",
            dec!(96.80),
            dec!(0.00),
            dec!(96.80),
            BalanceImpact::Credit,
            Some("0TEST00000000001A"),
        ),
        (
            11,
            date(2020, 3, 25),
            "Website Payment",
            "Completed",
            "EUR",
            dec!(30.00),
            dec!(-1.26),
            dec!(28.74),
            BalanceImpact::Credit,
            None,
        ),
        (
            12,
            date(2020, 3, 30),
            "Payment Refund",
            "Completed",
            "EUR",
            dec!(-30.00),
            dec!(0.96),
            dec!(-29.04),
            BalanceImpact::Debit,
            Some("0TEST0000000000AK"),
        ),
        // La note de la ligne 12 tient sur deux lignes : la suivante est la 14.
        (
            14,
            date(2020, 4, 16),
            "Website Payment",
            "Pending",
            "USD",
            dec!(15.00),
            dec!(-0.80),
            dec!(14.20),
            BalanceImpact::Credit,
            None,
        ),
        (
            15,
            date(2020, 4, 21),
            "General Payment",
            "Denied",
            "EUR",
            dec!(-12.00),
            dec!(0.00),
            dec!(-12.00),
            BalanceImpact::Debit,
            None,
        ),
        (
            16,
            date(2020, 4, 22),
            "General Authorization",
            "Completed",
            "EUR",
            dec!(-25.00),
            dec!(0.00),
            dec!(-25.00),
            BalanceImpact::Memo,
            None,
        ),
    ];
    for (row, (line, d, kind, status, cur, gross, fee, net, impact, reference)) in
        out.rows.iter().zip(expected)
    {
        assert_eq!(row.line, line);
        assert_eq!(row.date, d, "ligne {line}");
        assert_eq!(row.kind, kind, "ligne {line}");
        assert_eq!(row.status, status, "ligne {line}");
        assert_eq!(row.currency, cur, "ligne {line}");
        assert_eq!(
            (row.gross, row.fee, row.net),
            (gross, fee, net),
            "ligne {line}"
        );
        assert_eq!(row.balance_impact, impact, "ligne {line}");
        assert_eq!(row.reference_txn_id.as_deref(), reference, "ligne {line}");
        assert_eq!(row.transaction_id.len(), 17, "ligne {line}");
        assert!(row.transaction_id.starts_with("0TEST"));
    }

    let first = &out.rows[0];
    assert_eq!(first.transaction_id, "0TEST00000000001A");
    assert_eq!(first.time.as_deref(), Some("09:15:02"));
    assert_eq!(first.time_zone.as_deref(), Some("CET"));
    assert_eq!(first.name.as_deref(), Some("Alice Fictive"));
    assert_eq!(first.from_email.as_deref(), Some("alice@example.invalid"));
    assert_eq!(first.to_email.as_deref(), Some("compte@example.invalid"));
    assert_eq!(first.balance, Some(dec!(96.80)));
    assert_eq!(first.item_title.as_deref(), Some("Commande 1001"));
    assert_eq!(first.invoice_number.as_deref(), Some("WWW-1001"));
    assert_eq!(out.rows[1].subject.as_deref(), Some("Abonnement"));
    assert_eq!(out.rows[3].name, None, "nom vide");
    assert_eq!(out.rows[5].balance, Some(dec!(1194.56)));
    assert_eq!(
        out.rows[10].note.as_deref(),
        Some("Remboursement complet\nà la demande du client")
    );
}

#[test]
fn bom_is_removed_and_optional() {
    let with_bom = parse_paypal_csv(FIXTURE).unwrap();
    let without = parse_paypal_csv(&FIXTURE[3..]).unwrap();
    assert_eq!(with_bom, without);
}

#[test]
fn columns_in_another_order() {
    // En-tête et cellules retournés : même lecture.
    let reversed = rewrite(|mut cells| {
        cells.reverse();
        cells
    });
    assert_eq!(
        parse_paypal_csv(&reversed).unwrap().rows,
        parse_paypal_csv(FIXTURE).unwrap().rows
    );
}

#[test]
fn missing_required_columns_are_all_named() {
    let drop = ["Currency", "Transaction ID", "Balance Impact"];
    let bytes = rewrite(|cells| {
        // Indices de la fixture : 6 Currency, 12 Transaction ID, 40 Balance Impact.
        cells
            .into_iter()
            .enumerate()
            .filter(|(i, _)| ![6, 12, 40].contains(i))
            .map(|(_, c)| c)
            .collect()
    });
    let err = parse_paypal_csv(&bytes).unwrap_err();
    assert_eq!(
        err,
        PaypalCsvError::MissingColumns(drop.iter().map(|s| s.to_string()).collect())
    );
    assert_eq!(err.code(), "PAYPAL_CSV_MISSING_COLUMNS");
}

#[test]
fn a_report_in_another_language_names_every_required_column() {
    let err = parse_paypal_csv(b"Datum,Name,Typ\n01.02.2020,x,y\n").unwrap_err();
    let PaypalCsvError::MissingColumns(names) = err else {
        panic!("MissingColumns attendu");
    };
    assert_eq!(
        names.len(),
        REQUIRED_COLUMNS.len() - 1,
        "seul Name est commun"
    );
    assert_eq!(names[0], "Date");
}

#[test]
fn duplicate_column_is_refused() {
    let bytes = rewrite(|mut cells| {
        cells[3] = if cells[0] == "Date" {
            "Gross".to_string()
        } else {
            cells[7].clone()
        };
        cells
    });
    let err = parse_paypal_csv(&bytes).unwrap_err();
    assert_eq!(err, PaypalCsvError::DuplicateColumn("Gross".to_string()));
    assert_eq!(err.code(), "PAYPAL_CSV_DUPLICATE_COLUMN");
}

#[test]
fn iso_8859_1_is_decoded() {
    // Un accent dès la première ligne : la détection (`detect_encoding`,
    // partagée avec le CSV bancaire) n'examine que les 1024 premiers octets,
    // et un fichier Latin-1 tout ASCII sur cette longueur y est refusé
    // (limite du détecteur commun, hors de cette story).
    let text = format!(
        "{}\n\"15/01/2020\",\"Société Fictive Sàrl\",\"Website Payment\",\"Completed\",\"EUR\",\
         \"10.00\",\"-0.50\",\"9.50\",\"\",\"\",\"0TEST00000000001A\",\"\",\"Credit\"\n",
        REQUIRED_COLUMNS.join(",")
    );
    let latin1: Vec<u8> = text
        .chars()
        .map(|c| u8::try_from(u32::from(c)).expect("caractère Latin-1"))
        .collect();
    assert!(
        std::str::from_utf8(&latin1).is_err(),
        "le fichier n'est pas de l'UTF-8"
    );
    let out = parse_paypal_csv(&latin1).unwrap();
    assert_eq!(out.rows[0].name.as_deref(), Some("Société Fictive Sàrl"));
}

#[test]
fn wrong_field_count_names_the_file_line() {
    // Une cellule de trop à la ligne 14 (après la note sur deux lignes).
    let mut lines: Vec<String> = Vec::new();
    let text = fixture_text();
    for (i, l) in text.lines().enumerate() {
        lines.push(if i == 13 {
            format!("{l},\"extra\"")
        } else {
            l.to_string()
        });
    }
    let err = parse_paypal_csv(lines.join("\n").as_bytes()).unwrap_err();
    let PaypalCsvError::Lines(errors) = err else {
        panic!("Lines attendu");
    };
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].line, 14);
    assert_eq!(errors[0].code, PaypalLineErrorCode::FieldCount);
    assert_eq!(errors[0].field, None);
}

#[test]
fn one_faulty_line_refuses_the_whole_file() {
    let bytes = rewrite(|mut cells| {
        if cells[12] == "0TEST00000000003C" {
            cells[9] = "-50.40".to_string(); // net ≠ brut + frais
        }
        cells
    });
    let PaypalCsvError::Lines(errors) = parse_paypal_csv(&bytes).unwrap_err() else {
        panic!("Lines attendu");
    };
    assert_eq!(errors.len(), 1);
    assert_eq!(
        (errors[0].line, errors[0].code),
        (4, PaypalLineErrorCode::NetMismatch)
    );
    assert_eq!(errors[0].field, Some("Net"));
}

/// Valeur extrême de `Decimal` : refusée en erreur de ligne, sans panique
/// (correctif post-revue : débordement des montants).
#[test]
fn extreme_amount_is_refused_without_panicking() {
    let extreme = "79228162514264337593543950335".to_string();
    let bytes = rewrite(|mut cells| {
        if cells[12] == "0TEST00000000003C" {
            cells[7] = extreme.clone(); // Gross
            cells[8] = extreme.clone(); // Fee
        }
        cells
    });
    let PaypalCsvError::Lines(errors) = parse_paypal_csv(&bytes).unwrap_err() else {
        panic!("Lines attendu");
    };
    assert!(errors.iter().all(|e| e.line == 4));
    assert_eq!(errors[0].code, PaypalLineErrorCode::Amount);
}
