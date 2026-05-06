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

/// Sentinel publique pour `period_from`/`period_to` quand
/// `parse_csv_collect` retourne `PartialFailure { valid: stmt, ... }`
/// avec `stmt.transactions.is_empty()` (M9, Pass 1 review).
///
/// Le caller `kesh-api::create_csv` doit comparer explicitement contre
/// cette sentinel avant tout usage en DB (typiquement
/// `find_in_dedup_window`) afin d'éviter un BETWEEN scan complet de
/// l'historique. La voie nominale est de rejeter avec
/// `reason = "no_valid_lines_to_commit"` (AC #16) avant cet appel.
///
/// Pourquoi 1970-01-01 et pas `NaiveDate::MIN` (an -262143) : si la
/// garde caller est jamais ratée par un futur refactor, le SQL BETWEEN
/// scanne au pire 56 ans plutôt que 263 000 ans — moins catastrophique.
pub fn empty_valid_sentinel_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(1970, 1, 1).expect("1970-01-01 is a valid NaiveDate")
}

/// Issue du parser CSV en mode « collect » (Story 8-3 T4).
///
/// Discrimine trois résultats :
///
/// - [`ParseCsvOutcome::AllValid`] : toutes les lignes data ont parsé
///   avec succès — l'`ImportedStatement` est complet.
/// - [`ParseCsvOutcome::PartialFailure`] : au moins une ligne data a
///   échoué (mais le parsing global a démarré sans erreur fatale). Le
///   caller décide :
///     - `confirmPartialImport=false` (8-2 strict reject) → retourne
///       `422 BANK_CSV_PARTIAL_FAILURE`.
///     - `confirmPartialImport=true` (8-3 partial commit) → persiste les
///       transactions valides + retourne un warning `invalidLines`.
///
///   La partie `valid: ImportedStatement` peut avoir un `transactions`
///   vide (cas « 0 valides parmi N invalides », AC #16) auquel cas le
///   handler retourne `422 BANK_CSV_PARTIAL_FAILURE` avec
///   `reason = "no_valid_lines_to_commit"`.
/// - [`ParseCsvOutcome::HardFailure`] : le parser n'a pas pu démarrer
///   (encoding non supporté, profil mal configuré, fichier vide). Le
///   caller mappe vers la variante `AppError` correspondante.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseCsvOutcome {
    AllValid(ImportedStatement),
    PartialFailure {
        valid: ImportedStatement,
        errors: Vec<CsvLineError>,
        total_errors: usize,
        truncated: bool,
    },
    HardFailure(CsvError),
}

/// Parse un fichier CSV bancaire selon le profil.
///
/// Wrapper legacy 8-2 autour de [`parse_csv_collect`] (Story 8-3 T4) :
/// convertit `ParseCsvOutcome::PartialFailure` en
/// `Err(CsvError::PartialFailure { errors, total_errors, truncated })`
/// (sans le champ `valid`). **Backward-compat absolue** — les tests
/// 8-2 ne changent pas.
///
/// Pour les nouveaux call-sites (Story 8-3 partial commit), utiliser
/// [`parse_csv_collect`].
pub fn parse_csv(bytes: &[u8], profile: &CsvProfile) -> Result<ImportedStatement, CsvError> {
    match parse_csv_collect(bytes, profile) {
        ParseCsvOutcome::AllValid(stmt) => Ok(stmt),
        ParseCsvOutcome::PartialFailure {
            errors,
            total_errors,
            truncated,
            ..
        } => Err(CsvError::PartialFailure {
            errors,
            total_errors,
            truncated,
        }),
        ParseCsvOutcome::HardFailure(e) => Err(e),
    }
}

/// Parse un fichier CSV bancaire en mode « collect » (Story 8-3 T4).
///
/// Contrairement à [`parse_csv`] (strict reject 8-2), `parse_csv_collect`
/// **collecte** systématiquement les transactions valides ET les
/// erreurs ligne — laisse au caller le choix de rejeter strictement ou
/// de persister les valides via le flag multipart `confirmPartialImport`.
///
/// Retourne [`ParseCsvOutcome`] discriminant les 3 issues. Les caps
/// anti-DoS hérités 8-2 (`MAX_CSV_LINE_ERRORS = 100` + flag
/// `truncated`) sont préservés.
pub fn parse_csv_collect(bytes: &[u8], profile: &CsvProfile) -> ParseCsvOutcome {
    match parse_csv_collect_inner(bytes, profile) {
        Ok(outcome) => outcome,
        Err(e) => ParseCsvOutcome::HardFailure(e),
    }
}

/// Helper interne : retourne `Ok(AllValid|PartialFailure)` pour le cas
/// nominal et `Err(CsvError)` pour les hard failures (encoding, profile,
/// missing header, empty file). [`parse_csv_collect`] convertit
/// `Err(...)` en [`ParseCsvOutcome::HardFailure`] uniformément.
fn parse_csv_collect_inner(
    bytes: &[u8],
    profile: &CsvProfile,
) -> Result<ParseCsvOutcome, CsvError> {
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
    // Pass 1 review G1 H5 : check défensif sur le cast `field_separator as u8`.
    // `validate()` borne déjà le char à `{',', ';', '\t'}` (tous ASCII), mais
    // si un CsvProfile est construit directement sans validate(), un char
    // Unicode > U+00FF tronquerait silencieusement à un byte arbitraire.
    let field_sep_byte = u8::try_from(profile.field_separator as u32).map_err(|_| {
        CsvError::ProfileMisconfigured(format!(
            "field_separator '{}' n'est pas ASCII",
            profile.field_separator.escape_default()
        ))
    })?;
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(field_sep_byte)
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

    // Pass 1 review G1 H3 : ligne vide post-header (csv::flexible(true)
    // accepte des records de longueur 0) → diagnostic "EmptyFile" plutôt
    // que "ProfileMisconfigured" qui induirait l'utilisateur en erreur.
    if first_record.is_empty() {
        return Err(CsvError::EmptyFile {
            reason: "blank line after header skip".to_string(),
        });
    }

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
                // Pass 1 review G1 H2 : extraire la position depuis l'erreur
                // csv (champ quoted mal formé, NUL byte, etc.) plutôt que
                // hardcoder line=0. Le crate csv 1.3 expose `position()`
                // sur `csv::Error` via la trait `IntoInnerError` indirect ;
                // en pratique le `kind` `Utf8` ou `UnequalLengths` peut
                // contenir une `Position`. Fallback line=0 si non dispo.
                let line = match e.kind() {
                    csv::ErrorKind::Utf8 { pos: Some(p), .. } => p.line() as usize,
                    csv::ErrorKind::UnequalLengths { pos: Some(p), .. } => p.line() as usize,
                    _ => 0,
                };
                if errors.len() < MAX_CSV_LINE_ERRORS {
                    errors.push(CsvLineError::new(
                        line,
                        CsvLineErrorCode::RowTooShort,
                        Some(e.to_string()),
                    ));
                }
            }
        }
    }

    // 8. Construit l'`ImportedStatement` candidat (avec les transactions
    //    valides, possiblement vide si toutes ont échoué). Story 8-3 T4 :
    //    `parse_csv_collect` retourne toujours un `ImportedStatement` —
    //    le caller décide de rejeter (strict mode 8-2) ou persister
    //    (partial commit 8-3) selon `confirmPartialImport`.
    //
    // Note Pass 2 M'4 alignement : `ImportedStatement.account_iban: String`
    // (pas Option). CSV n'expose pas d'IBAN → string vide. Le frontend
    // serialise `accountIban === ""` comme « pas d'IBAN » côté UI
    // (cf. §preview-csv-response-shape). statement_id reste Option.
    let sentinel = empty_valid_sentinel_date();
    let (period_from, period_to) = if transactions.is_empty() {
        // M9 (Pass 1 review) — sentinel epoch 1970-01-01 (cf. doc-comment
        // de [`empty_valid_sentinel_date`]). Le caller doit comparer
        // explicitement avant tout usage en DB.
        (sentinel, sentinel)
    } else {
        let from = transactions
            .iter()
            .map(|t| t.booking_date)
            .min()
            .expect("transactions non-vide");
        let to = transactions
            .iter()
            .map(|t| t.booking_date)
            .max()
            .expect("transactions non-vide");
        (from, to)
    };
    debug_assert!(
        (period_from == sentinel) == transactions.is_empty(),
        "empty_valid_sentinel_date invariant: sentinel iff transactions.is_empty()"
    );

    let stmt = ImportedStatement {
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
    };

    // 9. Discrimine AllValid vs PartialFailure vs EmptyFile.
    if !errors.is_empty() {
        let truncated = total_errors > MAX_CSV_LINE_ERRORS;
        return Ok(ParseCsvOutcome::PartialFailure {
            valid: stmt,
            errors,
            total_errors,
            truncated,
        });
    }

    // 0 erreurs ET 0 valides — défensif (en pratique unreachable post
    // first_record check, mais garde la sémantique 8-2 EmptyFile).
    if stmt.transactions.is_empty() {
        return Err(CsvError::EmptyFile {
            reason: "0 valid transactions after parsing".to_string(),
        });
    }

    Ok(ParseCsvOutcome::AllValid(stmt))
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
/// - decimal_separator = `,` → strip point milliers européen + remplacer
///   virgule décimale par point. Pass 1 review G1 H1 : `"1.234,56"`
///   (format allemand/suisse-allemand typique) doit parser comme 1234.56,
///   pas être rejeté `InvalidAmount`. Quand `decimal_sep == ','`, le `.`
///   est non-ambigu = séparateur milliers et peut être strip.
/// - decimal_separator = `.` → strip espace insécable U+00A0 et espace
///   ASCII (séparateurs milliers européens) avant parse.
///
/// **Notation scientifique** : `Decimal::from_str_exact` rejette explicitement
/// `1.5e10`, `NaN`, `inf` (vs `from_str`). Rejet documenté §csv-parser.
///
/// **Préfixe devise** : `"CHF 1234.56"` n'est PAS supporté (rejeté
/// `InvalidAmount`). Documenté pour éviter les rapports faux-bug.
fn parse_amount(raw: &str, decimal_sep: char) -> Result<Decimal, ()> {
    // Strip apostrophe (séparateur milliers suisse).
    let s = raw.replace('\'', "");
    // Strip espaces ASCII et insécables (séparateurs milliers européens).
    let s = s.replace([' ', '\u{00A0}'], "");

    // Pass 2 review M1 (EH2-1) : détection ambiguïté format mélangé.
    // Si la valeur contient à la fois `.` et `,`, vérifier que l'ordre
    // est cohérent avec le decimal_sep du profil :
    // - decimal_sep == ',' (DE) : `1.234,56` OK car `.` (milliers) AVANT
    //   `,` (décimal). `1,500.00` (US) → INCOHÉRENT car `,` avant `.`.
    // - decimal_sep == '.' (US) : `1,234.56` OK. `1.500,00` (DE) →
    //   INCOHÉRENT car `.` avant `,`.
    // Sans ce check, `parse_amount("1,500.00", ',')` produirait
    // silencieusement 1.5 au lieu de rejeter (bug financier critique).
    let last_dot = s.rfind('.');
    let last_comma = s.rfind(',');
    if let (Some(dot_pos), Some(comma_pos)) = (last_dot, last_comma) {
        match decimal_sep {
            ',' if comma_pos < dot_pos => return Err(()), // ambigu (US dans profil DE)
            '.' if dot_pos < comma_pos => return Err(()), // ambigu (DE dans profil US)
            _ => {}
        }
    }

    // Selon le decimal_sep, strip aussi le séparateur milliers
    // complémentaire et normaliser en `.`.
    let s = match decimal_sep {
        ',' => {
            // Format allemand : `1.234,56` → strip `.`, replace `,` → `.`
            s.replace('.', "").replace(',', ".")
        }
        '.' => {
            // Format US : `1,234.56` → strip `,`
            s.replace(',', "")
        }
        _ => s, // Validation déjà faite dans CsvProfile::validate
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

    // ====================================================================
    // Pass 1 review G1 patches : tests manquants
    // ====================================================================

    /// Pass 2 review M1 (EH2-1) : `parse_amount` doit rejeter les
    /// formats mélangés ambigus pour éviter la corruption financière
    /// silencieuse. `"1,500.00"` (US) avec `decimal_sep=','` (DE)
    /// produirait silencieusement 1.5 sans ce check.
    #[test]
    fn parse_amount_rejects_ambiguous_us_format_with_de_profile() {
        // Profile DE (decimal_sep=','), value en format US (1,234.56).
        assert!(parse_amount("1,500.00", ',').is_err());
        assert!(parse_amount("1,234.56", ',').is_err());
        assert!(parse_amount("12,345.67", ',').is_err());
    }

    /// Pass 2 review M1 (EH2-1) symétrique : `"1.500,00"` (DE) avec
    /// `decimal_sep='.'` (US) → rejet.
    #[test]
    fn parse_amount_rejects_ambiguous_de_format_with_us_profile() {
        assert!(parse_amount("1.500,00", '.').is_err());
        assert!(parse_amount("1.234,56", '.').is_err());
    }

    /// Pass 1 review G1 H1 : `parse_amount` doit accepter le format
    /// allemand `1.234,56` quand `decimal_sep = ','`. Le point est
    /// strip comme séparateur milliers.
    #[test]
    fn parses_german_format_amount_with_dot_thousands_and_comma_decimal() {
        let csv = "date;montant\n2026-01-15;1.234,56\n";
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

    /// Pass 1 review G1 H1 : `parse_amount` doit accepter le format US
    /// `1,234.56` quand `decimal_sep = '.'`.
    #[test]
    fn parses_us_format_amount_with_comma_thousands_and_dot_decimal() {
        let csv = "date,montant\n2026-01-15,\"1,234.56\"\n";
        let profile = make_profile(
            ',',
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
        assert_eq!(stmt.transactions[0].amount, dec!(1234.56));
    }

    /// Pass 1 review G1 BH-9 : profil DB corrompu avec à la fois `amount`
    /// ET `debit_credit_split` non-null. Doit prioriser `debit_credit_split`
    /// au parse (cf. spec §profile-model M16). Notons que `validate()`
    /// rejette ce cas, mais le parse défensif protège contre une DB
    /// corrompue ou un bypass de validation.
    #[test]
    fn parses_corrupt_profile_with_both_amount_and_split_prefers_split() {
        // Construction directe de CsvProfile sans appeler validate().
        // Simule un profil DB corrompu désérialisé.
        let profile = CsvProfile {
            bank_name: "Corrupt".to_string(),
            filename_pattern: None,
            column_mapping: ColumnMapping {
                date: 0,
                amount: Some(1),                  // shouldn't be used
                debit_credit_split: Some((1, 2)), // priority
                reference: None,
                details: None,
                counterparty: None,
            },
            date_format: "%Y-%m-%d".to_string(),
            decimal_separator: '.',
            field_separator: ';',
            encoding: None,
            header_row_count: 1,
        };
        // validate() rejette ce cas — vérifions :
        assert!(profile.validate().is_err());

        // Mais si on bypass validate (via accès direct à parse_row),
        // la priorité parse est sur debit_credit_split.
        // On teste via une version contournée : construire un parse_row
        // directement avec un record et le profil corrompu.
        let record = csv::StringRecord::from(vec!["2026-01-15", "100.00", ""]);
        let result = super::parse_row(&record, 1, &profile);
        // Avec debit=100, credit=empty → amount = -100.
        let tx = result.expect("parse_row corrupt profile prefers split");
        assert_eq!(tx.amount, dec!(-100));
    }

    /// Pass 1 review G1 AA-2 : line numbers absolus avec multi-header.
    /// `header_row_count = 2` + erreur sur 5e ligne de données →
    /// `line` retourné = 7 (1-based, position absolue dans le fichier).
    #[test]
    fn line_numbers_are_file_absolute_with_multi_header() {
        // 2 lignes de header + 4 lignes valides + 1 ligne invalide (date malformée)
        let csv = "header1;header2\nheader1b;header2b\n2026-01-01;100\n2026-01-02;200\n2026-01-03;300\n2026-01-04;400\nINVALID_DATE;500\n";
        let profile = make_profile(
            ';',
            '.',
            2,
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
                assert_eq!(errors.len(), 1);
                // Position absolue dans le fichier : ligne 7 (2 headers + 4 data + 1 invalid).
                assert_eq!(errors[0].line, 7);
                assert_eq!(errors[0].code, CsvLineErrorCode::InvalidDate);
            }
            other => panic!("expected PartialFailure, got {:?}", other),
        }
    }

    // ─────────────────────────────────────────────────────────────
    // Story 8-3 T4.3 — `parse_csv_collect` 6 tests
    // ─────────────────────────────────────────────────────────────

    fn simple_profile() -> CsvProfile {
        make_profile(
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
        )
    }

    #[test]
    fn parse_csv_collect_all_valid_returns_all_valid() {
        // T4.3#1 — happy path : 2 lignes valides → AllValid.
        let csv =
            "date;amount;ref;details\n2026-01-15;100.00;R1;Loyer\n2026-01-16;-50.00;R2;Achat\n";
        let outcome = parse_csv_collect(csv.as_bytes(), &simple_profile());
        match outcome {
            ParseCsvOutcome::AllValid(stmt) => {
                assert_eq!(stmt.transactions.len(), 2);
                assert_eq!(stmt.transactions[0].amount, dec!(100.00));
            }
            other => panic!("expected AllValid, got {:?}", other),
        }
    }

    #[test]
    fn parse_csv_collect_partial_returns_valid_and_errors() {
        // T4.3#2 — fixture-like : 3 valides + 2 invalides (date + amount).
        let csv = "date;amount;ref;details\n\
                   2026-01-15;100.00;R1;A\n\
                   INVALID_DATE;200.00;R2;B\n\
                   2026-01-17;NOT_A_NUMBER;R3;C\n\
                   2026-01-18;300.00;R4;D\n\
                   2026-01-19;400.00;R5;E\n";
        let outcome = parse_csv_collect(csv.as_bytes(), &simple_profile());
        match outcome {
            ParseCsvOutcome::PartialFailure {
                valid,
                errors,
                total_errors,
                truncated,
            } => {
                assert_eq!(valid.transactions.len(), 3);
                assert_eq!(errors.len(), 2);
                assert_eq!(total_errors, 2);
                assert!(!truncated);
                // Vérifie codes d'erreur attendus.
                let codes: Vec<_> = errors.iter().map(|e| e.code).collect();
                assert!(codes.contains(&CsvLineErrorCode::InvalidDate));
                assert!(codes.contains(&CsvLineErrorCode::InvalidAmount));
            }
            other => panic!("expected PartialFailure, got {:?}", other),
        }
    }

    #[test]
    fn parse_csv_collect_caps_errors_at_max() {
        // T4.3#3 — 50 valides + 150 invalides → errors=100, total_errors=150,
        // truncated=true, valid=50.
        let mut csv = String::from("date;amount;ref;details\n");
        for i in 0..50 {
            csv.push_str(&format!("2026-01-15;{}.00;R{};V\n", i + 1, i));
        }
        for i in 0..150 {
            csv.push_str(&format!("INVALID_DATE_{};100.00;R;X\n", i));
        }
        let outcome = parse_csv_collect(csv.as_bytes(), &simple_profile());
        match outcome {
            ParseCsvOutcome::PartialFailure {
                valid,
                errors,
                total_errors,
                truncated,
            } => {
                assert_eq!(valid.transactions.len(), 50);
                assert_eq!(errors.len(), 100, "cap MAX_CSV_LINE_ERRORS=100");
                assert_eq!(total_errors, 150);
                assert!(truncated, "truncated flag doit être true");
            }
            other => panic!("expected PartialFailure, got {:?}", other),
        }
    }

    #[test]
    fn parse_csv_collect_zero_valid_returns_partial_with_empty_valid() {
        // T4.3#4 — 0 valides + 3 invalides → PartialFailure { valid empty }.
        let csv = "date;amount;ref;details\n\
                   INVALID_A;100;R1;A\n\
                   INVALID_B;200;R2;B\n\
                   INVALID_C;300;R3;C\n";
        let outcome = parse_csv_collect(csv.as_bytes(), &simple_profile());
        match outcome {
            ParseCsvOutcome::PartialFailure {
                valid,
                errors,
                total_errors,
                truncated,
            } => {
                assert!(valid.transactions.is_empty(), "0 valides attendu");
                assert_eq!(errors.len(), 3);
                assert_eq!(total_errors, 3);
                assert!(!truncated);
            }
            other => panic!("expected PartialFailure with empty valid, got {:?}", other),
        }
    }

    #[test]
    fn parse_csv_collect_zero_valid_uses_sentinel_date() {
        // M9 (Pass 1 review) — quand `transactions.is_empty()` après parse,
        // `period_from`/`period_to` doivent être la sentinel publique
        // [`empty_valid_sentinel_date`] (= 1970-01-01) pour permettre au
        // caller de comparer explicitement avant un find_in_dedup_window.
        let csv = "date;amount;ref;details\n\
                   INVALID_A;100;R1;A\n";
        let outcome = parse_csv_collect(csv.as_bytes(), &simple_profile());
        match outcome {
            ParseCsvOutcome::PartialFailure { valid, .. } => {
                let sentinel = empty_valid_sentinel_date();
                assert_eq!(valid.transactions.len(), 0);
                assert_eq!(valid.period_from, sentinel);
                assert_eq!(valid.period_to, sentinel);
                assert_eq!(sentinel, NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
            }
            other => panic!("expected PartialFailure, got {:?}", other),
        }
    }

    #[test]
    fn parse_csv_collect_hard_failure_on_empty_file() {
        // T4.3#5 — header seul (0 data rows) → HardFailure(EmptyFile).
        let csv = "date;amount;ref;details\n";
        let outcome = parse_csv_collect(csv.as_bytes(), &simple_profile());
        match outcome {
            ParseCsvOutcome::HardFailure(CsvError::EmptyFile { .. }) => {}
            other => panic!("expected HardFailure(EmptyFile), got {:?}", other),
        }
    }

    #[test]
    fn parse_csv_wrapper_preserves_legacy_behavior() {
        // T4.3#6 — `parse_csv` (signature 8-2) doit retourner Err sur
        // les partial failures, sans exposer le champ `valid`.
        let csv = "date;amount;ref;details\n\
                   2026-01-15;100.00;R1;A\n\
                   INVALID_DATE;200;R2;B\n";
        let err = parse_csv(csv.as_bytes(), &simple_profile()).unwrap_err();
        match err {
            CsvError::PartialFailure {
                errors,
                total_errors,
                truncated,
            } => {
                assert_eq!(errors.len(), 1);
                assert_eq!(total_errors, 1);
                assert!(!truncated);
            }
            other => panic!("expected PartialFailure (legacy), got {:?}", other),
        }
    }
}
