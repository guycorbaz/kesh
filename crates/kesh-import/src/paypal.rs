//! Lecteur du CSV PayPal « Balance affecting transactions » (Story 38-11a).
//!
//! PayPal exporte, pour une période, les mouvements qui touchent le solde du
//! compte : un CSV UTF-8 (avec BOM), à virgule et guillemets doubles, dont
//! l'en-tête porte une quarantaine de colonnes **en anglais**. Ce module le lit
//! **par noms de colonnes** ([`crate::csv::ColumnsByName`]), contrôle chaque
//! ligne et rend des [`PaypalRow`]. Il ne filtre rien : statut, lignes
//! `Memo`, doublons sont l'affaire de l'import (Story 38-11b).
//!
//! Garanties, chacune refusant le fichier **entier** (pas d'import partiel) :
//!
//! - les colonnes requises ([`REQUIRED_COLUMNS`]) sont toutes là, sinon
//!   [`PaypalCsvError::MissingColumns`] les nomme toutes — un CSV d'une autre
//!   langue ou d'un autre rapport s'arrête là ;
//! - les montants sont à forme fermée ([`crate::csv::parse_dot_decimal_amount`]) :
//!   aucune virgule décimale ne peut se lire comme une virgule des milliers ;
//! - `net = gross + fee`, et le signe du brut suit `Balance Impact` ;
//! - l'ordre jour/mois d'une date à barre oblique se décide sur **tout** le
//!   fichier, jamais par supposition ([`DateOrder`]).
//!
//! L'heure et le fuseau sont rendus tels quels, sans conversion : la date est
//! celle de l'export, dans le fuseau choisi dans PayPal.
//!
//! Fonction pure, sans I/O, sans dépendance interne au workspace.

use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::csv::encoding::{decode_bytes, detect_encoding};
use crate::csv::{ColumnsByName, HeaderByNameError, parse_dot_decimal_amount};
use crate::error::{CsvError, MAX_CSV_LINE_ERROR_VALUE_CHARS, MAX_CSV_LINE_ERRORS};

/// Colonnes que l'import et le rapprochement lisent (38-11b, 38-11c2) :
/// toutes requises, dans l'ordre où un refus les nomme.
pub const REQUIRED_COLUMNS: [&str; 13] = [
    COL_DATE,
    COL_NAME,
    COL_TYPE,
    COL_STATUS,
    COL_CURRENCY,
    COL_GROSS,
    COL_FEE,
    COL_NET,
    COL_FROM_EMAIL,
    COL_TO_EMAIL,
    COL_TRANSACTION_ID,
    COL_REFERENCE_TXN_ID,
    COL_BALANCE_IMPACT,
];

/// Colonnes lues si présentes ; absentes, elles se lisent vides.
pub const OPTIONAL_COLUMNS: [&str; 7] = [
    COL_TIME,
    COL_TIME_ZONE,
    COL_ITEM_TITLE,
    COL_SUBJECT,
    COL_NOTE,
    COL_BALANCE,
    COL_INVOICE_NUMBER,
];

/// Colonne `Date`.
pub const COL_DATE: &str = "Date";
/// Colonne `Time`.
pub const COL_TIME: &str = "Time";
/// Colonne `TimeZone`.
pub const COL_TIME_ZONE: &str = "TimeZone";
/// Colonne `Name`.
pub const COL_NAME: &str = "Name";
/// Colonne `Type`.
pub const COL_TYPE: &str = "Type";
/// Colonne `Status`.
pub const COL_STATUS: &str = "Status";
/// Colonne `Currency`.
pub const COL_CURRENCY: &str = "Currency";
/// Colonne `Gross`.
pub const COL_GROSS: &str = "Gross";
/// Colonne `Fee`.
pub const COL_FEE: &str = "Fee";
/// Colonne `Net`.
pub const COL_NET: &str = "Net";
/// Colonne `From Email Address`.
pub const COL_FROM_EMAIL: &str = "From Email Address";
/// Colonne `To Email Address`.
pub const COL_TO_EMAIL: &str = "To Email Address";
/// Colonne `Transaction ID`.
pub const COL_TRANSACTION_ID: &str = "Transaction ID";
/// Colonne `Reference Txn ID`.
pub const COL_REFERENCE_TXN_ID: &str = "Reference Txn ID";
/// Colonne `Balance Impact`.
pub const COL_BALANCE_IMPACT: &str = "Balance Impact";
/// Colonne `Balance`.
pub const COL_BALANCE: &str = "Balance";
/// Colonne `Item Title`.
pub const COL_ITEM_TITLE: &str = "Item Title";
/// Colonne `Subject`.
pub const COL_SUBJECT: &str = "Subject";
/// Colonne `Note`.
pub const COL_NOTE: &str = "Note";
/// Colonne `Invoice Number`.
pub const COL_INVOICE_NUMBER: &str = "Invoice Number";

/// Sens d'une ligne sur le solde PayPal (colonne `Balance Impact`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BalanceImpact {
    /// Entrée sur le solde (`Credit`) : brut positif ou nul.
    Credit,
    /// Sortie du solde (`Debit`) : brut négatif ou nul.
    Debit,
    /// Ligne pour mémoire (`Memo`), sans effet sur le solde ; aucun contrôle
    /// de signe.
    Memo,
}

/// Format des dates du fichier, décidé sur l'ensemble de ses lignes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DateOrder {
    /// `JJ/MM/AAAA`.
    DayFirst,
    /// `MM/JJ/AAAA`.
    MonthFirst,
    /// `JJ.MM.AAAA`.
    Dotted,
    /// `AAAA-MM-JJ`.
    Iso,
}

/// Une ligne du CSV PayPal, contrôlée.
///
/// Les champs texte facultatifs valent `None` quand la cellule est vide après
/// `trim` (ou que la colonne facultative est absente).
#[derive(Debug, Clone, PartialEq)]
pub struct PaypalRow {
    /// Numéro de ligne dans le fichier (1 = l'en-tête).
    pub line: usize,
    /// Date de la ligne, dans le fuseau de l'export.
    pub date: NaiveDate,
    /// Heure, telle qu'écrite (affichage seulement).
    pub time: Option<String>,
    /// Fuseau, tel qu'écrit (affichage seulement, aucune conversion).
    pub time_zone: Option<String>,
    /// Nom de la contrepartie.
    pub name: Option<String>,
    /// Type PayPal (`Website Payment`, `General Payment`…), tel qu'écrit.
    pub kind: String,
    /// Statut PayPal (`Completed`, `Pending`, `Denied`…), tel qu'écrit.
    pub status: String,
    /// Code de devise, trois lettres ASCII majuscules.
    pub currency: String,
    /// Montant brut.
    pub gross: Decimal,
    /// Frais (en général négatifs ; positifs sur certains remboursements).
    pub fee: Decimal,
    /// Net = brut + frais (contrôlé).
    pub net: Decimal,
    /// E-mail de l'émetteur.
    pub from_email: Option<String>,
    /// E-mail du destinataire.
    pub to_email: Option<String>,
    /// Identifiant de la transaction, `[0-9A-Z]` seulement (clé de doublon de
    /// l'import).
    pub transaction_id: String,
    /// Identifiant de la transaction liée (conversion, remboursement,
    /// retenue…), en majuscules.
    pub reference_txn_id: Option<String>,
    /// Sens sur le solde.
    pub balance_impact: BalanceImpact,
    /// Solde après la ligne, s'il est écrit.
    pub balance: Option<Decimal>,
    /// Titre de l'article.
    pub item_title: Option<String>,
    /// Objet.
    pub subject: Option<String>,
    /// Note (peut tenir sur plusieurs lignes).
    pub note: Option<String>,
    /// Numéro de facture.
    pub invoice_number: Option<String>,
}

/// Le fichier lu : ses lignes, dans l'ordre du fichier, et le format de date
/// décidé.
#[derive(Debug, Clone, PartialEq)]
pub struct PaypalCsv {
    /// Les lignes de données, sans filtre.
    pub rows: Vec<PaypalRow>,
    /// Le format de date du fichier.
    pub date_order: DateOrder,
}

/// Code stable d'une erreur de ligne (sérialisé en `SCREAMING_SNAKE_CASE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaypalLineErrorCode {
    /// La ligne n'a pas autant de cellules que l'en-tête.
    FieldCount,
    /// Date illisible, hors format, hors plage d'années, ou d'un autre format
    /// que celui du fichier.
    Date,
    /// Montant hors de la forme fermée (ou vide pour `Gross`, `Fee`, `Net`).
    Amount,
    /// `net ≠ gross + fee`.
    NetMismatch,
    /// Brut négatif sur un `Credit`, ou positif sur un `Debit`.
    SignMismatch,
    /// Devise autre que trois lettres ASCII.
    Currency,
    /// Identifiant de transaction vide ou hors `[0-9A-Z]`.
    TransactionId,
    /// `Balance Impact` autre que `Credit`, `Debit`, `Memo`.
    BalanceImpact,
    /// Cellule requise vide (`Type`, `Status`).
    Required,
}

impl PaypalLineErrorCode {
    /// Graphie stable du code, identique à sa sérialisation serde.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FieldCount => "FIELD_COUNT",
            Self::Date => "DATE",
            Self::Amount => "AMOUNT",
            Self::NetMismatch => "NET_MISMATCH",
            Self::SignMismatch => "SIGN_MISMATCH",
            Self::Currency => "CURRENCY",
            Self::TransactionId => "TRANSACTION_ID",
            Self::BalanceImpact => "BALANCE_IMPACT",
            Self::Required => "REQUIRED",
        }
    }
}

/// Une erreur de ligne, nommant la ligne, le champ et la valeur fautive.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct PaypalLineError {
    /// Numéro de ligne dans le fichier (1 = l'en-tête).
    pub line: usize,
    /// Nom de la colonne en cause ; `None` pour [`PaypalLineErrorCode::FieldCount`].
    pub field: Option<&'static str>,
    /// Code stable.
    pub code: PaypalLineErrorCode,
    /// Valeur fautive, tronquée à [`MAX_CSV_LINE_ERROR_VALUE_CHARS`]
    /// caractères (suffixe `…`).
    pub value: Option<String>,
}

impl PaypalLineError {
    fn new(
        line: usize,
        field: Option<&'static str>,
        code: PaypalLineErrorCode,
        value: Option<&str>,
    ) -> Self {
        let value = value.map(|v| {
            if v.chars().count() <= MAX_CSV_LINE_ERROR_VALUE_CHARS {
                v.to_string()
            } else {
                let mut s: String = v.chars().take(MAX_CSV_LINE_ERROR_VALUE_CHARS).collect();
                s.push('…');
                s
            }
        });
        Self {
            line,
            field,
            code,
            value,
        }
    }
}

/// Refus du fichier PayPal. Type fermé : chaque variante a un code stable
/// ([`PaypalCsvError::code`]).
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum PaypalCsvError {
    /// Fichier vide, ou en-tête sans ligne de données.
    #[error("fichier PayPal vide")]
    Empty,
    /// Encodage non reconnu ou décodage impossible.
    #[error("encodage du fichier PayPal : {0}")]
    Encoding(CsvError),
    /// Colonnes requises absentes, toutes, dans l'ordre de [`REQUIRED_COLUMNS`].
    #[error("colonnes PayPal requises absentes : {}", .0.join(", "))]
    MissingColumns(Vec<String>),
    /// Une colonne lue apparaît deux fois dans l'en-tête.
    #[error("colonne PayPal en double : {0}")]
    DuplicateColumn(String),
    /// Dates à barre oblique dont certaines ne se lisent que jour en tête et
    /// d'autres que mois en tête.
    #[error("dates du fichier PayPal contradictoires (jour et mois en tête)")]
    DateOrderConflict,
    /// Dates à barre oblique dont aucune ne dit l'ordre (jour et mois tous
    /// deux ≤ 12) : exporter une période plus longue, ou changer le format de
    /// date dans PayPal.
    #[error("ordre jour/mois des dates du fichier PayPal indécidable")]
    DateOrderAmbiguous,
    /// Erreurs de ligne, au plus [`MAX_CSV_LINE_ERRORS`] (la collecte s'arrête
    /// au plafond).
    #[error("{} ligne(s) PayPal en erreur", .0.len())]
    Lines(Vec<PaypalLineError>),
}

impl PaypalCsvError {
    /// Code stable de la variante (placé par l'import dans `details`).
    pub fn code(&self) -> &'static str {
        match self {
            Self::Empty => "PAYPAL_CSV_EMPTY",
            Self::Encoding(_) => "PAYPAL_CSV_ENCODING",
            Self::MissingColumns(_) => "PAYPAL_CSV_MISSING_COLUMNS",
            Self::DuplicateColumn(_) => "PAYPAL_CSV_DUPLICATE_COLUMN",
            Self::DateOrderConflict => "PAYPAL_CSV_DATE_ORDER_CONFLICT",
            Self::DateOrderAmbiguous => "PAYPAL_CSV_DATE_ORDER_AMBIGUOUS",
            Self::Lines(_) => "PAYPAL_CSV_LINES",
        }
    }
}

/// Collecte plafonnée des erreurs de ligne.
#[derive(Default)]
struct Errors(Vec<PaypalLineError>);

impl Errors {
    fn push(&mut self, e: PaypalLineError) {
        if self.0.len() < MAX_CSV_LINE_ERRORS {
            self.0.push(e);
        }
    }
}

/// Forme d'une date, avant la décision de l'ordre jour/mois.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DateShape {
    /// `a/b/aaaa`, ordre à décider.
    Slash(u32, u32, i32),
    /// `jj.mm.aaaa`.
    Dotted(u32, u32, i32),
    /// `aaaa-mm-jj`.
    Iso(i32, u32, u32),
}

impl DateShape {
    fn same_format(&self, other: &DateShape) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

/// Reconnaît la forme d'une date : jour et mois sur un ou deux chiffres ASCII,
/// année sur quatre.
fn date_shape(raw: &str) -> Option<DateShape> {
    fn num(s: &str, min: usize, max: usize) -> Option<u32> {
        (s.len() >= min && s.len() <= max && s.bytes().all(|b| b.is_ascii_digit()))
            .then(|| s.parse().ok())
            .flatten()
    }
    let three = |sep: char| -> Option<(&str, &str, &str)> {
        let mut it = raw.split(sep);
        let t = (it.next()?, it.next()?, it.next()?);
        it.next().is_none().then_some(t)
    };
    if let Some((a, b, y)) = three('/') {
        return Some(DateShape::Slash(
            num(a, 1, 2)?,
            num(b, 1, 2)?,
            num(y, 4, 4)? as i32,
        ));
    }
    if let Some((d, m, y)) = three('.') {
        return Some(DateShape::Dotted(
            num(d, 1, 2)?,
            num(m, 1, 2)?,
            num(y, 4, 4)? as i32,
        ));
    }
    if let Some((y, m, d)) = three('-') {
        return Some(DateShape::Iso(
            num(y, 4, 4)? as i32,
            num(m, 1, 2)?,
            num(d, 1, 2)?,
        ));
    }
    None
}

/// Une forme `a/b` porte-t-elle un indice d'ordre ? Ne compte que si les deux
/// nombres sont dans `1..=31` et qu'au moins l'un est ≤ 12 (sinon la date est
/// fausse, non indicative).
fn slash_hint(a: u32, b: u32) -> Option<DateOrder> {
    let plausible = (1..=31).contains(&a) && (1..=31).contains(&b) && (a <= 12 || b <= 12);
    if !plausible {
        None
    } else if a > 12 {
        Some(DateOrder::DayFirst)
    } else if b > 12 {
        Some(DateOrder::MonthFirst)
    } else {
        None
    }
}

/// Date d'une forme, sous l'ordre décidé ; `None` si elle n'existe pas au
/// calendrier ou sort de la plage d'années.
fn shape_to_date(shape: DateShape, order: DateOrder) -> Option<NaiveDate> {
    let (y, m, d) = match (shape, order) {
        (DateShape::Slash(a, b, y), DateOrder::DayFirst) => (y, b, a),
        (DateShape::Slash(a, b, y), DateOrder::MonthFirst) => (y, a, b),
        (DateShape::Dotted(d, m, y), _) => (y, m, d),
        (DateShape::Iso(y, m, d), _) => (y, m, d),
        (DateShape::Slash(..), _) => return None,
    };
    NaiveDate::from_ymd_opt(y, m, d).filter(|date| crate::annee_recevable(*date))
}

/// Décide le format de date du fichier.
///
/// Le format (barre oblique, point, ISO) est celui de la première date de forme
/// reconnue ; pour la barre oblique, l'ordre jour/mois se décide sur toutes les
/// dates à barre oblique du fichier. `Ok(None)` : aucune date reconnaissable
/// (chaque ligne sera en erreur `DATE`).
fn decide_date_order(shapes: &[Option<DateShape>]) -> Result<Option<DateOrder>, PaypalCsvError> {
    let Some(first) = shapes.iter().flatten().next() else {
        return Ok(None);
    };
    match first {
        DateShape::Dotted(..) => Ok(Some(DateOrder::Dotted)),
        DateShape::Iso(..) => Ok(Some(DateOrder::Iso)),
        DateShape::Slash(..) => {
            let (mut day_first, mut month_first) = (false, false);
            for shape in shapes.iter().flatten() {
                if let DateShape::Slash(a, b, _) = shape {
                    match slash_hint(*a, *b) {
                        Some(DateOrder::DayFirst) => day_first = true,
                        Some(DateOrder::MonthFirst) => month_first = true,
                        _ => {}
                    }
                }
            }
            match (day_first, month_first) {
                (true, true) => Err(PaypalCsvError::DateOrderConflict),
                (true, false) => Ok(Some(DateOrder::DayFirst)),
                (false, true) => Ok(Some(DateOrder::MonthFirst)),
                (false, false) => Err(PaypalCsvError::DateOrderAmbiguous),
            }
        }
    }
}

/// Lit le CSV PayPal « Balance affecting transactions ».
///
/// Décodage UTF-8 (BOM retranché) ou ISO-8859-1, en-tête par noms, contrôles
/// de chaque ligne ; une seule erreur refuse le fichier entier. Voir le module
/// pour les règles.
///
/// # Erreurs
///
/// [`PaypalCsvError`], dont [`PaypalCsvError::Lines`] agrège les erreurs de
/// ligne jusqu'à [`MAX_CSV_LINE_ERRORS`].
pub fn parse_paypal_csv(bytes: &[u8]) -> Result<PaypalCsv, PaypalCsvError> {
    let (encoding, bom_len) = detect_encoding(bytes).map_err(|e| match e {
        CsvError::EmptyFile { .. } => PaypalCsvError::Empty,
        other => PaypalCsvError::Encoding(other),
    })?;
    let text = decode_bytes(&bytes[bom_len..], encoding).map_err(PaypalCsvError::Encoding)?;

    // `flexible(true)` : le nombre de cellules se contrôle ici, ce qui permet
    // de sauter une ligne vide (fin de fichier) avant de la compter fausse.
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(text.as_bytes());

    let mut errors = Errors::default();
    let mut header: Option<csv::StringRecord> = None;
    let mut records: Vec<(usize, csv::StringRecord)> = Vec::new();
    for result in reader.records() {
        match result {
            Ok(record) => {
                if record.iter().all(|c| c.trim().is_empty()) {
                    continue;
                }
                let line = record.position().map_or(0, |p| p.line() as usize);
                match &header {
                    None => header = Some(record),
                    Some(h) if record.len() != h.len() => errors.push(PaypalLineError::new(
                        line,
                        None,
                        PaypalLineErrorCode::FieldCount,
                        Some(&record.len().to_string()),
                    )),
                    Some(_) => records.push((line, record)),
                }
            }
            Err(e) => {
                let line = e.position().map_or(0, |p| p.line() as usize);
                errors.push(PaypalLineError::new(
                    line,
                    None,
                    PaypalLineErrorCode::FieldCount,
                    Some(&e.to_string()),
                ));
            }
        }
    }

    let Some(header) = header else {
        return Err(PaypalCsvError::Empty);
    };
    let columns = ColumnsByName::locate(header.iter(), &REQUIRED_COLUMNS, &OPTIONAL_COLUMNS)
        .map_err(|e| match e {
            HeaderByNameError::Missing(names) => PaypalCsvError::MissingColumns(names),
            HeaderByNameError::Duplicate(name) => PaypalCsvError::DuplicateColumn(name),
        })?;
    if records.is_empty() && errors.0.is_empty() {
        return Err(PaypalCsvError::Empty);
    }

    let shapes: Vec<Option<DateShape>> = records
        .iter()
        .map(|(_, r)| columns.cell(r, COL_DATE).and_then(date_shape))
        .collect();
    let date_order = decide_date_order(&shapes)?;
    let reference_shape = shapes.iter().flatten().next().copied();

    let mut rows = Vec::with_capacity(records.len());
    for ((line, record), shape) in records.iter().zip(&shapes) {
        if let Some(row) = read_row(
            *line,
            record,
            &columns,
            *shape,
            reference_shape,
            date_order,
            &mut errors,
        ) {
            rows.push(row);
        }
    }

    if !errors.0.is_empty() {
        errors.0.sort_by_key(|e| e.line);
        return Err(PaypalCsvError::Lines(errors.0));
    }
    // `date_order` est `Some` dès qu'une ligne a une date reconnue ; sans
    // aucune, chaque ligne est en erreur et l'on est sorti ci-dessus.
    let date_order = date_order.ok_or(PaypalCsvError::Empty)?;
    Ok(PaypalCsv { rows, date_order })
}

/// Lit et contrôle une ligne ; rend `None` (erreurs poussées) si elle est
/// fautive.
fn read_row(
    line: usize,
    record: &csv::StringRecord,
    columns: &ColumnsByName,
    shape: Option<DateShape>,
    reference_shape: Option<DateShape>,
    date_order: Option<DateOrder>,
    errors: &mut Errors,
) -> Option<PaypalRow> {
    // Chaque champ fautif vaut `None` et pousse son erreur ; `incoherent`
    // marque une ligne aux champs lisibles mais contradictoires.
    let mut push = |field, code, value: Option<&str>| {
        errors.push(PaypalLineError::new(line, Some(field), code, value));
    };
    let cell = |name| columns.cell(record, name);
    let text = |name| cell(name).map(str::to_string);

    let raw_date = cell(COL_DATE);
    let date = match (shape, reference_shape, date_order) {
        (Some(s), Some(r), Some(order)) if s.same_format(&r) => shape_to_date(s, order),
        _ => None,
    };
    if date.is_none() {
        push(COL_DATE, PaypalLineErrorCode::Date, raw_date);
    }

    let kind = text(COL_TYPE);
    if kind.is_none() {
        push(COL_TYPE, PaypalLineErrorCode::Required, None);
    }
    let status = text(COL_STATUS);
    if status.is_none() {
        push(COL_STATUS, PaypalLineErrorCode::Required, None);
    }

    let raw_currency = cell(COL_CURRENCY);
    let currency = raw_currency
        .filter(|c| c.len() == 3 && c.bytes().all(|b| b.is_ascii_alphabetic()))
        .map(str::to_ascii_uppercase);
    if currency.is_none() {
        push(COL_CURRENCY, PaypalLineErrorCode::Currency, raw_currency);
    }

    let raw_id = cell(COL_TRANSACTION_ID);
    let transaction_id = raw_id.map(str::to_ascii_uppercase).filter(|id| {
        id.bytes()
            .all(|b| b.is_ascii_digit() || b.is_ascii_uppercase())
    });
    if transaction_id.is_none() {
        push(
            COL_TRANSACTION_ID,
            PaypalLineErrorCode::TransactionId,
            raw_id,
        );
    }

    let raw_impact = cell(COL_BALANCE_IMPACT);
    let balance_impact = match raw_impact.map(str::to_ascii_lowercase).as_deref() {
        Some("credit") => Some(BalanceImpact::Credit),
        Some("debit") => Some(BalanceImpact::Debit),
        Some("memo") => Some(BalanceImpact::Memo),
        _ => None,
    };
    if balance_impact.is_none() {
        push(
            COL_BALANCE_IMPACT,
            PaypalLineErrorCode::BalanceImpact,
            raw_impact,
        );
    }

    let mut amount = |name: &'static str| -> Option<Decimal> {
        let raw = cell(name);
        let value = raw.and_then(parse_dot_decimal_amount);
        if value.is_none() {
            push(name, PaypalLineErrorCode::Amount, raw);
        }
        value
    };
    let gross = amount(COL_GROSS);
    let fee = amount(COL_FEE);
    let net = amount(COL_NET);
    // `Balance` est facultatif : vide, il vaut `None` sans erreur.
    let raw_balance = cell(COL_BALANCE);
    let balance = raw_balance.map(|_| amount(COL_BALANCE));
    let balance_ok = !matches!(balance, Some(None));

    let mut incoherent = false;
    if let (Some(gross), Some(fee), Some(net)) = (gross, fee, net) {
        if net != gross + fee {
            incoherent = true;
            push(COL_NET, PaypalLineErrorCode::NetMismatch, cell(COL_NET));
        }
        let sign_mismatch = match balance_impact {
            Some(BalanceImpact::Credit) => gross < Decimal::ZERO,
            Some(BalanceImpact::Debit) => gross > Decimal::ZERO,
            _ => false,
        };
        if sign_mismatch {
            incoherent = true;
            push(
                COL_GROSS,
                PaypalLineErrorCode::SignMismatch,
                cell(COL_GROSS),
            );
        }
    }
    if incoherent || !balance_ok {
        return None;
    }

    Some(PaypalRow {
        line,
        date: date?,
        time: text(COL_TIME),
        time_zone: text(COL_TIME_ZONE),
        name: text(COL_NAME),
        kind: kind?,
        status: status?,
        currency: currency?,
        gross: gross?,
        fee: fee?,
        net: net?,
        from_email: text(COL_FROM_EMAIL),
        to_email: text(COL_TO_EMAIL),
        transaction_id: transaction_id?,
        reference_txn_id: cell(COL_REFERENCE_TXN_ID).map(str::to_ascii_uppercase),
        balance_impact: balance_impact?,
        balance: balance.flatten(),
        item_title: text(COL_ITEM_TITLE),
        subject: text(COL_SUBJECT),
        note: text(COL_NOTE),
        invoice_number: text(COL_INVOICE_NUMBER),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    /// En-tête minimal : les requises, dans l'ordre de [`REQUIRED_COLUMNS`].
    fn header() -> String {
        REQUIRED_COLUMNS.join(",")
    }

    /// Une ligne minimale aux valeurs par défaut, cellules surchargées par nom.
    fn line(over: &[(&str, &str)]) -> String {
        let defaults = [
            (COL_DATE, "15/01/2020"),
            (COL_NAME, "Nom Fictif"),
            (COL_TYPE, "Website Payment"),
            (COL_STATUS, "Completed"),
            (COL_CURRENCY, "EUR"),
            (COL_GROSS, "10.00"),
            (COL_FEE, "-0.50"),
            (COL_NET, "9.50"),
            (COL_FROM_EMAIL, "a@example.invalid"),
            (COL_TO_EMAIL, "b@example.invalid"),
            (COL_TRANSACTION_ID, "0TEST00000000001A"),
            (COL_REFERENCE_TXN_ID, ""),
            (COL_BALANCE_IMPACT, "Credit"),
        ];
        defaults
            .iter()
            .map(|(k, v)| {
                let v = over.iter().find(|(n, _)| n == k).map_or(*v, |(_, o)| *o);
                format!("\"{v}\"")
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    fn csv(lines: &[String]) -> Vec<u8> {
        let mut s = header();
        for l in lines {
            s.push('\n');
            s.push_str(l);
        }
        s.into_bytes()
    }

    fn line_errors(bytes: &[u8]) -> Vec<PaypalLineError> {
        match parse_paypal_csv(bytes) {
            Err(PaypalCsvError::Lines(e)) => e,
            other => panic!("erreurs de ligne attendues, obtenu {other:?}"),
        }
    }

    #[test]
    fn reads_a_minimal_line() {
        let out = parse_paypal_csv(&csv(&[line(&[])])).unwrap();
        assert_eq!(out.date_order, DateOrder::DayFirst);
        let r = &out.rows[0];
        assert_eq!(r.line, 2);
        assert_eq!(r.date, NaiveDate::from_ymd_opt(2020, 1, 15).unwrap());
        assert_eq!(
            (r.gross, r.fee, r.net),
            (dec!(10.00), dec!(-0.50), dec!(9.50))
        );
        assert_eq!(r.balance_impact, BalanceImpact::Credit);
        assert_eq!(r.reference_txn_id, None);
        assert_eq!(r.time, None, "facultative absente");
        assert_eq!(r.balance, None);
    }

    #[test]
    fn date_shapes() {
        assert_eq!(date_shape("5/3/2025"), Some(DateShape::Slash(5, 3, 2025)));
        assert_eq!(
            date_shape("05.03.2025"),
            Some(DateShape::Dotted(5, 3, 2025))
        );
        assert_eq!(date_shape("2025-03-05"), Some(DateShape::Iso(2025, 3, 5)));
        for bad in [
            "5/3/25",
            "005/3/2025",
            "5/3/2025/1",
            "5-3-2025",
            "a/b/cccc",
            "",
            "５/3/2025",
        ] {
            assert_eq!(date_shape(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn date_order_is_decided_on_the_whole_file() {
        // Une seule ligne au jour > 12 suffit à décider pour toutes.
        let out = parse_paypal_csv(&csv(&[
            line(&[(COL_DATE, "03/01/2020")]),
            line(&[(COL_DATE, "13/01/2020")]),
        ]))
        .unwrap();
        assert_eq!(out.date_order, DateOrder::DayFirst);
        assert_eq!(
            out.rows[0].date,
            NaiveDate::from_ymd_opt(2020, 1, 3).unwrap()
        );

        let out = parse_paypal_csv(&csv(&[
            line(&[(COL_DATE, "03/01/2020")]),
            line(&[(COL_DATE, "01/13/2020")]),
        ]))
        .unwrap();
        assert_eq!(out.date_order, DateOrder::MonthFirst);
        assert_eq!(
            out.rows[0].date,
            NaiveDate::from_ymd_opt(2020, 3, 1).unwrap()
        );
    }

    #[test]
    fn date_order_conflict_and_ambiguity_refuse_the_file() {
        let err = parse_paypal_csv(&csv(&[
            line(&[(COL_DATE, "13/01/2020")]),
            line(&[(COL_DATE, "01/13/2020")]),
        ]))
        .unwrap_err();
        assert_eq!(err, PaypalCsvError::DateOrderConflict);
        assert_eq!(err.code(), "PAYPAL_CSV_DATE_ORDER_CONFLICT");

        let err = parse_paypal_csv(&csv(&[line(&[(COL_DATE, "03/01/2020")])])).unwrap_err();
        assert_eq!(err, PaypalCsvError::DateOrderAmbiguous);
        assert_eq!(err.code(), "PAYPAL_CSV_DATE_ORDER_AMBIGUOUS");
    }

    #[test]
    fn impossible_slash_date_is_a_line_error_not_a_hint() {
        // 13/13 ne décide rien ; la ligne au 31/01 décide jour en tête.
        let e = line_errors(&csv(&[
            line(&[(COL_DATE, "31/01/2020")]),
            line(&[(COL_DATE, "13/13/2020")]),
            line(&[(COL_DATE, "31/02/2020")]),
        ]));
        assert_eq!(e.len(), 2);
        assert!(
            e.iter()
                .all(|e| e.code == PaypalLineErrorCode::Date && e.field == Some(COL_DATE))
        );
        assert_eq!((e[0].line, e[1].line), (3, 4));
    }

    #[test]
    fn dotted_and_iso_dates_and_mixed_formats() {
        let out = parse_paypal_csv(&csv(&[line(&[(COL_DATE, "5.3.2020")])])).unwrap();
        assert_eq!(out.date_order, DateOrder::Dotted);
        assert_eq!(
            out.rows[0].date,
            NaiveDate::from_ymd_opt(2020, 3, 5).unwrap()
        );
        let out = parse_paypal_csv(&csv(&[line(&[(COL_DATE, "2020-03-05")])])).unwrap();
        assert_eq!(out.date_order, DateOrder::Iso);

        let e = line_errors(&csv(&[
            line(&[(COL_DATE, "15.01.2020")]),
            line(&[(COL_DATE, "15/01/2020")]),
        ]));
        assert_eq!(e.len(), 1);
        assert_eq!((e[0].line, e[0].code), (3, PaypalLineErrorCode::Date));
        assert_eq!(e[0].value.as_deref(), Some("15/01/2020"));
    }

    #[test]
    fn year_out_of_range_is_a_line_error() {
        let e = line_errors(&csv(&[
            line(&[(COL_DATE, "2020-01-15")]),
            line(&[(COL_DATE, "0999-01-15")]),
        ]));
        assert_eq!((e[0].line, e[0].code), (3, PaypalLineErrorCode::Date));
    }

    #[test]
    fn net_and_sign_mismatches() {
        let e = line_errors(&csv(&[line(&[(COL_NET, "9.40")])]));
        assert_eq!(e.len(), 1);
        assert_eq!(
            (e[0].code, e[0].field, e[0].value.as_deref()),
            (
                PaypalLineErrorCode::NetMismatch,
                Some(COL_NET),
                Some("9.40")
            )
        );

        let e = line_errors(&csv(&[line(&[
            (COL_GROSS, "-10.00"),
            (COL_FEE, "0.00"),
            (COL_NET, "-10.00"),
        ])]));
        assert_eq!(
            (e[0].code, e[0].field),
            (PaypalLineErrorCode::SignMismatch, Some(COL_GROSS))
        );

        let e = line_errors(&csv(&[line(&[(COL_BALANCE_IMPACT, "Debit")])]));
        assert_eq!(e[0].code, PaypalLineErrorCode::SignMismatch);

        // Memo : aucun contrôle de signe.
        assert!(
            parse_paypal_csv(&csv(&[line(&[
                (COL_BALANCE_IMPACT, "memo"),
                (COL_GROSS, "-10.00"),
                (COL_FEE, "0.00"),
                (COL_NET, "-10.00"),
            ])]))
            .is_ok()
        );
    }

    #[test]
    fn refund_with_positive_fee_is_accepted() {
        let out = parse_paypal_csv(&csv(&[line(&[
            (COL_TYPE, "Payment Refund"),
            (COL_BALANCE_IMPACT, "Debit"),
            (COL_GROSS, "-30.00"),
            (COL_FEE, "0.96"),
            (COL_NET, "-29.04"),
        ])]))
        .unwrap();
        assert_eq!(out.rows[0].fee, dec!(0.96));
    }

    #[test]
    fn amounts_out_of_the_closed_form_are_refused() {
        for raw in ["1,50", "1.500,00", "1 500.00", "+10.00", ""] {
            let e = line_errors(&csv(&[line(&[(COL_GROSS, raw)])]));
            assert_eq!(e[0].code, PaypalLineErrorCode::Amount, "{raw:?}");
            assert_eq!(e[0].field, Some(COL_GROSS));
        }
        let out = parse_paypal_csv(&csv(&[line(&[
            (COL_GROSS, "1,234.56"),
            (COL_FEE, "-34.56"),
            (COL_NET, "1,200.00"),
        ])]))
        .unwrap();
        assert_eq!(out.rows[0].gross, dec!(1234.56));
    }

    #[test]
    fn currency_id_impact_and_required_text() {
        let e = line_errors(&csv(&[line(&[(COL_CURRENCY, "EU")])]));
        assert_eq!(
            (e[0].code, e[0].value.as_deref()),
            (PaypalLineErrorCode::Currency, Some("EU"))
        );
        let out = parse_paypal_csv(&csv(&[line(&[(COL_CURRENCY, "usd")])])).unwrap();
        assert_eq!(out.rows[0].currency, "USD");

        for bad in ["0TEST:FEE", "", "0TEST-1"] {
            let e = line_errors(&csv(&[line(&[(COL_TRANSACTION_ID, bad)])]));
            assert_eq!(e[0].code, PaypalLineErrorCode::TransactionId, "{bad:?}");
        }
        let out = parse_paypal_csv(&csv(&[line(&[
            (COL_TRANSACTION_ID, " 0test1a "),
            (COL_REFERENCE_TXN_ID, "0ref2b"),
        ])]))
        .unwrap();
        assert_eq!(out.rows[0].transaction_id, "0TEST1A");
        assert_eq!(out.rows[0].reference_txn_id.as_deref(), Some("0REF2B"));

        let e = line_errors(&csv(&[line(&[(COL_BALANCE_IMPACT, "Neutral")])]));
        assert_eq!(e[0].code, PaypalLineErrorCode::BalanceImpact);

        let e = line_errors(&csv(&[line(&[(COL_TYPE, " "), (COL_STATUS, "")])]));
        assert_eq!(e.len(), 2);
        assert!(e.iter().all(|e| e.code == PaypalLineErrorCode::Required));
    }

    #[test]
    fn errors_of_one_line_are_all_reported() {
        let e = line_errors(&csv(&[line(&[(COL_CURRENCY, "E"), (COL_FEE, "x")])]));
        let codes: Vec<_> = e.iter().map(|e| e.code).collect();
        assert_eq!(
            codes,
            vec![PaypalLineErrorCode::Currency, PaypalLineErrorCode::Amount]
        );
    }

    #[test]
    fn error_value_is_truncated() {
        let long = "X".repeat(MAX_CSV_LINE_ERROR_VALUE_CHARS + 5);
        let e = line_errors(&csv(&[line(&[(COL_CURRENCY, &long)])]));
        let v = e[0].value.as_deref().unwrap();
        assert_eq!(v.chars().count(), MAX_CSV_LINE_ERROR_VALUE_CHARS + 1);
        assert!(v.ends_with('…'));
    }

    #[test]
    fn error_collection_is_capped() {
        let lines: Vec<String> = (0..MAX_CSV_LINE_ERRORS + 10)
            .map(|_| line(&[(COL_CURRENCY, "E")]))
            .collect();
        let e = line_errors(&csv(&lines));
        assert_eq!(e.len(), MAX_CSV_LINE_ERRORS);
    }

    #[test]
    fn empty_inputs() {
        assert_eq!(parse_paypal_csv(b"").unwrap_err(), PaypalCsvError::Empty);
        assert_eq!(
            parse_paypal_csv(b"\xEF\xBB\xBF").unwrap_err(),
            PaypalCsvError::Empty
        );
        assert_eq!(
            parse_paypal_csv(header().as_bytes()).unwrap_err(),
            PaypalCsvError::Empty
        );
        let mut with_blank = csv(&[line(&[])]);
        with_blank.extend_from_slice(b"\n\n");
        assert_eq!(parse_paypal_csv(&with_blank).unwrap().rows.len(), 1);
        assert_eq!(PaypalCsvError::Empty.code(), "PAYPAL_CSV_EMPTY");
    }

    #[test]
    fn every_variant_has_its_code() {
        let all = [
            PaypalCsvError::Empty,
            PaypalCsvError::Encoding(CsvError::UnsupportedEncoding { detected: None }),
            PaypalCsvError::MissingColumns(vec![]),
            PaypalCsvError::DuplicateColumn(String::new()),
            PaypalCsvError::DateOrderConflict,
            PaypalCsvError::DateOrderAmbiguous,
            PaypalCsvError::Lines(vec![]),
        ];
        let codes: Vec<_> = all.iter().map(PaypalCsvError::code).collect();
        assert_eq!(
            codes,
            [
                "PAYPAL_CSV_EMPTY",
                "PAYPAL_CSV_ENCODING",
                "PAYPAL_CSV_MISSING_COLUMNS",
                "PAYPAL_CSV_DUPLICATE_COLUMN",
                "PAYPAL_CSV_DATE_ORDER_CONFLICT",
                "PAYPAL_CSV_DATE_ORDER_AMBIGUOUS",
                "PAYPAL_CSV_LINES",
            ]
        );
    }

    #[test]
    fn line_error_codes_serialize_as_their_str() {
        for code in [
            PaypalLineErrorCode::FieldCount,
            PaypalLineErrorCode::Date,
            PaypalLineErrorCode::Amount,
            PaypalLineErrorCode::NetMismatch,
            PaypalLineErrorCode::SignMismatch,
            PaypalLineErrorCode::Currency,
            PaypalLineErrorCode::TransactionId,
            PaypalLineErrorCode::BalanceImpact,
            PaypalLineErrorCode::Required,
        ] {
            assert_eq!(serde_json::to_value(code).unwrap(), code.as_str());
        }
    }
}
