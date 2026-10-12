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
//!
//!    **Lignes à ignorer** (Story 26-1, #621) : `header_row_count` compte
//!    les lignes **non vides** avant les données, ligne de titres
//!    comprise — une ligne vide (aucun caractère) est avalée par le
//!    lecteur `csv` et n'entre pas dans le compte (relevé UBS : 9).
//!    **Garde de surcomptage** : juste après le saut, avant de lire le
//!    premier enregistrement de données, la dernière ligne ignorée est
//!    examinée ; si sa cellule date se lit au format du profil, le parseur
//!    refuse par [`CsvError::HeaderSkipsData`] (une transaction aurait
//!    disparu). La garde ne voit qu'une ligne **datée**.
//! 5. Itère sur tous les records (incluant le 1er) : pour chaque
//!    ligne, mappe les cellules via `column_mapping` → soit
//!    `ImportedTransaction` valide, soit `CsvLineError` collecté.
//!    En mode débit/crédit séparés, le **sens des débits** est le réglage
//!    [`DebitSign`] du profil, jamais une déduction : `Positive` → montant
//!    `-débit`, `Negative` → montant `débit` ; un montant de signe
//!    contraire à sa colonne est une erreur `AMOUNT_SIGN_MISMATCH`.
//!    **Devise** (Story 27-2, #624) : si le profil règle la colonne Devise
//!    (`ColumnMapping::currency`), sa cellule est lue **après** le montant —
//!    vide → `EMPTY_MANDATORY_FIELD` (`currency`), autre chose que trois
//!    lettres ASCII → `INVALID_CURRENCY`, casse normalisée (`chf` → `CHF`) ;
//!    sinon, la devise est [`DEFAULT_CSV_CURRENCY`], supposée et nommée.
//! 6. Cap collection à `MAX_CSV_LINE_ERRORS = 100` (Pass 2 H'1)
//!    avec `total_errors` + `truncated` flag. Les erreurs de signe ont
//!    leur **propre** compteur et leur propre vecteur, indépendants de ce
//!    plafond général.
//! 7. Si `transactions.is_empty()` → `CsvError::EmptyFile`
//!    (Pass 2 M'2 header-only file).
//! 8. Si au moins une erreur de signe → `CsvError::AmountSignMismatch`
//!    (refus entier, **non confirmable** — Story 26-1) ; sinon, si la
//!    colonne Devise est réglée et que les lignes **valides** portent plus
//!    d'une devise → `CsvError::MixedCurrencies` (refus entier, non
//!    confirmable — Story 27-2) ; sinon, si `errors` non-vide →
//!    `CsvError::PartialFailure { ... }` (strict reject FR51 v0.1, ou
//!    import des valides sur confirmation).
//! 9. Sinon retourne `ImportedStatement` (devise : celle, unique, des
//!    lignes valides, ou [`DEFAULT_CSV_CURRENCY`]) avec
//!    `period_from = min(booking_dates)` et
//!    `period_to = max(booking_dates)` (Pass 1 H2).

use crate::csv::encoding::{decode_bytes, detect_encoding};
use crate::csv::profile::{CsvProfile, DebitSign};
use crate::error::{CsvError, CsvLineError, CsvLineErrorCode, MAX_CSV_LINE_ERRORS};
use crate::types::{ImportedStatement, ImportedTransaction, SourceFormat};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::BTreeSet;

/// Devise supposée d'un relevé dont le profil ne lit aucune colonne Devise
/// (Story 27-2, #624). C'est la **seule** supposition de devise du
/// parseur CSV, et l'API la dit à l'aperçu (`bank_csv_currency_assumed`) ;
/// une cellule Devise vide n'est jamais remplacée par elle.
pub const DEFAULT_CSV_CURRENCY: &str = "CHF";

/// Nombre maximal de devises rapportées par
/// [`CsvError::MixedCurrencies`] : les premières de l'ensemble **trié**
/// des devises distinctes (Story 27-2, C-27-2-13). Un relevé de compte qui
/// en porterait davantage n'en est pas un ; le message reste vrai tronqué.
pub const MAX_REPORTED_CURRENCIES: usize = 10;

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
/// **F5 (Pass 3 review)** : utilise `NaiveDate::MIN` (an -262143) plutôt
/// que `1970-01-01` (initial M9 Pass 1). Justification : `1970-01-01`
/// est une date réelle qui pourrait apparaître dans un import légitime
/// (système-default, dataset de test/dev). Si une telle transaction
/// arrive avec une autre clé composite à dédupliquer dans la même
/// fenêtre, la garde runtime L-1 court-circuiterait `find_in_dedup_window`
/// et **bypasserait silencieusement** la dedup pour cette transaction.
/// `NaiveDate::MIN` ne peut jamais correspondre à une transaction
/// bancaire réelle → la sentinel reste défensive sans risque de
/// collision avec des données légitimes. Le runtime guard L-1
/// (côté kesh-api) couvre déjà le risque de scan complet.
pub fn empty_valid_sentinel_date() -> NaiveDate {
    NaiveDate::MIN
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
    if let Some(ref profile_encoding) = profile.encoding
        && profile_encoding != detected_encoding.as_str()
    {
        return Err(CsvError::EncodingMismatch {
            profile: profile_encoding.clone(),
            detected: detected_encoding.as_str().to_string(),
        });
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

    // 5. Dernière ligne ignorée, pour la garde de surcomptage (Story 26-1,
    //    AC5). Avec `header_row_count == 1`, c'est l'enregistrement
    //    d'en-têtes, que `records()` ne rend pas : le lire ici, AVANT
    //    d'emprunter le lecteur pour itérer. Sa position est celle qu'il
    //    porte lui-même (`headers()?.position()`), pas `reader.position()`
    //    après lecture, qui vaut déjà la ligne suivante.
    let mut last_skipped: Option<csv::StringRecord> = None;
    if profile.header_row_count == 1 {
        let headers = reader
            .headers()
            .map_err(|e| CsvError::Io(format!("header row : {}", e)))?;
        last_skipped = Some(headers.clone());
    }

    // 5.b Skip extra header rows (Pass 1 H4 saturating_sub anti-underflow)
    let extra_skip = profile.header_row_count.saturating_sub(1) as usize;
    let mut iter = reader.records();
    for _ in 0..extra_skip {
        match iter.next() {
            Some(Ok(rec)) => last_skipped = Some(rec),
            Some(Err(e)) => return Err(CsvError::Io(format!("skip header row : {}", e))),
            None => {
                // Pas assez de lignes pour le header → MissingHeader.
                return Err(CsvError::MissingHeader);
            }
        }
    }

    // 5.c Garde de surcomptage — juste après le saut, AVANT de lire le
    //     premier enregistrement de données : sur un fichier dont la seule
    //     transaction est avalée, `EmptyFile` répondrait sinon avant elle.
    if let Some(ref rec) = last_skipped {
        guard_header_skips_data(rec, profile)?;
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
    let mut acc = RowAccumulator::default();

    let first_line = first_record.position().map(|p| p.line()).unwrap_or(0) as usize;
    acc.push(parse_row(&first_record, first_line, profile));

    for result in iter {
        match result {
            Ok(record) => {
                let line = record.position().map(|p| p.line()).unwrap_or(0) as usize;
                acc.push(parse_row(&record, line, profile));
            }
            Err(e) => {
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
                acc.push_general(CsvLineError::new(
                    line,
                    CsvLineErrorCode::RowTooShort,
                    Some(e.to_string()),
                ));
            }
        }
    }

    // 7.b Refus de signe (Story 26-1, #620) : décidé sur le compteur PROPRE
    //     au signe, jamais sur le vecteur général écrêté (qui peut avoir été
    //     rempli par 100 erreurs d'une autre nature avant la première de
    //     signe). Il l'emporte sur les autres erreurs de ligne, non
    //     rapportées : régler le profil d'abord.
    if acc.sign_total > 0 {
        return Err(CsvError::AmountSignMismatch {
            truncated: acc.sign_total > MAX_CSV_LINE_ERRORS,
            whole_debit_column: acc.whole_debit_column(),
            errors: acc.sign_errors,
            total_errors: acc.sign_total,
        });
    }
    let RowAccumulator {
        transactions,
        errors,
        total_errors,
        ..
    } = acc;

    // 7.c Un relevé, une devise (Story 27-2, #624) — APRÈS le refus de
    //     signe, qui reste prioritaire, et AVANT le rejet partiel : refus
    //     non confirmable, même s'il y a aussi des erreurs de ligne.
    //     ⚠️ Seules les lignes VALIDES révèlent leur devise : une ligne
    //     écartée pour sa date ou son montant n'est pas lue jusqu'à sa
    //     devise (C-27-2-14) — un fichier CHF dont les lignes EUR sont
    //     toutes invalides pour une autre raison n'est donc pas refusé ici.
    let statement_currency = if profile.column_mapping.currency.is_some() {
        let distinct: BTreeSet<&str> = transactions.iter().map(|t| t.currency.as_str()).collect();
        if distinct.len() > 1 {
            return Err(CsvError::MixedCurrencies {
                currencies: distinct
                    .into_iter()
                    .take(MAX_REPORTED_CURRENCIES)
                    .map(String::from)
                    .collect(),
            });
        }
        // Aucune ligne valide : supposée (l'import est de toute façon
        // refusé, `no_valid_lines_to_commit`).
        distinct
            .into_iter()
            .next()
            .unwrap_or(DEFAULT_CSV_CURRENCY)
            .to_string()
    } else {
        DEFAULT_CSV_CURRENCY.to_string()
    };

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

    // Invariant : chaque transaction valide porte `statement_currency`
    // (lue unique ci-dessus, ou la même constante pour chaque ligne quand
    // la colonne n'est pas réglée). Vérifié par les tests AC13-1, -2, -5 et
    // -9, qui assertent chaque `tx.currency` — pas de `debug_assert!`,
    // absent en release.
    let stmt = ImportedStatement {
        source_format: SourceFormat::Csv {
            encoding: detected_encoding.as_str().to_string(),
            profile_name: Some(profile.bank_name.clone()),
        },
        statement_id: None,
        account_iban: String::new(),
        currency: statement_currency,
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

/// Garde de surcomptage (Story 26-1, AC5) : refuse si la dernière ligne
/// ignorée porte, dans la colonne date, une valeur qui se lit au format
/// du profil. Une ligne trop courte pour porter la colonne date, ou dont
/// la date est vide ou illisible, passe (angle mort assumé, cf.
/// [`CsvError::HeaderSkipsData`]).
fn guard_header_skips_data(
    record: &csv::StringRecord,
    profile: &CsvProfile,
) -> Result<(), CsvError> {
    let Some(cell) = record.get(profile.column_mapping.date) else {
        return Ok(());
    };
    if NaiveDate::parse_from_str(cell.trim(), &profile.date_format).is_ok() {
        let line = record.position().map(|p| p.line()).unwrap_or(0) as usize;
        return Err(CsvError::HeaderSkipsData { line });
    }
    Ok(())
}

/// Ce que `parse_row` a constaté sur la colonne **Débit** d'une ligne
/// (Story 26-1, C-26-1-25) — sert à calculer `whole_debit_column` sans
/// rien ajouter au type public [`CsvLineError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DebitObservation {
    /// Aucun débit lu : colonne unique, ligne de crédit, échec avant le
    /// montant (date, ligne trop courte, ambiguïté, cellules vides) ou
    /// débit illisible.
    NotRead,
    /// Débit lu, égal à zéro (`0.00`, `-0.00`) : hors des deux comptes.
    Zero,
    /// Débit lu, non nul, de signe conforme au réglage.
    Consistent,
    /// Débit lu, non nul, de signe contraire au réglage.
    SignMismatch,
}

/// Issue de [`parse_row`] : la ligne (transaction ou erreur) et
/// l'observation sur sa colonne Débit.
#[derive(Debug)]
struct RowParse {
    result: Result<ImportedTransaction, CsvLineError>,
    debit: DebitObservation,
}

impl RowParse {
    fn err(e: CsvLineError) -> Self {
        Self {
            result: Err(e),
            debit: DebitObservation::NotRead,
        }
    }
}

/// Accumulateur de la boucle des lignes : transactions valides, erreurs
/// générales plafonnées, erreurs de signe (compteur et vecteur propres,
/// Story 26-1, R2-1) et comptes `d`/`e` de `whole_debit_column`.
#[derive(Default)]
struct RowAccumulator {
    transactions: Vec<ImportedTransaction>,
    errors: Vec<CsvLineError>,
    total_errors: usize,
    sign_errors: Vec<CsvLineError>,
    sign_total: usize,
    /// `d` : débits lus non nuls.
    debits_read: usize,
    /// `e` : débits lus non nuls, en défaut de signe.
    debits_mismatched: usize,
}

impl RowAccumulator {
    fn push(&mut self, row: RowParse) {
        match row.debit {
            DebitObservation::Consistent => self.debits_read += 1,
            DebitObservation::SignMismatch => {
                self.debits_read += 1;
                self.debits_mismatched += 1;
            }
            DebitObservation::NotRead | DebitObservation::Zero => {}
        }
        match row.result {
            Ok(tx) => self.transactions.push(tx),
            Err(e) if e.code == CsvLineErrorCode::AmountSignMismatch => {
                // Compteur et vecteur propres, alimentés quel que soit
                // l'état du plafond général.
                self.sign_total += 1;
                if self.sign_errors.len() < MAX_CSV_LINE_ERRORS {
                    self.sign_errors.push(e);
                }
            }
            Err(e) => self.push_general(e),
        }
    }

    fn push_general(&mut self, e: CsvLineError) {
        self.total_errors += 1;
        if self.errors.len() < MAX_CSV_LINE_ERRORS {
            self.errors.push(e);
        }
    }

    /// `whole_debit_column ⇔ d ≥ 2 et e = d` (C-26-1-25) : jamais vrai
    /// pour un seul débit, qui peut être un remboursement — le conseil de
    /// changer le réglage l'importerait comme une dépense, en silence.
    fn whole_debit_column(&self) -> bool {
        self.debits_read >= 2 && self.debits_mismatched == self.debits_read
    }
}

/// Parse une ligne CSV en `ImportedTransaction` ou `CsvLineError`, et
/// rapporte ce qu'elle a constaté sur la colonne Débit.
///
/// Ordre des contrôles : longueur de la ligne, **date**, montant, puis
/// **devise** (Story 27-2) — une date illisible rend `INVALID_DATE` et le
/// signe de la ligne n'est pas examiné ; un montant illisible ou de signe
/// contraire masque la devise de la ligne. Une erreur de devise garde
/// l'observation de débit faite au stade du montant (jamais
/// [`RowParse::err`]), sans quoi un débit cohérent sortirait du
/// dénominateur de `whole_debit_column`.
///
/// Montant en mode débit/crédit séparés (Story 26-1, #620) :
///
/// | Réglage `debit_sign` | Colonne Débit `d` | Colonne Crédit `c` |
/// |---|---|---|
/// | `Positive` (défaut) | `-d` ; `d < 0` → `AMOUNT_SIGN_MISMATCH` | `c` ; `c < 0` → `AMOUNT_SIGN_MISMATCH` |
/// | `Negative` | `d` ; `d > 0` → `AMOUNT_SIGN_MISMATCH` | idem |
///
/// Zéro (`0.00`, `-0.00`) est accepté partout : la comparaison est
/// stricte à `Decimal::ZERO`, jamais sur le bit de signe. En colonne
/// unique (`amount`), le signe du fichier est gardé tel quel.
fn parse_row(record: &csv::StringRecord, line: usize, profile: &CsvProfile) -> RowParse {
    let cm = &profile.column_mapping;

    // Helper : récupère cellule trim ou None si Optional + empty.
    let get_cell = |idx: usize| -> Option<&str> { record.get(idx).map(|s| s.trim()) };

    // RowTooShort : si le 1er record passait, les autres pourraient être
    // plus courts (csv flexible mode). Vérifier une dernière fois.
    let max_idx = profile.column_mapping.max_index();
    if record.len() <= max_idx {
        return RowParse::err(CsvLineError::new(line, CsvLineErrorCode::RowTooShort, None));
    }

    // Date (obligatoire)
    let date_raw = match get_cell(cm.date) {
        Some(s) if !s.is_empty() => s,
        _ => {
            return RowParse::err(CsvLineError::new(
                line,
                CsvLineErrorCode::EmptyMandatoryField,
                Some("date".to_string()),
            ));
        }
    };
    let booking_date = match NaiveDate::parse_from_str(date_raw, &profile.date_format) {
        Ok(d) => d,
        Err(_) => {
            return RowParse::err(CsvLineError::new(
                line,
                CsvLineErrorCode::InvalidDate,
                Some(date_raw.to_string()),
            ));
        }
    };

    let sign_mismatch = |raw: &str| {
        CsvLineError::new(
            line,
            CsvLineErrorCode::AmountSignMismatch,
            Some(raw.to_string()),
        )
    };
    let invalid_amount =
        |raw: &str| CsvLineError::new(line, CsvLineErrorCode::InvalidAmount, Some(raw.to_string()));

    // Amount : XOR amount vs debit_credit_split. Priorité parse au
    // debit_credit_split (Pass 1 M16 — DB corrompue protection).
    let mut debit = DebitObservation::NotRead;
    let amount = if let Some((debit_idx, credit_idx)) = cm.debit_credit_split {
        let debit_raw = get_cell(debit_idx).unwrap_or("");
        let credit_raw = get_cell(credit_idx).unwrap_or("");
        let debit_empty = debit_raw.is_empty();
        let credit_empty = credit_raw.is_empty();
        match (debit_empty, credit_empty) {
            (true, true) => {
                return RowParse::err(CsvLineError::new(
                    line,
                    CsvLineErrorCode::EmptyMandatoryField,
                    Some("amount (debit+credit empty)".to_string()),
                ));
            }
            (false, false) => {
                return RowParse::err(CsvLineError::new(
                    line,
                    CsvLineErrorCode::AmbiguousDebitCredit,
                    None,
                ));
            }
            (false, true) => {
                // Débit : le sens est le réglage du profil, jamais une
                // déduction (décision du Project Lead, #620).
                let Ok(parsed) = parse_amount(debit_raw, profile.decimal_separator) else {
                    return RowParse::err(invalid_amount(debit_raw));
                };
                let mismatched = match profile.debit_sign {
                    DebitSign::Positive => parsed < Decimal::ZERO,
                    DebitSign::Negative => parsed > Decimal::ZERO,
                };
                if mismatched {
                    return RowParse {
                        result: Err(sign_mismatch(debit_raw)),
                        debit: DebitObservation::SignMismatch,
                    };
                }
                debit = if parsed == Decimal::ZERO {
                    DebitObservation::Zero
                } else {
                    DebitObservation::Consistent
                };
                if parsed == Decimal::ZERO {
                    // `-parsed` sur un zéro porterait le bit de signe (`Neg`
                    // de `rust_decimal`) et s'afficherait « -0.00 » : un
                    // débit nul rend le zéro sans signe (revue P1, E-4).
                    // `abs()` efface le bit de signe en gardant l'échelle
                    // lue — `Decimal::ZERO`, d'échelle 0, s'afficherait
                    // « 0 » à l'aperçu au lieu de « 0.00 » (revue P2, E2-2).
                    parsed.abs()
                } else {
                    match profile.debit_sign {
                        DebitSign::Positive => -parsed,
                        DebitSign::Negative => parsed,
                    }
                }
            }
            (true, false) => {
                // Crédit : positif, quel que soit le réglage.
                let Ok(parsed) = parse_amount(credit_raw, profile.decimal_separator) else {
                    return RowParse::err(invalid_amount(credit_raw));
                };
                if parsed < Decimal::ZERO {
                    return RowParse::err(sign_mismatch(credit_raw));
                }
                parsed
            }
        }
    } else if let Some(amount_idx) = cm.amount {
        let amount_raw = match get_cell(amount_idx) {
            Some(s) if !s.is_empty() => s,
            _ => {
                return RowParse::err(CsvLineError::new(
                    line,
                    CsvLineErrorCode::EmptyMandatoryField,
                    Some("amount".to_string()),
                ));
            }
        };
        match parse_amount(amount_raw, profile.decimal_separator) {
            Ok(a) => a,
            Err(()) => return RowParse::err(invalid_amount(amount_raw)),
        }
    } else {
        // validate() bloque déjà ce cas, mais defense in depth
        return RowParse::err(CsvLineError::new(
            line,
            CsvLineErrorCode::InvalidAmount,
            Some("profil invalide : ni amount ni debit_credit_split".to_string()),
        ));
    };

    // Devise (Story 27-2, #624) : lue APRÈS le montant. ⛔ Une erreur de
    // devise rend `debit` tel qu'observé au stade du montant, jamais
    // `RowParse::err` (qui le remettrait à `NotRead`).
    let currency = match cm.currency {
        None => DEFAULT_CSV_CURRENCY.to_string(),
        Some(idx) => match get_cell(idx) {
            Some(s) if !s.is_empty() => match normalize_currency(s) {
                Some(c) => c,
                None => {
                    // Valeur rapportée : la cellule **rognée** par `get_cell`
                    // (revue de code P1, A-4), tronquée comme toute valeur.
                    return RowParse {
                        result: Err(CsvLineError::new(
                            line,
                            CsvLineErrorCode::InvalidCurrency,
                            Some(s.to_string()),
                        )),
                        debit,
                    };
                }
            },
            _ => {
                return RowParse {
                    result: Err(CsvLineError::new(
                        line,
                        CsvLineErrorCode::EmptyMandatoryField,
                        Some("currency".to_string()),
                    )),
                    debit,
                };
            }
        },
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

    RowParse {
        result: Ok(ImportedTransaction {
            booking_date,
            value_date: Some(booking_date),
            amount,
            currency,
            reference,
            details,
            end_to_end_id: None,
            transaction_id: None,
            counterparty_iban: None,
            counterparty_name,
        }),
        debit,
    }
}

/// Forme d'une cellule Devise (Story 27-2, C-27-2-5) : exactement trois
/// lettres **ASCII** (`s.len() == 3` compte des octets, et trois octets
/// ASCII font trois caractères), mises en majuscules. Pas de contrôle
/// contre la liste ISO 4217 : la forme suffit au parseur, le périmètre
/// des devises acceptées est décidé hors de `kesh-import`.
fn normalize_currency(s: &str) -> Option<String> {
    if s.len() == 3 && s.bytes().all(|b| b.is_ascii_alphabetic()) {
        Some(s.to_ascii_uppercase())
    } else {
        None
    }
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
            debit_sign: DebitSign::Positive,
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
                currency: None,
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
                currency: None,
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
                currency: None,
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
                currency: None,
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
                currency: None,
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
                currency: None,
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
                currency: None,
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
                currency: None,
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
                currency: None,
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
                currency: None,
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
                currency: None,
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
                currency: None,
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
                currency: None,
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
                currency: None,
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
                currency: None,
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
                currency: None,
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
                currency: None,
            },
            date_format: "%Y-%m-%d".to_string(),
            decimal_separator: '.',
            field_separator: ';',
            encoding: None,
            header_row_count: 1,
            debit_sign: DebitSign::Positive,
        };
        // validate() rejette ce cas — vérifions :
        assert!(profile.validate().is_err());

        // Mais si on bypass validate (via accès direct à parse_row),
        // la priorité parse est sur debit_credit_split.
        // On teste via une version contournée : construire un parse_row
        // directement avec un record et le profil corrompu.
        let record = csv::StringRecord::from(vec!["2026-01-15", "100.00", ""]);
        let result = super::parse_row(&record, 1, &profile).result;
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
                currency: None,
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
                currency: None,
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
        // M9 (Pass 1) + F5 (Pass 3) — quand `transactions.is_empty()` après
        // parse, `period_from`/`period_to` doivent être la sentinel publique
        // [`empty_valid_sentinel_date`] (= `NaiveDate::MIN`, jamais une date
        // bancaire réelle) pour permettre au caller de comparer explicitement
        // avant un `find_in_dedup_window` sans risque de collision avec des
        // données légitimes (ex. transaction datée 1970-01-01).
        let csv = "date;amount;ref;details\n\
                   INVALID_A;100;R1;A\n";
        let outcome = parse_csv_collect(csv.as_bytes(), &simple_profile());
        match outcome {
            ParseCsvOutcome::PartialFailure { valid, .. } => {
                let sentinel = empty_valid_sentinel_date();
                assert_eq!(valid.transactions.len(), 0);
                assert_eq!(valid.period_from, sentinel);
                assert_eq!(valid.period_to, sentinel);
                assert_eq!(sentinel, NaiveDate::MIN);
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

    // ─────────────────────────────────────────────────────────────
    // Story 26-1 — sens des débits (#620), lignes à ignorer (#621)
    // ─────────────────────────────────────────────────────────────

    /// Fixture synthétique de structure UBS (cf. `tests/fixtures/README.md`).
    const UBS_FIXTURE: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/csv/ubs_synthetique.csv"
    ));

    /// Profil UBS recommandé (C-26-1-13) : date de comptabilisation (2),
    /// débit/crédit (5, 6), référence (9), détails (10), 9 lignes ignorées.
    fn ubs_profile(sign: DebitSign, header_rows: u8) -> CsvProfile {
        let mut p = make_profile(
            ';',
            '.',
            header_rows,
            ColumnMapping {
                date: 2,
                amount: None,
                debit_credit_split: Some((5, 6)),
                reference: Some(9),
                details: Some(10),
                counterparty: None,
                currency: None,
            },
        );
        p.debit_sign = sign;
        p
    }

    /// Profil `date;debit;credit` à une ligne de titres.
    fn split_profile(sign: DebitSign) -> CsvProfile {
        let mut p = make_profile(
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
                currency: None,
            },
        );
        p.debit_sign = sign;
        p
    }

    /// Valeur d'une clé du préambule de la fixture (`Clé:;valeur;`).
    fn preamble_value(key: &str) -> Decimal {
        let text = std::str::from_utf8(&UBS_FIXTURE[3..]).unwrap();
        let line = text
            .lines()
            .find(|l| l.starts_with(key))
            .unwrap_or_else(|| panic!("clé {key} absente du préambule"));
        line.split(';').nth(1).unwrap().parse().unwrap()
    }

    /// Déstructure un refus de signe.
    fn sign_refusal(outcome: ParseCsvOutcome) -> (Vec<CsvLineError>, usize, bool, bool) {
        match outcome {
            ParseCsvOutcome::HardFailure(CsvError::AmountSignMismatch {
                errors,
                total_errors,
                truncated,
                whole_debit_column,
            }) => (errors, total_errors, truncated, whole_debit_column),
            other => panic!("expected HardFailure(AmountSignMismatch), got {other:?}"),
        }
    }

    /// AC14-1 : la fixture sous `Negative` → 11 transactions, 9 sorties,
    /// 2 entrées, somme = solde final − solde initial **lus dans le
    /// préambule** par le test.
    #[test]
    fn ubs_fixture_negative_imports_signs_and_sum() {
        let outcome = parse_csv_collect(UBS_FIXTURE, &ubs_profile(DebitSign::Negative, 9));
        let ParseCsvOutcome::AllValid(stmt) = outcome else {
            panic!("expected AllValid, got {outcome:?}");
        };
        assert_eq!(stmt.transactions.len(), 11);
        assert_eq!(
            stmt.transactions
                .iter()
                .filter(|t| t.amount < Decimal::ZERO)
                .count(),
            9
        );
        assert_eq!(
            stmt.transactions
                .iter()
                .filter(|t| t.amount > Decimal::ZERO)
                .count(),
            2
        );
        let sum: Decimal = stmt.transactions.iter().map(|t| t.amount).sum();
        assert_eq!(sum, dec!(-236.25));
        assert_eq!(
            sum,
            preamble_value("Solde final:") - preamble_value("Solde initial:")
        );
        assert_eq!(
            stmt.transactions.len(),
            preamble_value("Nombre de transactions")
                .to_string()
                .parse::<usize>()
                .unwrap()
        );
        // Référence et détails : colonnes 9 et 10.
        assert_eq!(
            stmt.transactions[0].reference.as_deref(),
            Some("9900111ZZ0000011")
        );
        assert_eq!(stmt.transactions[0].details, "Électricité Fictive SA");
    }

    /// AC14-2 : la période vient de la colonne 2 (date de comptabilisation).
    #[test]
    fn ubs_fixture_period_uses_booking_date_column() {
        let stmt = parse_csv(UBS_FIXTURE, &ubs_profile(DebitSign::Negative, 9)).unwrap();
        assert_eq!(
            stmt.period_from,
            NaiveDate::from_ymd_opt(2026, 1, 6).unwrap()
        );
        assert_eq!(stmt.period_to, NaiveDate::from_ymd_opt(2026, 3, 2).unwrap());
    }

    /// AC14-3 : la même fixture sous `Positive` → refus de signe entier,
    /// aucune transaction rendue (#620 ne passe plus, ni en silence ni par
    /// l'import partiel) ; `parse_csv` rend la même `Err`.
    #[test]
    fn ubs_fixture_positive_refuses_whole_file() {
        let (errors, total, truncated, whole) = sign_refusal(parse_csv_collect(
            UBS_FIXTURE,
            &ubs_profile(DebitSign::Positive, 9),
        ));
        assert_eq!(total, 9);
        assert!(!truncated);
        assert!(whole);
        assert_eq!(errors.len(), 9);
        assert!(
            errors
                .iter()
                .all(|e| e.code == CsvLineErrorCode::AmountSignMismatch)
        );
        assert_eq!(errors[0].line, 11);
        assert_eq!(errors[0].value.as_deref(), Some("-199.95"));
        assert!(matches!(
            parse_csv(UBS_FIXTURE, &ubs_profile(DebitSign::Positive, 9)),
            Err(CsvError::AmountSignMismatch {
                total_errors: 9,
                ..
            })
        ));
    }

    /// AC14-3 (suite) : une erreur de signe et une `INVALID_DATE` sur deux
    /// lignes → le signe domine ; l'autre erreur n'est pas rapportée.
    #[test]
    fn sign_mismatch_dominates_other_line_errors() {
        let csv = "date;debit;credit\nNOT_A_DATE;10.00;\n2026-01-02;-5.00;\n2026-01-03;7.00;\n";
        let (errors, total, _, _) = sign_refusal(parse_csv_collect(
            csv.as_bytes(),
            &split_profile(DebitSign::Positive),
        ));
        assert_eq!(total, 1);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].line, 3);
    }

    /// AC14-4 : `header_row_count = 8` → la ligne de titres arrive en
    /// données ; son erreur est rapportée à la **ligne 9** (la ligne vide
    /// qui la précède : le lecteur pose la position avant de l'avaler),
    /// non à sa ligne physique 10.
    #[test]
    fn ubs_fixture_header_8_reports_title_row_at_blank_line() {
        match parse_csv_collect(UBS_FIXTURE, &ubs_profile(DebitSign::Negative, 8)) {
            ParseCsvOutcome::PartialFailure { valid, errors, .. } => {
                assert_eq!(valid.transactions.len(), 11);
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].code, CsvLineErrorCode::InvalidDate);
                assert_eq!(errors[0].line, 9);
            }
            other => panic!("expected PartialFailure, got {other:?}"),
        }
    }

    /// AC14-4 (suite) : sans ligne vide devant, le numéro est la ligne
    /// physique exacte.
    #[test]
    fn line_number_is_physical_without_blank_line() {
        let csv = "a;b;c\nd;e;f\nTitre;Debit;Credit\n2026-01-01;-1.00;\nXX;-2.00;\n";
        let mut profile = split_profile(DebitSign::Negative);
        profile.header_row_count = 3;
        match parse_csv_collect(csv.as_bytes(), &profile) {
            ParseCsvOutcome::PartialFailure { errors, .. } => {
                assert_eq!(errors[0].code, CsvLineErrorCode::InvalidDate);
                assert_eq!(errors[0].line, 5);
            }
            other => panic!("expected PartialFailure, got {other:?}"),
        }
    }

    /// AC14-5 : `header_row_count = 10` → la première transaction serait
    /// avalée : refus nommant sa ligne (11, aucune ligne vide devant).
    #[test]
    fn ubs_fixture_header_10_refuses_header_skips_data() {
        assert_eq!(
            parse_csv_collect(UBS_FIXTURE, &ubs_profile(DebitSign::Negative, 10)),
            ParseCsvOutcome::HardFailure(CsvError::HeaderSkipsData { line: 11 })
        );
    }

    /// AC14-6 : garde muette quand la dernière ligne ignorée est un titre
    /// (9 lignes, en ligne) ; `header_row_count = 0` n'examine rien.
    #[test]
    fn header_guard_silent_on_title_row_and_zero() {
        let csv = "k1:;v;\nk2:;v;\nk3:;v;\nk4:;v;\nk5:;v;\nk6:;v;\nk7:;v;\nk8:;v;\n\
                   date;debit;credit\n2026-01-01;-1.00;\n";
        let mut profile = split_profile(DebitSign::Negative);
        profile.header_row_count = 9;
        assert!(matches!(
            parse_csv_collect(csv.as_bytes(), &profile),
            ParseCsvOutcome::AllValid(_)
        ));

        let csv0 = "2026-01-01;-1.00;\n2026-01-02;;3.00\n";
        profile.header_row_count = 0;
        match parse_csv_collect(csv0.as_bytes(), &profile) {
            ParseCsvOutcome::AllValid(stmt) => assert_eq!(stmt.transactions.len(), 2),
            other => panic!("expected AllValid, got {other:?}"),
        }
    }

    /// AC14-6 (F12, M20) : `header_row_count = 1` sur un CSV sans titres →
    /// refus nommant la **ligne 1** — lue par `headers()?.position()`, pas
    /// par `reader.position()` après lecture (qui vaudrait 2).
    #[test]
    fn header_guard_one_row_reports_line_1() {
        let csv = "2026-01-01;-1.00;\n2026-01-02;-2.00;\n";
        assert_eq!(
            parse_csv_collect(csv.as_bytes(), &split_profile(DebitSign::Negative)),
            ParseCsvOutcome::HardFailure(CsvError::HeaderSkipsData { line: 1 })
        );
    }

    /// AC14-6 (revue P1, E-1) : la garde passe **avant** la lecture du
    /// premier enregistrement de données. Sur un fichier dont la seule
    /// transaction est avalée par le saut, il ne reste aucune donnée :
    /// une garde placée après `iter.next()` laisserait répondre
    /// `EmptyFile`, qui ne dit pas que le nombre de lignes est trop grand.
    #[test]
    fn header_guard_runs_before_first_data_record() {
        // `header_row_count = 1` : l'unique ligne est l'en-tête lu par
        // `headers()`.
        let csv = "2026-01-01;-1.00;\n";
        assert_eq!(
            parse_csv_collect(csv.as_bytes(), &split_profile(DebitSign::Negative)),
            ParseCsvOutcome::HardFailure(CsvError::HeaderSkipsData { line: 1 })
        );
        // `header_row_count = 2` : titre, puis l'unique transaction avalée
        // par la boucle de saut.
        let mut profile = split_profile(DebitSign::Negative);
        profile.header_row_count = 2;
        let csv2 = "date;debit;credit\n2026-01-01;-1.00;\n";
        assert_eq!(
            parse_csv_collect(csv2.as_bytes(), &profile),
            ParseCsvOutcome::HardFailure(CsvError::HeaderSkipsData { line: 2 })
        );
    }

    /// AC5 (revue P2, E2-1) : la garde lit la cellule date **trimée**, au
    /// **format du profil** — pas au format ISO codé en dur. (a) Profil
    /// `%d.%m.%Y` : une transaction `05.01.2026` avalée par le saut est
    /// vue ; (b) profil ISO : une date entourée de blancs ` 2026-01-05 `
    /// est vue (`parse_row` la trime et l'aurait importée).
    #[test]
    fn header_guard_uses_profile_format_on_trimmed_cell() {
        let mut swiss = split_profile(DebitSign::Negative);
        swiss.date_format = "%d.%m.%Y".to_string();
        swiss.header_row_count = 2;
        let csv = "date;debit;credit\n05.01.2026;-1.00;\n06.01.2026;-2.00;\n";
        assert_eq!(
            parse_csv_collect(csv.as_bytes(), &swiss),
            ParseCsvOutcome::HardFailure(CsvError::HeaderSkipsData { line: 2 })
        );

        let mut iso = split_profile(DebitSign::Negative);
        iso.header_row_count = 2;
        let csv = "date;debit;credit\n 2026-01-05 ;-1.00;\n2026-01-06;-2.00;\n";
        assert_eq!(
            parse_csv_collect(csv.as_bytes(), &iso),
            ParseCsvOutcome::HardFailure(CsvError::HeaderSkipsData { line: 2 })
        );
    }

    /// AC14-6 (M13) : limite de longueur — la dernière ligne ignorée à
    /// exactement `date` champs (trop courte pour porter la colonne date)
    /// laisse la garde muette ; à `date + 1` champs et datée, refus.
    #[test]
    fn header_guard_length_boundary() {
        let mut profile = ubs_profile(DebitSign::Negative, 2);
        profile.column_mapping = ColumnMapping {
            date: 2,
            amount: None,
            debit_credit_split: Some((0, 1)),
            reference: None,
            details: None,
            counterparty: None,
            currency: None,
        };
        // Ligne 2 à 2 champs (= date) : muette, même si ses cellules sont datées.
        let short = "titre\n2026-01-01;2026-01-01\n-1.00;;2026-01-05\n";
        match parse_csv_collect(short.as_bytes(), &profile) {
            ParseCsvOutcome::AllValid(stmt) => assert_eq!(stmt.transactions.len(), 1),
            other => panic!("expected AllValid, got {other:?}"),
        }
        // Ligne 2 à 3 champs (= date + 1), datée : refus.
        let exact = "titre\nx;y;2026-01-01\n-1.00;;2026-01-05\n";
        assert_eq!(
            parse_csv_collect(exact.as_bytes(), &profile),
            ParseCsvOutcome::HardFailure(CsvError::HeaderSkipsData { line: 2 })
        );
    }

    /// AC14-6-bis — **angle mort** de la garde (R-12) : un nombre trop grand
    /// d'une unité qui avale une transaction à date **vide** ne produit
    /// aucune erreur ; la transaction manque. Ce test documente, il ne
    /// promet pas.
    #[test]
    fn header_guard_blind_spot_on_undated_row() {
        let csv = "date;debit;credit\n;-9.00;\n2026-01-02;-1.00;\n2026-01-03;;2.00\n";
        let mut profile = split_profile(DebitSign::Negative);
        profile.header_row_count = 2;
        match parse_csv_collect(csv.as_bytes(), &profile) {
            ParseCsvOutcome::AllValid(stmt) => assert_eq!(stmt.transactions.len(), 2),
            other => panic!("expected AllValid (angle mort), got {other:?}"),
        }
    }

    /// AC14-7 : les trois formes de signe contraire, sous les deux réglages.
    #[test]
    fn sign_checks_per_column_and_setting() {
        let cases: &[(DebitSign, &str, &str)] = &[
            (DebitSign::Positive, "2026-01-01;-5.00;\n", "-5.00"),
            (DebitSign::Negative, "2026-01-01;5.00;\n", "5.00"),
            (DebitSign::Positive, "2026-01-01;;-5.00\n", "-5.00"),
            (DebitSign::Negative, "2026-01-01;;-5.00\n", "-5.00"),
        ];
        for (sign, row, value) in cases {
            let csv = format!("date;debit;credit\n{row}");
            let (errors, total, _, _) =
                sign_refusal(parse_csv_collect(csv.as_bytes(), &split_profile(*sign)));
            assert_eq!(total, 1, "{sign:?} {row}");
            assert_eq!(errors[0].code, CsvLineErrorCode::AmountSignMismatch);
            assert_eq!(errors[0].value.as_deref(), Some(*value));
            assert_eq!(errors[0].line, 2);
        }
    }

    /// AC14-7 (M19) : zéro accepté partout (`0.00`, `-0.00`) — comparaison
    /// stricte à zéro, jamais le bit de signe ; un crédit `+500.00` est lu
    /// `500.00` et accepté.
    #[test]
    fn zeros_and_explicit_plus_are_accepted() {
        for sign in [DebitSign::Positive, DebitSign::Negative] {
            let csv = "date;debit;credit\n2026-01-01;0.00;\n2026-01-02;-0.00;\n\
                       2026-01-03;;0.00\n2026-01-04;;-0.00\n2026-01-05;;+500.00\n";
            match parse_csv_collect(csv.as_bytes(), &split_profile(sign)) {
                ParseCsvOutcome::AllValid(stmt) => {
                    assert_eq!(stmt.transactions.len(), 5, "{sign:?}");
                    for t in &stmt.transactions[..4] {
                        assert_eq!(t.amount, Decimal::ZERO);
                        // Revue P1, E-4 : `==` ignore le bit de signe ; le
                        // montant produit n'en porte pas (« -0.00 » sinon).
                        assert!(!t.amount.is_sign_negative(), "{sign:?} {t:?}");
                        // Revue P2, E2-2 : `==` ignore aussi l'échelle ; le
                        // montant garde celle du fichier (« 0 » sinon).
                        assert_eq!(t.amount.to_string(), "0.00", "{sign:?} {t:?}");
                    }
                    assert_eq!(stmt.transactions[4].amount, dec!(500.00));
                }
                other => panic!("expected AllValid under {sign:?}, got {other:?}"),
            }
        }
    }

    /// M19 — constat sur `rust_decimal`, figé : `-0.00` lu par
    /// `parse_amount` ne porte **pas** le bit de signe. La mutation « comparer
    /// par `is_sign_negative()` » est donc aujourd'hui **équivalente** à la
    /// comparaison stricte (elle ne peut pas rougir), alors que sa forme
    /// « `<= ZERO` » rougit sur `zeros_and_explicit_plus_are_accepted`. Si la
    /// bibliothèque se mettait à garder le signe d'un zéro, ce test rougirait
    /// et rappellerait que la comparaison doit rester stricte à `ZERO`.
    #[test]
    fn negative_zero_has_no_sign_bit_after_parse() {
        let z = parse_amount("-0.00", '.').unwrap();
        assert_eq!(z, Decimal::ZERO);
        assert!(!z.is_sign_negative());
    }

    /// AC2 : `Positive` rend `-d`, `Negative` garde `d` ; le crédit est
    /// inchangé.
    #[test]
    fn debit_sign_setting_drives_amount() {
        let csv_pos = "date;debit;credit\n2026-01-01;12.50;\n2026-01-02;;3.00\n";
        let stmt = parse_csv(csv_pos.as_bytes(), &split_profile(DebitSign::Positive)).unwrap();
        assert_eq!(stmt.transactions[0].amount, dec!(-12.50));
        assert_eq!(stmt.transactions[1].amount, dec!(3.00));
        let csv_neg = "date;debit;credit\n2026-01-01;-12.50;\n2026-01-02;;3.00\n";
        let stmt = parse_csv(csv_neg.as_bytes(), &split_profile(DebitSign::Negative)).unwrap();
        assert_eq!(stmt.transactions[0].amount, dec!(-12.50));
        assert_eq!(stmt.transactions[1].amount, dec!(3.00));
    }

    /// AC14-7-bis (a) (M17) : 101 erreurs `INVALID_AMOUNT` puis une erreur
    /// de signe → refus de signe : la décision porte sur le compteur
    /// propre au signe, pas sur le vecteur général plein.
    #[test]
    fn sign_counter_is_independent_of_general_cap() {
        let mut csv = String::from("date;debit;credit\n");
        for _ in 0..101 {
            csv.push_str("2026-01-01;abc;\n");
        }
        csv.push_str("2026-01-02;;-1.00\n");
        let (errors, total, truncated, _) = sign_refusal(parse_csv_collect(
            csv.as_bytes(),
            &split_profile(DebitSign::Positive),
        ));
        assert_eq!(total, 1);
        assert!(!truncated);
        assert_eq!(errors.len(), 1);
    }

    /// AC14-7-bis (b) : 150 débits négatifs sous `Positive` → 150, tronqué
    /// à 100.
    #[test]
    fn sign_errors_are_capped_with_full_count() {
        let mut csv = String::from("date;debit;credit\n");
        for _ in 0..150 {
            csv.push_str("2026-01-01;-1.00;\n");
        }
        let (errors, total, truncated, whole) = sign_refusal(parse_csv_collect(
            csv.as_bytes(),
            &split_profile(DebitSign::Positive),
        ));
        assert_eq!(total, 150);
        assert!(truncated);
        assert_eq!(errors.len(), 100);
        assert!(whole);
    }

    /// Revue P2, E2-3 : borne exacte de `truncated` du refus de signe —
    /// à exactement `MAX_CSV_LINE_ERRORS` (100) lignes en cause, toutes
    /// sont rendues et rien n'est tronqué ; à 101, tronqué à 100.
    #[test]
    fn sign_errors_truncation_boundary() {
        for (n, expect_truncated) in [
            (MAX_CSV_LINE_ERRORS, false),
            (MAX_CSV_LINE_ERRORS + 1, true),
        ] {
            let mut csv = String::from("date;debit;credit\n");
            for _ in 0..n {
                csv.push_str("2026-01-01;-1.00;\n");
            }
            let (errors, total, truncated, _) = sign_refusal(parse_csv_collect(
                csv.as_bytes(),
                &split_profile(DebitSign::Positive),
            ));
            assert_eq!(total, n, "n = {n}");
            assert_eq!(errors.len(), MAX_CSV_LINE_ERRORS, "n = {n}");
            assert_eq!(truncated, expect_truncated, "n = {n}");
        }
    }

    /// AC14-7-ter (M21, M23) : `whole_debit_column ⇔ d ≥ 2 et e = d`, sur
    /// les seuls débits lus.
    #[test]
    fn whole_debit_column_definition() {
        let whole = |sign: DebitSign, rows: &str| -> bool {
            let csv = format!("date;debit;credit\n{rows}");
            sign_refusal(parse_csv_collect(csv.as_bytes(), &split_profile(sign))).3
        };
        let pos = DebitSign::Positive;
        // Débits justes + un crédit négatif (extourne) → false, refus quand même.
        let csv = "date;debit;credit\n2026-01-01;5.00;\n2026-01-02;6.00;\n2026-01-03;;-7.00\n";
        let (_, total, _, w) = sign_refusal(parse_csv_collect(csv.as_bytes(), &split_profile(pos)));
        assert_eq!(total, 1);
        assert!(!w);
        // Une partie seulement des débits en défaut → false.
        assert!(!whole(
            pos,
            "2026-01-01;-5.00;\n2026-01-02;6.00;\n2026-01-03;-7.00;\n"
        ));
        // Aucun débit (crédits seulement, un crédit négatif) → false.
        assert!(!whole(pos, "2026-01-01;;5.00\n2026-01-02;;-6.00\n"));
        // Débits tous à 0.00 + un crédit négatif → false.
        assert!(!whole(
            pos,
            "2026-01-01;0.00;\n2026-01-02;0.00;\n2026-01-03;;-6.00\n"
        ));
        // Revue P1, E-2 : un débit à zéro est hors dénominateur — deux
        // débits en défaut + un débit 0.00 → true (compté comme lu, d = 3,
        // e = 2, il rendrait false).
        assert!(whole(
            pos,
            "2026-01-01;-5.00;\n2026-01-02;-6.00;\n2026-01-03;0.00;\n"
        ));
        // Revue P1, E-2 : un crédit négatif n'est pas une observation de
        // débit — deux crédits négatifs, aucun débit → false (compté comme
        // débit en défaut, d = e = 2, il rendrait true).
        assert!(!whole(pos, "2026-01-01;;-5.00\n2026-01-02;;-6.00\n"));
        // Un seul débit, en défaut (d = 1) → false (message « lignes listées »).
        assert!(!whole(pos, "2026-01-01;-5.00;\n2026-01-02;;6.00\n"));
        // Deux débits, tous deux en défaut → true.
        assert!(whole(pos, "2026-01-01;-5.00;\n2026-01-02;-6.00;\n"));
        // Profil Negative dont tous les débits (≥ 2) sont positifs → true.
        assert!(whole(
            DebitSign::Negative,
            "2026-01-01;5.00;\n2026-01-02;6.00;\n2026-01-03;;1.00\n"
        ));
        // Débit illisible + deux débits en défaut → true (hors dénominateur).
        assert!(whole(
            pos,
            "2026-01-01;abc;\n2026-01-02;-5.00;\n2026-01-03;-6.00;\n"
        ));
        // Débit de mauvais signe à date illisible + deux débits justes →
        // pas de refus de signe (la ligne n'entre pas au dénominateur).
        let csv = "date;debit;credit\nXX;-5.00;\n2026-01-02;5.00;\n2026-01-03;6.00;\n";
        match parse_csv_collect(csv.as_bytes(), &split_profile(pos)) {
            ParseCsvOutcome::PartialFailure { errors, .. } => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].code, CsvLineErrorCode::InvalidDate);
            }
            other => panic!("expected PartialFailure, got {other:?}"),
        }
    }

    /// AC14-7-quater (M24) : sur une même ligne, la date est lue avant le
    /// montant — date illisible et débit de mauvais signe → `INVALID_DATE`,
    /// pas de refus de signe ; sur deux lignes distinctes → refus de signe.
    #[test]
    fn date_is_checked_before_sign_on_the_same_row() {
        let pos = split_profile(DebitSign::Positive);
        let same = "date;debit;credit\nXX;-5.00;\n2026-01-02;5.00;\n";
        match parse_csv_collect(same.as_bytes(), &pos) {
            ParseCsvOutcome::PartialFailure { errors, .. } => {
                assert_eq!(errors[0].code, CsvLineErrorCode::InvalidDate);
            }
            other => panic!("expected PartialFailure, got {other:?}"),
        }
        let two = "date;debit;credit\nXX;5.00;\n2026-01-02;-5.00;\n";
        let (errors, total, _, _) = sign_refusal(parse_csv_collect(two.as_bytes(), &pos));
        assert_eq!(total, 1);
        assert_eq!(errors[0].line, 3);
    }

    /// AC2 : la colonne unique garde le signe du fichier, quel que soit le
    /// contenu (aucun contrôle de signe).
    #[test]
    fn single_column_keeps_file_sign() {
        let csv = "date;amount\n2026-01-01;-5.00\n2026-01-02;5.00\n";
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
                currency: None,
            },
        );
        let stmt = parse_csv(csv.as_bytes(), &profile).unwrap();
        assert_eq!(stmt.transactions[0].amount, dec!(-5.00));
        assert_eq!(stmt.transactions[1].amount, dec!(5.00));
    }

    /// AC14-10 (R3-11, M28) : la fixture garde la forme du relevé UBS —
    /// BOM, pas de LF final, aucun `\r`, 21 lignes dont la 9ᵉ vide.
    #[test]
    fn ubs_fixture_has_the_documented_shape() {
        assert_eq!(&UBS_FIXTURE[..3], b"\xEF\xBB\xBF");
        assert_ne!(*UBS_FIXTURE.last().unwrap(), b'\n');
        assert!(!UBS_FIXTURE.contains(&b'\r'));
        let text = std::str::from_utf8(&UBS_FIXTURE[3..]).unwrap();
        let lines: Vec<&str> = text.split('\n').collect();
        assert_eq!(lines.len(), 21);
        assert_eq!(lines[8], "");
        assert!(lines.iter().skip(9).all(|l| l.split(';').count() >= 15));
    }

    // ── Story 27-2 (#624) : la colonne Devise ──────────────────────────

    /// Profil UBS recommandé (26-1, `Negative`, 9 lignes ignorées) qui lit
    /// la colonne Devise (4) — AC13.
    fn ubs_currency_profile(currency: Option<usize>) -> CsvProfile {
        let mut p = ubs_profile(DebitSign::Negative, 9);
        p.column_mapping.currency = currency;
        p
    }

    /// Dérive une variante de `UBS_FIXTURE` en modifiant ses **seules lignes
    /// de données** (C-27-2-9) : `edit(i, champs)` reçoit l'indice 0-based
    /// de la ligne de données et ses cellules, et rend `true` s'il en a
    /// modifié une. Rend les octets (BOM compris) et le nombre de lignes
    /// modifiées, que chaque test asserte.
    ///
    /// Le saut suit la règle du lecteur `csv` : `header_row_count` (9)
    /// lignes **non vides** — préambule et ligne de titres —, une ligne
    /// « vide » étant une ligne de **zéro caractère** une fois le
    /// terminateur retiré (`line.is_empty()`, jamais `trim()` : une ligne
    /// d'espaces ou de seuls `;` est un enregistrement et compte ;
    /// C-27-2-23). La fixture a 10 lignes physiques avant la première
    /// donnée, dont une vide.
    fn derive_ubs(mut edit: impl FnMut(usize, &mut Vec<String>) -> bool) -> (Vec<u8>, usize) {
        let text = std::str::from_utf8(&UBS_FIXTURE[3..]).unwrap();
        let mut skipped = 0usize;
        let mut data_index = 0usize;
        let mut modified = 0usize;
        let mut out: Vec<String> = Vec::new();
        for line in text.split('\n') {
            if skipped < 9 {
                if !line.is_empty() {
                    skipped += 1;
                }
                out.push(line.to_string());
                continue;
            }
            let mut fields: Vec<String> = line.split(';').map(String::from).collect();
            if edit(data_index, &mut fields) {
                modified += 1;
            }
            data_index += 1;
            out.push(fields.join(";"));
        }
        assert_eq!(data_index, 11, "la fixture porte 11 lignes de données");
        let mut bytes = b"\xEF\xBB\xBF".to_vec();
        bytes.extend_from_slice(out.join("\n").as_bytes());
        (bytes, modified)
    }

    /// Remplace la devise (colonne 4, `CHF` dans la fixture) des lignes de
    /// données que `pick` désigne.
    fn derive_currency(mut pick: impl FnMut(usize) -> Option<String>) -> (Vec<u8>, usize) {
        derive_ubs(|i, f| match pick(i) {
            Some(c) => {
                assert_eq!(f[4], "CHF", "ligne de données {i}");
                f[4] = c;
                true
            }
            None => false,
        })
    }

    fn all_valid(outcome: ParseCsvOutcome) -> ImportedStatement {
        match outcome {
            ParseCsvOutcome::AllValid(stmt) => stmt,
            other => panic!("expected AllValid, got {other:?}"),
        }
    }

    /// AC13-1 : la fixture telle quelle, colonne Devise réglée → CHF lu.
    #[test]
    fn ubs_currency_column_reads_chf() {
        let stmt = all_valid(parse_csv_collect(
            UBS_FIXTURE,
            &ubs_currency_profile(Some(4)),
        ));
        assert_eq!(stmt.currency, "CHF");
        assert_eq!(stmt.transactions.len(), 11);
        assert!(stmt.transactions.iter().all(|t| t.currency == "CHF"));
    }

    /// AC13-2 : toutes les lignes en EUR → relevé EUR (le parseur reste
    /// agnostique ; le refus est côté API).
    #[test]
    fn ubs_currency_column_reads_eur() {
        let (bytes, n) = derive_currency(|_| Some("EUR".into()));
        assert_eq!(n, 11);
        let stmt = all_valid(parse_csv_collect(&bytes, &ubs_currency_profile(Some(4))));
        assert_eq!(stmt.currency, "EUR");
        assert_eq!(stmt.transactions.len(), 11);
        assert!(stmt.transactions.iter().all(|t| t.currency == "EUR"));
    }

    /// AC13-3 : une seule ligne en EUR → refus du mélange.
    #[test]
    fn one_eur_row_is_mixed_currencies() {
        let (bytes, n) = derive_currency(|i| (i == 3).then(|| "EUR".into()));
        assert_eq!(n, 1);
        assert_eq!(
            parse_csv_collect(&bytes, &ubs_currency_profile(Some(4))),
            ParseCsvOutcome::HardFailure(CsvError::MixedCurrencies {
                currencies: vec!["CHF".into(), "EUR".into()]
            })
        );
    }

    /// AC13-4 : `eur` partout → `EUR` (casse normalisée).
    #[test]
    fn lowercase_currency_is_uppercased() {
        let (bytes, n) = derive_currency(|_| Some("eur".into()));
        assert_eq!(n, 11);
        let stmt = all_valid(parse_csv_collect(&bytes, &ubs_currency_profile(Some(4))));
        assert_eq!(stmt.currency, "EUR");
        assert!(stmt.transactions.iter().all(|t| t.currency == "EUR"));
    }

    /// Revue de code P1 (E-3) : la cellule Devise est rognée avant d'être
    /// lue — ` chf ` vaut `CHF`, jamais `INVALID_CURRENCY`.
    #[test]
    fn padded_currency_cell_is_trimmed() {
        let (bytes, n) = derive_currency(|_| Some(" chf ".into()));
        assert_eq!(n, 11);
        let stmt = all_valid(parse_csv_collect(&bytes, &ubs_currency_profile(Some(4))));
        assert_eq!(stmt.currency, "CHF");
        assert_eq!(stmt.transactions.len(), 11);
        assert!(stmt.transactions.iter().all(|t| t.currency == "CHF"));
    }

    /// Revue de code P1 (E-1) : colonne Devise réglée, **aucune** ligne
    /// valide (toutes les cellules Devise vides) → la devise du relevé est
    /// `DEFAULT_CSV_CURRENCY`, et non une valeur tirée d'ailleurs : l'aperçu
    /// d'un tel fichier n'annonce pas de devise non supportée.
    #[test]
    fn currency_column_without_valid_line_gives_default_currency() {
        let (bytes, n) = derive_currency(|_| Some(String::new()));
        assert_eq!(n, 11);
        match parse_csv_collect(&bytes, &ubs_currency_profile(Some(4))) {
            ParseCsvOutcome::PartialFailure {
                valid,
                total_errors,
                ..
            } => {
                assert_eq!(total_errors, 11);
                assert!(valid.transactions.is_empty());
                assert_eq!(valid.currency, DEFAULT_CSV_CURRENCY);
                assert_eq!(valid.currency, "CHF");
            }
            other => panic!("expected PartialFailure, got {other:?}"),
        }
    }

    /// AC13-5 : une cellule vide → erreur de ligne `EMPTY_MANDATORY_FIELD`
    /// (`currency`), jamais CHF supposé ; les 10 autres lignes en CHF.
    #[test]
    fn empty_currency_cell_is_a_line_error() {
        let (bytes, n) = derive_currency(|i| (i == 5).then(String::new));
        assert_eq!(n, 1);
        match parse_csv_collect(&bytes, &ubs_currency_profile(Some(4))) {
            ParseCsvOutcome::PartialFailure {
                valid,
                errors,
                total_errors,
                ..
            } => {
                assert_eq!(total_errors, 1);
                assert_eq!(errors[0].code, CsvLineErrorCode::EmptyMandatoryField);
                assert_eq!(errors[0].value.as_deref(), Some("currency"));
                assert_eq!(valid.currency, "CHF");
                assert_eq!(valid.transactions.len(), 10);
                assert!(valid.transactions.iter().all(|t| t.currency == "CHF"));
            }
            other => panic!("expected PartialFailure, got {other:?}"),
        }
    }

    /// AC13-6 : formes illisibles → `INVALID_CURRENCY`. `ÉUR` (trois
    /// caractères, quatre octets) tue un contrôle alphabétique Unicode,
    /// `ÉU` (trois octets) un contrôle sur la longueur en octets, `CH1` un
    /// contrôle alphanumérique, `CH` une borne « au plus trois ».
    #[test]
    fn malformed_currency_cells_are_invalid_currency() {
        for bad in ["Fr.", "€", "CHF1", "ÉUR", "ÉU", "CH1", "CH"] {
            let (bytes, n) = derive_currency(|i| (i == 2).then(|| bad.to_string()));
            assert_eq!(n, 1);
            match parse_csv_collect(&bytes, &ubs_currency_profile(Some(4))) {
                ParseCsvOutcome::PartialFailure {
                    valid,
                    errors,
                    total_errors,
                    ..
                } => {
                    assert_eq!(total_errors, 1, "{bad}");
                    assert_eq!(errors[0].code, CsvLineErrorCode::InvalidCurrency, "{bad}");
                    assert_eq!(errors[0].value.as_deref(), Some(bad));
                    assert_eq!(valid.transactions.len(), 10, "{bad}");
                }
                other => panic!("{bad} : expected PartialFailure, got {other:?}"),
            }
        }
    }

    /// AC13-7 : mélange ET erreur de ligne → le mélange l'emporte (refus
    /// non confirmable).
    #[test]
    fn mixed_currencies_dominates_line_errors() {
        let (bytes, n) = derive_ubs(|i, f| match i {
            1 => {
                f[4] = "EUR".into();
                true
            }
            6 => {
                f[2] = "pas-une-date".into();
                true
            }
            _ => false,
        });
        assert_eq!(n, 2);
        assert!(matches!(
            parse_csv_collect(&bytes, &ubs_currency_profile(Some(4))),
            ParseCsvOutcome::HardFailure(CsvError::MixedCurrencies { .. })
        ));
    }

    /// AC13-8 : un débit de signe contraire ET une AUTRE ligne, valide, en
    /// EUR → le refus de signe l'emporte (ordre de l'AC3). Pas la même
    /// ligne : une ligne de signe contraire sort avant la lecture de sa
    /// devise.
    #[test]
    fn sign_mismatch_dominates_mixed_currencies() {
        let mut debit_flipped = false;
        let (bytes, n) = derive_ubs(|i, f| {
            if !debit_flipped && f[5].starts_with('-') {
                f[5] = f[5].trim_start_matches('-').to_string();
                debit_flipped = true;
                return true;
            }
            if i == 10 {
                assert!(f[5].is_empty() || f[5].starts_with('-'));
                f[4] = "EUR".into();
                return true;
            }
            false
        });
        assert_eq!(n, 2);
        let (_, total, _, _) =
            sign_refusal(parse_csv_collect(&bytes, &ubs_currency_profile(Some(4))));
        assert_eq!(total, 1);
    }

    /// AC13-9 : sans colonne Devise → la constante nommée, pour le relevé
    /// et pour chaque ligne (comportement d'avant).
    #[test]
    fn without_currency_column_default_is_assumed() {
        let (bytes, _) = derive_currency(|_| Some("EUR".into()));
        let stmt = all_valid(parse_csv_collect(&bytes, &ubs_currency_profile(None)));
        assert_eq!(stmt.currency, DEFAULT_CSV_CURRENCY);
        assert!(
            stmt.transactions
                .iter()
                .all(|t| t.currency == DEFAULT_CSV_CURRENCY)
        );
    }

    /// AC13-10 : colonne Devise hors borne → profil mal configuré.
    #[test]
    fn currency_column_out_of_bounds_is_misconfigured() {
        match parse_csv_collect(UBS_FIXTURE, &ubs_currency_profile(Some(40))) {
            ParseCsvOutcome::HardFailure(CsvError::ProfileMisconfigured(msg)) => {
                assert!(msg.contains("column_mapping.currency"), "{msg}");
            }
            other => panic!("expected ProfileMisconfigured, got {other:?}"),
        }
    }

    /// AC13-11 : 11 devises distinctes → les 10 PREMIÈRES DU TRI. Le code
    /// qui se trie en dernier (`ZZZ`) est sur la PREMIÈRE ligne : une
    /// troncature « 10 premières rencontrées » le garderait.
    #[test]
    fn mixed_currencies_reports_first_ten_sorted() {
        let codes = [
            "ZZZ", "AAA", "AAB", "AAC", "AAD", "AAE", "AAF", "AAG", "AAH", "AAI", "AAJ",
        ];
        let (bytes, n) = derive_currency(|i| Some(codes[i].to_string()));
        assert_eq!(n, 11);
        let mut expected: Vec<String> = codes.iter().map(|c| c.to_string()).collect();
        expected.sort();
        expected.truncate(MAX_REPORTED_CURRENCIES);
        assert_eq!(expected.len(), 10);
        assert!(!expected.contains(&"ZZZ".to_string()));
        assert_eq!(
            parse_csv_collect(&bytes, &ubs_currency_profile(Some(4))),
            ParseCsvOutcome::HardFailure(CsvError::MixedCurrencies {
                currencies: expected
            })
        );
    }

    /// AC13-12 : l'erreur de devise GARDE l'observation de débit (AC2). Un
    /// débit négatif (cohérent sous `Negative`) a sa cellule Devise vidée,
    /// les huit autres débits passent en positif : `d` = 9, `e` = 8, donc
    /// `whole_debit_column = false`. Sous `RowParse::err` sur l'erreur de
    /// devise, `d` = 8 = `e` et le conseil de régler le profil serait donné
    /// à tort (C-26-1-25).
    #[test]
    fn currency_error_keeps_debit_observation() {
        let mut kept_one = false;
        let mut debits = 0usize;
        let (bytes, n) = derive_ubs(|_, f| {
            if !f[5].starts_with('-') {
                return false;
            }
            debits += 1;
            if !kept_one {
                kept_one = true;
                f[4] = String::new();
            } else {
                f[5] = f[5].trim_start_matches('-').to_string();
            }
            true
        });
        assert_eq!(debits, 9, "la fixture porte neuf débits");
        assert_eq!(n, 9);
        let (_, total, _, whole) =
            sign_refusal(parse_csv_collect(&bytes, &ubs_currency_profile(Some(4))));
        assert_eq!(total, 8);
        assert!(!whole, "d = 9, e = 8 : pas toute la colonne");
    }
}
