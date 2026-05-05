//! Parseur CSV bancaire (Story 8-2).
//!
//! Pipeline : `parse_csv(bytes, profile)` :
//! 1. [`detect_encoding`](super::encoding::detect_encoding) — retourne
//!    encoding + `bom_len`.
//! 2. [`decode_bytes`](super::encoding::decode_bytes) → `String`.
//! 3. `csv::ReaderBuilder` configuré sur `profile.field_separator` +
//!    `has_headers(header_row_count > 0)`. Skip additionnel
//!    `saturating_sub(1)` (Pass 1 H4) pour `header_row_count > 1`.
//! 4. Lit le **premier record** post header skip → early-reject
//!    `CsvError::ProfileMisconfigured` si indices `column_mapping`
//!    hors-borne (Pass 2 M'1 + Pass 3 M''2).
//! 5. Itère sur tous les records (incluant le 1er) : pour chaque
//!    ligne, mappe les cellules via `column_mapping` → soit
//!    `ImportedTransaction` valide, soit `CsvLineError` collecté.
//! 6. Cap collection à `MAX_CSV_LINE_ERRORS = 100` (Pass 2 H'1)
//!    avec `total_errors` + `truncated` flag.
//! 7. Si `transactions.is_empty()` → `CsvError::EmptyFile`
//!    (Pass 2 M'2 header-only file).
//! 8. Si `errors` non-vide → `CsvError::PartialFailure { ... }`
//!    (strict reject FR51 v0.1).
//! 9. Sinon retourne `ImportedStatement` avec
//!    `period_from = min(booking_dates)` et
//!    `period_to = max(booking_dates)` (Pass 1 H2).

use crate::csv::encoding::{decode_bytes, detect_encoding};
use crate::csv::profile::CsvProfile;
use crate::error::{CsvError, CsvLineError, CsvLineErrorCode, MAX_CSV_LINE_ERRORS};
use crate::types::{ImportedStatement, ImportedTransaction, SourceFormat};
use chrono::NaiveDate;
use rust_decimal::Decimal;

/// Parse un fichier CSV bancaire selon le profil.
///
/// Retourne un `ImportedStatement` complet avec `period_from`/`period_to`
/// calculés depuis les booking_dates si succès. En cas d'erreur, voir
/// [`CsvError`] pour la sémantique de chaque variante.
pub fn parse_csv(bytes: &[u8], profile: &CsvProfile) -> Result<ImportedStatement, CsvError> {
    profile.validate()?;

    // 1. Détection encoding + skip BOM
    let (detected_encoding, bom_len) = detect_encoding(bytes)?;

    // 2. Si profil spécifie un encoding non-null différent de la détection :
    //    EncodingMismatch (handler API gère le branching preview/final).
    if let Some(ref profile_encoding) = profile.encoding {
        if profile_encoding != detected_encoding.as_str() {
            return Err(CsvError::EncodingMismatch {
                profile: profile_encoding.clone(),
                detected: detected_encoding.as_str().to_string(),
            });
        }
    }

    // 3. Decode (post BOM)
    let decoded = decode_bytes(&bytes[bom_len..], detected_encoding)?;

    // 4. csv::ReaderBuilder
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(profile.field_separator as u8)
        .has_headers(profile.header_row_count > 0)
        .flexible(true) // tolère lignes de longueur variable, on gère l'erreur RowTooShort manuellement
        .from_reader(decoded.as_bytes());

    // 5. Skip extra header rows (Pass 1 H4 saturating_sub anti-underflow)
    let extra_skip = profile.header_row_count.saturating_sub(1) as usize;
    let mut iter = reader.records();
    for _ in 0..extra_skip {
        match iter.next() {
            Some(Ok(_)) => continue,
            Some(Err(e)) => return Err(CsvError::Io(format!("skip header row : {}", e))),
            None => {
                // Pas assez de lignes pour le header → MissingHeader.
                return Err(CsvError::MissingHeader);
            }
        }
    }

    // 6. Lire 1er record de données pour early-reject indices OOB
    //    (Pass 2 M'1 + Pass 3 M''2 parser-side).
    let first_record = match iter.next() {
        Some(Ok(rec)) => rec,
        Some(Err(e)) => return Err(CsvError::Io(format!("first data row : {}", e))),
        None => {
            // 0 data rows post header skip → EmptyFile (Pass 2 M'2).
            return Err(CsvError::EmptyFile {
                reason: "0 data rows after header skip".to_string(),
            });
        }
    };

    // 6.b Validation indices hors-borne sur 1er record.
    let max_idx = profile.column_mapping.max_index();
    if max_idx >= first_record.len() {
        // Trouver quel champ est OOB pour message ciblé.
        for (name, idx) in profile.column_mapping.all_indices() {
            if idx >= first_record.len() {
                return Err(CsvError::ProfileMisconfigured(format!(
                    "column_mapping.{} (index {}) out of bounds for {} columns",
                    name,
                    idx,
                    first_record.len()
                )));
            }
        }
    }

    // 7. Itérer sur tous les records (1er + suite)
    let mut transactions: Vec<ImportedTransaction> = Vec::new();
    let mut errors: Vec<CsvLineError> = Vec::new();
    let mut total_errors: usize = 0;

    let process = |record: &csv::StringRecord,
                   line: usize,
                   transactions: &mut Vec<ImportedTransaction>,
                   errors: &mut Vec<CsvLineError>,
                   total_errors: &mut usize| {
        match parse_row(record, line, profile) {
            Ok(tx) => transactions.push(tx),
            Err(line_err) => {
                *total_errors += 1;
                if errors.len() < MAX_CSV_LINE_ERRORS {
                    errors.push(line_err);
                }
            }
        }
    };

    let first_line = first_record.position().map(|p| p.line()).unwrap_or(0) as usize;
    process(
        &first_record,
        first_line,
        &mut transactions,
        &mut errors,
        &mut total_errors,
    );

    for result in iter {
        match result {
            Ok(record) => {
                let line = record.position().map(|p| p.line()).unwrap_or(0) as usize;
                process(
                    &record,
                    line,
                    &mut transactions,
                    &mut errors,
                    &mut total_errors,
                );
            }
            Err(e) => {
                total_errors += 1;
                if errors.len() < MAX_CSV_LINE_ERRORS {
                    errors.push(CsvLineError::new(
                        0,
                        CsvLineErrorCode::RowTooShort,
                        Some(e.to_string()),
                    ));
                }
            }
        }
    }

    // 8. Strict reject si erreurs (FR51 v0.1)
    if !errors.is_empty() {
        let truncated = total_errors > MAX_CSV_LINE_ERRORS;
        return Err(CsvError::PartialFailure {
            errors,
            total_errors,
            truncated,
        });
    }

    // 9. EmptyFile si 0 transactions valides après iteration complète.
    if transactions.is_empty() {
        return Err(CsvError::EmptyFile {
            reason: "0 valid transactions after parsing".to_string(),
        });
    }

    // 10. Calcul period_from / period_to (Pass 1 H2)
    let period_from = transactions
        .iter()
        .map(|t| t.booking_date)
        .min()
        .expect("transactions non-vide vérifié ligne 8");
    let period_to = transactions
        .iter()
        .map(|t| t.booking_date)
        .max()
        .expect("transactions non-vide vérifié ligne 8");

    // 11. Construire ImportedStatement
    //
    // Note Pass 2 M'4 alignement : `ImportedStatement.account_iban: String`
    // (pas Option). CSV n'expose pas d'IBAN → string vide. Le frontend
    // serialise `accountIban === ""` comme « pas d'IBAN » côté UI
    // (cf. §preview-csv-response-shape). statement_id reste Option.
    Ok(ImportedStatement {
        source_format: SourceFormat::Csv {
            encoding: detected_encoding.as_str().to_string(),
            profile_name: Some(profile.bank_name.clone()),
        },
        statement_id: None,
        account_iban: String::new(),
        currency: "CHF".to_string(),
        period_from,
        period_to,
        opening_balance: None,
        closing_balance: None,
        transactions,
    })
}

/// Parse une ligne CSV en `ImportedTransaction` ou `CsvLineError`.
fn parse_row(
    record: &csv::StringRecord,
    line: usize,
    profile: &CsvProfile,
) -> Result<ImportedTransaction, CsvLineError> {
    let cm = &profile.column_mapping;

    // Helper : récupère cellule trim ou None si Optional + empty.
    let get_cell = |idx: usize| -> Option<&str> { record.get(idx).map(|s| s.trim()) };

    // RowTooShort : si le 1er record passait, les autres pourraient être
    // plus courts (csv flexible mode). Vérifier une dernière fois.
    let max_idx = profile.column_mapping.max_index();
    if record.len() <= max_idx {
        return Err(CsvLineError::new(line, CsvLineErrorCode::RowTooShort, None));
    }

    // Date (obligatoire)
    let date_raw = match get_cell(cm.date) {
        Some(s) if !s.is_empty() => s,
        _ => {
            return Err(CsvLineError::new(
                line,
                CsvLineErrorCode::EmptyMandatoryField,
                Some("date".to_string()),
            ));
        }
    };
    let booking_date = NaiveDate::parse_from_str(date_raw, &profile.date_format).map_err(|_| {
        CsvLineError::new(
            line,
            CsvLineErrorCode::InvalidDate,
            Some(date_raw.to_string()),
        )
    })?;

    // Amount : XOR amount vs debit_credit_split. Priorité parse au
    // debit_credit_split (Pass 1 M16 — DB corrompue protection).
    let amount = if let Some((debit_idx, credit_idx)) = cm.debit_credit_split {
        let debit_raw = get_cell(debit_idx).unwrap_or("");
        let credit_raw = get_cell(credit_idx).unwrap_or("");
        let debit_empty = debit_raw.is_empty();
        let credit_empty = credit_raw.is_empty();
        match (debit_empty, credit_empty) {
            (true, true) => {
                return Err(CsvLineError::new(
                    line,
                    CsvLineErrorCode::EmptyMandatoryField,
                    Some("amount (debit+credit empty)".to_string()),
                ));
            }
            (false, false) => {
                return Err(CsvLineError::new(
                    line,
                    CsvLineErrorCode::AmbiguousDebitCredit,
                    None,
                ));
            }
            (false, true) => {
                // débit non-vide → amount négatif
                let parsed = parse_amount(debit_raw, profile.decimal_separator).map_err(|_| {
                    CsvLineError::new(
                        line,
                        CsvLineErrorCode::InvalidAmount,
                        Some(debit_raw.to_string()),
                    )
                })?;
                -parsed
            }
            (true, false) => {
                // crédit non-vide → amount positif
                parse_amount(credit_raw, profile.decimal_separator).map_err(|_| {
                    CsvLineError::new(
                        line,
                        CsvLineErrorCode::InvalidAmount,
                        Some(credit_raw.to_string()),
                    )
                })?
            }
        }
    } else if let Some(amount_idx) = cm.amount {
        let amount_raw = match get_cell(amount_idx) {
            Some(s) if !s.is_empty() => s,
            _ => {
                return Err(CsvLineError::new(
                    line,
                    CsvLineErrorCode::EmptyMandatoryField,
                    Some("amount".to_string()),
                ));
            }
        };
        parse_amount(amount_raw, profile.decimal_separator).map_err(|_| {
            CsvLineError::new(
                line,
                CsvLineErrorCode::InvalidAmount,
                Some(amount_raw.to_string()),
            )
        })?
    } else {
        // validate() bloque déjà ce cas, mais defense in depth
        return Err(CsvLineError::new(
            line,
            CsvLineErrorCode::InvalidAmount,
            Some("profil invalide : ni amount ni debit_credit_split".to_string()),
        ));
    };

    // Reference (optionnel) : trim, empty → None.
    let reference = cm
        .reference
        .and_then(|i| get_cell(i).filter(|s| !s.is_empty()).map(String::from));

    // Details (Pass 2 M'4 : String pas Option) : trim, empty → "".
    let details = cm
        .details
        .and_then(get_cell)
        .map(String::from)
        .unwrap_or_default();

    // Counterparty name (optionnel)
    let counterparty_name = cm
        .counterparty
        .and_then(|i| get_cell(i).filter(|s| !s.is_empty()).map(String::from));

    Ok(ImportedTransaction {
        booking_date,
        value_date: Some(booking_date),
        amount,
        currency: "CHF".to_string(),
        reference,
        details,
        end_to_end_id: None,
        transaction_id: None,
        counterparty_iban: None,
        counterparty_name,
    })
}

/// Parse un montant string vers `Decimal`, en gérant :
/// - apostrophe milliers suisse (`1'234.56` → strip `'`)
/// - decimal_separator = `,` → remplacer par `.` avant parse
fn parse_amount(raw: &str, decimal_sep: char) -> Result<Decimal, ()> {
    let s = raw.replace('\'', "");
    let s = if decimal_sep == ',' {
        s.replace(',', ".")
    } else {
        s
    };
    Decimal::from_str_exact(&s).map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv::profile::ColumnMapping;
    use rust_decimal_macros::dec;

    fn make_profile(
        field_sep: char,
        decimal_sep: char,
        header_count: u8,
        column_mapping: ColumnMapping,
    ) -> CsvProfile {
        CsvProfile {
            bank_name: "Test".to_string(),
            filename_pattern: None,
            column_mapping,
            date_format: "%Y-%m-%d".to_string(),
            decimal_separator: decimal_sep,
            field_separator: field_sep,
            encoding: None,
            header_row_count: header_count,
        }
    }

    #[test]
    fn parses_minimal_utf8_with_bom() {
        let csv = "\u{FEFF}date;amount;ref;details\n2026-01-15;100.00;R1;Loyer\n2026-01-16;-50.00;R2;Achat\n";
        let profile = make_profile(
            ';',
            '.',
            1,
            ColumnMapping {
                date: 0,
                amount: Some(1),
                debit_credit_split: None,
                reference: Some(2),
                details: Some(3),
                counterparty: None,
            },
        );
        let stmt = parse_csv(csv.as_bytes(), &profile).unwrap();
        assert_eq!(stmt.transactions.len(), 2);
        assert_eq!(stmt.transactions[0].amount, dec!(100.00));
        assert_eq!(stmt.transactions[1].amount, dec!(-50.00));
        assert_eq!(
            stmt.period_from,
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()
        );
        assert_eq!(
            stmt.period_to,
            NaiveDate::from_ymd_opt(2026, 1, 16).unwrap()
        );
        assert_eq!(stmt.currency, "CHF");
    }

    #[test]
    fn parses_swiss_amount_with_apostrophe_thousands_and_comma_decimal() {
        // AC #25
        let csv = "date;montant\n2026-01-15;1'234,56\n";
        let profile = make_profile(
            ';',
            ',',
            1,
            ColumnMapping {
                date: 0,
                amount: Some(1),
                debit_credit_split: None,
                reference: None,
                details: None,
                counterparty: None,
            },
        );
        let stmt = parse_csv(csv.as_bytes(), &profile).unwrap();
        assert_eq!(stmt.transactions[0].amount, dec!(1234.56));
    }

    #[test]
    fn parses_csv_with_zero_header_rows() {
        // AC #5ter Pass 1 H4
        let csv = "2026-01-15,100.00\n2026-01-16,200.00\n";
        let profile = make_profile(
            ',',
            '.',
            0,
            ColumnMapping {
                date: 0,
                amount: Some(1),
                debit_credit_split: None,
                reference: None,
                details: None,
                counterparty: None,
            },
        );
        let stmt = parse_csv(csv.as_bytes(), &profile).unwrap();
        assert_eq!(stmt.transactions.len(), 2);
    }

    #[test]
    fn parses_debit_credit_split_correctly() {
        let csv = "date;debit;credit\n2026-01-15;100.00;\n2026-01-16;;200.00\n";
        let profile = make_profile(
            ';',
            '.',
            1,
            ColumnMapping {
                date: 0,
                amount: None,
                debit_credit_split: Some((1, 2)),
                reference: None,
                details: None,
                counterparty: None,
            },
        );
        let stmt = parse_csv(csv.as_bytes(), &profile).unwrap();
        assert_eq!(stmt.transactions[0].amount, dec!(-100.00));
        assert_eq!(stmt.transactions[1].amount, dec!(200.00));
    }

    #[test]
    fn debit_credit_both_filled_returns_ambiguous_error() {
        // AC #15bis
        let csv = "date;debit;credit\n2026-01-15;100.00;50.00\n";
        let profile = make_profile(
            ';',
            '.',
            1,
            ColumnMapping {
                date: 0,
                amount: None,
                debit_credit_split: Some((1, 2)),
                reference: None,
                details: None,
                counterparty: None,
            },
        );
        let err = parse_csv(csv.as_bytes(), &profile).unwrap_err();
        match err {
            CsvError::PartialFailure { errors, .. } => {
                assert_eq!(errors[0].code, CsvLineErrorCode::AmbiguousDebitCredit);
            }
            _ => panic!("expected PartialFailure"),
        }
    }

    #[test]
    fn debit_credit_both_empty_returns_empty_mandatory_field() {
        // AC #15ter Pass 1 M10
        let csv = "date;debit;credit\n2026-01-15;;\n";
        let profile = make_profile(
            ';',
            '.',
            1,
            ColumnMapping {
                date: 0,
                amount: None,
                debit_credit_split: Some((1, 2)),
                reference: None,
                details: None,
                counterparty: None,
            },
        );
        let err = parse_csv(csv.as_bytes(), &profile).unwrap_err();
        match err {
            CsvError::PartialFailure { errors, .. } => {
                assert_eq!(errors[0].code, CsvLineErrorCode::EmptyMandatoryField);
            }
            _ => panic!("expected PartialFailure"),
        }
    }

    #[test]
    fn explicit_zero_debit_or_credit_accepted() {
        // AC #15ter — `"0.00"` explicite valide
        let csv = "date;debit;credit\n2026-01-15;0.00;\n2026-01-16;;0,00\n";
        let profile = make_profile(
            ';',
            ',',
            1,
            ColumnMapping {
                date: 0,
                amount: None,
                debit_credit_split: Some((1, 2)),
                reference: None,
                details: None,
                counterparty: None,
            },
        );
        // Note : ici decimal_separator = ',' donc "0.00" devient "0.00" → "0.00" (pas de remplacement),
        // puis Decimal::from_str_exact("0.00") = 0. Mais "0.00" sans virgule passe quand même
        // (Decimal accepte "0.00" comme zéro). OK.
        let stmt = parse_csv(csv.as_bytes(), &profile).unwrap();
        assert_eq!(stmt.transactions[0].amount, dec!(0));
        assert_eq!(stmt.transactions[1].amount, dec!(0));
    }

    #[test]
    fn partial_failure_caps_at_100_errors_for_huge_invalid_csv() {
        // Pass 2 H'1 cap 100
        let mut csv = String::from("date;amount\n");
        for _ in 0..500 {
            csv.push_str("invalid_date;100.00\n");
        }
        let profile = make_profile(
            ';',
            '.',
            1,
            ColumnMapping {
                date: 0,
                amount: Some(1),
                debit_credit_split: None,
                reference: None,
                details: None,
                counterparty: None,
            },
        );
        let err = parse_csv(csv.as_bytes(), &profile).unwrap_err();
        match err {
            CsvError::PartialFailure {
                errors,
                total_errors,
                truncated,
            } => {
                assert_eq!(errors.len(), 100);
                assert_eq!(total_errors, 500);
                assert!(truncated);
            }
            _ => panic!("expected PartialFailure"),
        }
    }

    #[test]
    fn parser_early_rejects_oob_indices_on_first_record() {
        // AC #15quater Pass 2 M'1 + Pass 3 M''2
        let csv = "date;amount\n2026-01-15;100.00\n";
        let profile = make_profile(
            ';',
            '.',
            1,
            ColumnMapping {
                date: 0,
                amount: Some(99), // OOB sur 2 colonnes
                debit_credit_split: None,
                reference: None,
                details: None,
                counterparty: None,
            },
        );
        let err = parse_csv(csv.as_bytes(), &profile).unwrap_err();
        match err {
            CsvError::ProfileMisconfigured(msg) => {
                assert!(msg.contains("amount"));
                assert!(msg.contains("99"));
                assert!(msg.contains("2 columns"));
            }
            other => panic!("expected ProfileMisconfigured, got {:?}", other),
        }
    }

    #[test]
    fn rejects_header_only_file_with_empty_file() {
        // AC #5bis-c Pass 2 M'2
        let csv = "date;amount\n";
        let profile = make_profile(
            ';',
            '.',
            1,
            ColumnMapping {
                date: 0,
                amount: Some(1),
                debit_credit_split: None,
                reference: None,
                details: None,
                counterparty: None,
            },
        );
        let err = parse_csv(csv.as_bytes(), &profile).unwrap_err();
        match err {
            CsvError::EmptyFile { reason } => {
                assert!(reason.contains("0 data rows"));
            }
            _ => panic!("expected EmptyFile"),
        }
    }

    #[test]
    fn period_from_to_calculated_from_min_max_booking_dates() {
        // Pass 1 H2
        let csv = "date;amount\n2026-03-15;100\n2026-01-05;200\n2026-02-20;300\n";
        let profile = make_profile(
            ';',
            '.',
            1,
            ColumnMapping {
                date: 0,
                amount: Some(1),
                debit_credit_split: None,
                reference: None,
                details: None,
                counterparty: None,
            },
        );
        let stmt = parse_csv(csv.as_bytes(), &profile).unwrap();
        assert_eq!(
            stmt.period_from,
            NaiveDate::from_ymd_opt(2026, 1, 5).unwrap()
        );
        assert_eq!(
            stmt.period_to,
            NaiveDate::from_ymd_opt(2026, 3, 15).unwrap()
        );
    }

    #[test]
    fn rejects_encoding_mismatch_when_profile_specifies_iso_but_file_is_utf8() {
        // AC #5quinquies-b Pass 1 H5
        let csv = "date;amount\n2026-01-15;100\n";
        let mut profile = make_profile(
            ';',
            '.',
            1,
            ColumnMapping {
                date: 0,
                amount: Some(1),
                debit_credit_split: None,
                reference: None,
                details: None,
                counterparty: None,
            },
        );
        profile.encoding = Some("ISO-8859-1".to_string());
        let err = parse_csv(csv.as_bytes(), &profile).unwrap_err();
        match err {
            CsvError::EncodingMismatch { profile, detected } => {
                assert_eq!(profile, "ISO-8859-1");
                assert_eq!(detected, "UTF-8");
            }
            other => panic!("expected EncodingMismatch, got {:?}", other),
        }
    }

    #[test]
    fn csv_line_error_value_truncated_at_100_chars() {
        // Pass 3 M''3 — UTF-8-aware truncation
        let long_value: String = "é".repeat(200); // 200 caractères Unicode (400 bytes UTF-8)
        let err = CsvLineError::new(7, CsvLineErrorCode::InvalidAmount, Some(long_value));
        let truncated = err.value.unwrap();
        let char_count = truncated.chars().count();
        // 100 chars + 1 ellipsis = 101 chars total
        assert_eq!(char_count, 101);
        assert!(truncated.ends_with('…'));
    }

    #[test]
    fn invalid_date_collected_as_partial_failure() {
        let csv = "date;amount\n32/13/2026;100\n";
        let profile = make_profile(
            ';',
            '.',
            1,
            ColumnMapping {
                date: 0,
                amount: Some(1),
                debit_credit_split: None,
                reference: None,
                details: None,
                counterparty: None,
            },
        );
        let err = parse_csv(csv.as_bytes(), &profile).unwrap_err();
        match err {
            CsvError::PartialFailure { errors, .. } => {
                assert_eq!(errors[0].code, CsvLineErrorCode::InvalidDate);
                assert_eq!(errors[0].value, Some("32/13/2026".to_string()));
            }
            _ => panic!("expected PartialFailure"),
        }
    }

    #[test]
    fn empty_details_field_produces_empty_string_not_none() {
        // Pass 2 M'4 alignement details:String avec DB NOT NULL
        let csv = "date;amount;details\n2026-01-15;100;\n";
        let profile = make_profile(
            ';',
            '.',
            1,
            ColumnMapping {
                date: 0,
                amount: Some(1),
                debit_credit_split: None,
                reference: None,
                details: Some(2),
                counterparty: None,
            },
        );
        let stmt = parse_csv(csv.as_bytes(), &profile).unwrap();
        assert_eq!(stmt.transactions[0].details, "");
    }
}
