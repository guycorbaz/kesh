//! Lecteur du CSV Stripe « Itemized payout reconciliation » (Story 40-7a).
//!
//! Stripe exporte, pour une période, le détail des versements : une ligne par
//! transaction de solde (charge, frais, remboursement, retour de versement…),
//! avec le versement qui la porte, la devise, le brut, les frais, le net et
//! l'objet Stripe lié. Ce module le lit **par noms de colonnes**
//! ([`crate::csv::ColumnsByName`]), contrôle chaque ligne et rend des
//! [`StripePayoutRow`]. Il ne filtre rien : le classement par catégorie, les
//! doublons et le rattachement aux ventes sont l'affaire de l'import et du
//! rapprochement (Stories 40-7b, 40-7c).
//!
//! Garanties, chacune refusant le fichier **entier** (pas d'import partiel) :
//!
//! - les colonnes requises sont toutes là, sinon
//!   [`StripeCsvError::MissingColumns`] les nomme toutes ; l'objet lié
//!   (`source_id` ou `charge_id`) est exigé, sans quoi aucune charge ne se
//!   rattache à sa vente ;
//! - les montants sont à forme fermée, **sans** séparateur de milliers
//!   ([`parse_amount`]) : un rapport demandé à virgule décimale (`12,50`) est
//!   refusé, jamais lu `1250` ;
//! - `net = gross − fee` (convention Stripe : frais positifs quand ils sont
//!   prélevés) ;
//! - un versement n'a qu'une devise.
//!
//! La date gardée est celle du fuseau choisi à l'export, sans conversion.
//!
//! Fonction pure, sans I/O, sans dépendance interne au workspace.

use std::collections::HashMap;

use chrono::{NaiveDate, NaiveTime};
use rust_decimal::Decimal;

use crate::csv::by_name::normalize;
use crate::csv::encoding::{decode_bytes, detect_encoding};
use crate::csv::{ColumnsByName, HeaderByNameError, parse_dot_decimal_amount};
use crate::error::{CsvError, MAX_CSV_LINE_ERROR_VALUE_CHARS, MAX_CSV_LINE_ERRORS};

/// Colonne `automatic_payout_id`.
pub const COL_PAYOUT_ID: &str = "automatic_payout_id";
/// Colonne `automatic_payout_effective_at`.
pub const COL_PAYOUT_DATE: &str = "automatic_payout_effective_at";
/// Colonne `balance_transaction_id`.
pub const COL_BALANCE_TXN_ID: &str = "balance_transaction_id";
/// Colonne `created` (rapport du tableau de bord, fuseau choisi).
pub const COL_CREATED: &str = "created";
/// Colonne `created_utc` (rapport de l'API, UTC).
pub const COL_CREATED_UTC: &str = "created_utc";
/// Colonne `currency`.
pub const COL_CURRENCY: &str = "currency";
/// Colonne `gross`.
pub const COL_GROSS: &str = "gross";
/// Colonne `fee`.
pub const COL_FEE: &str = "fee";
/// Colonne `net`.
pub const COL_NET: &str = "net";
/// Colonne `reporting_category`.
pub const COL_CATEGORY: &str = "reporting_category";
/// Colonne `source_id`.
pub const COL_SOURCE_ID: &str = "source_id";
/// Colonne `charge_id`.
pub const COL_CHARGE_ID: &str = "charge_id";
/// Colonne `available_on`.
pub const COL_AVAILABLE_ON: &str = "available_on";
/// Colonne `available_on_utc`.
pub const COL_AVAILABLE_ON_UTC: &str = "available_on_utc";
/// Colonne `description`.
pub const COL_DESCRIPTION: &str = "description";
/// Colonne `customer_facing_amount` (lue, non contrôlée).
pub const COL_CUSTOMER_AMOUNT: &str = "customer_facing_amount";
/// Colonne `customer_facing_currency` (lue, non contrôlée).
pub const COL_CUSTOMER_CURRENCY: &str = "customer_facing_currency";

/// Nom, dans un refus [`StripeCsvError::MissingColumns`], de l'absence des deux
/// colonnes d'objet lié.
pub const LINKED_OBJECT_COLUMNS: &str = "source_id|charge_id";
/// Nom, dans un refus [`StripeCsvError::MissingColumns`], de l'absence des deux
/// graphies de la date de création.
pub const CREATED_COLUMNS: &str = "created|created_utc";

/// Requises à nom unique, dans l'ordre où un refus les nomme.
const REQUIRED_PLAIN: [&str; 8] = [
    COL_PAYOUT_ID,
    COL_PAYOUT_DATE,
    COL_BALANCE_TXN_ID,
    COL_CURRENCY,
    COL_GROSS,
    COL_FEE,
    COL_NET,
    COL_CATEGORY,
];

/// Lues si présentes (absentes : cellule vide). Les graphies alternatives
/// (`created`, `created_utc`, `source_id`, `charge_id`) y figurent : leur
/// exigence « l'une ou l'autre » se contrôle ici, non dans l'aide commune.
const OPTIONAL: [&str; 9] = [
    COL_CREATED,
    COL_CREATED_UTC,
    COL_SOURCE_ID,
    COL_CHARGE_ID,
    COL_AVAILABLE_ON,
    COL_AVAILABLE_ON_UTC,
    COL_DESCRIPTION,
    COL_CUSTOMER_AMOUNT,
    COL_CUSTOMER_CURRENCY,
];

/// Une ligne du CSV Stripe, contrôlée.
#[derive(Debug, Clone, PartialEq)]
pub struct StripePayoutRow {
    /// Numéro de ligne dans le fichier (1 = l'en-tête).
    pub line: usize,
    /// Identifiant du versement (`automatic_payout_id`).
    pub payout_id: String,
    /// Date effective du versement, dans le fuseau de l'export.
    pub payout_date: NaiveDate,
    /// Identifiant de la transaction de solde.
    pub balance_transaction_id: String,
    /// Date de la transaction, dans le fuseau de l'export.
    pub date: NaiveDate,
    /// Date de disponibilité des fonds, si écrite.
    pub available_on: Option<NaiveDate>,
    /// Code de devise, trois lettres ASCII majuscules.
    pub currency: String,
    /// Montant brut.
    pub gross: Decimal,
    /// Frais (positifs quand prélevés, négatifs quand Stripe les rend).
    pub fee: Decimal,
    /// Net = brut − frais (contrôlé).
    pub net: Decimal,
    /// Catégorie de rapport, en minuscules (`charge`, `fee`, `refund`,
    /// `payout_reversal`…), telle quelle.
    pub category: String,
    /// Description, si écrite.
    pub description: Option<String>,
    /// Identifiant de l'objet lié (`ch_…`, `re_…`, `py_…`), s'il est écrit.
    pub source_id: Option<String>,
    /// Identifiant de la charge liée, s'il est écrit.
    pub charge_id: Option<String>,
}

/// Le fichier lu : ses lignes, dans l'ordre du fichier.
#[derive(Debug, Clone, PartialEq)]
pub struct StripePayoutCsv {
    /// Les lignes de données, sans filtre.
    pub rows: Vec<StripePayoutRow>,
}

/// Code stable d'une erreur de ligne (sérialisé en `SCREAMING_SNAKE_CASE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StripeLineErrorCode {
    /// La ligne n'a pas autant de cellules que l'en-tête.
    FieldCount,
    /// Identifiant (versement ou transaction de solde) ou catégorie vide, ou
    /// identifiant hors `[A-Za-z0-9_]`.
    Id,
    /// Date illisible, hors format ou hors plage d'années.
    Date,
    /// Devise autre que trois lettres ASCII.
    Currency,
    /// Montant hors de la forme fermée (ou vide).
    Amount,
    /// `net ≠ gross − fee`.
    NetMismatch,
    /// Un même versement porte deux devises (signalé sur la seconde).
    PayoutCurrencyMixed,
}

impl StripeLineErrorCode {
    /// Graphie stable du code, identique à sa sérialisation serde.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FieldCount => "FIELD_COUNT",
            Self::Id => "ID",
            Self::Date => "DATE",
            Self::Currency => "CURRENCY",
            Self::Amount => "AMOUNT",
            Self::NetMismatch => "NET_MISMATCH",
            Self::PayoutCurrencyMixed => "PAYOUT_CURRENCY_MIXED",
        }
    }
}

/// Une erreur de ligne, nommant la ligne, le champ et la valeur fautive.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct StripeLineError {
    /// Numéro de ligne dans le fichier (1 = l'en-tête).
    pub line: usize,
    /// Nom de la colonne en cause ; `None` pour [`StripeLineErrorCode::FieldCount`].
    pub field: Option<&'static str>,
    /// Code stable.
    pub code: StripeLineErrorCode,
    /// Valeur fautive, tronquée à [`MAX_CSV_LINE_ERROR_VALUE_CHARS`]
    /// caractères (suffixe `…`).
    pub value: Option<String>,
}

impl StripeLineError {
    fn new(
        line: usize,
        field: Option<&'static str>,
        code: StripeLineErrorCode,
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

/// Refus du fichier Stripe. Type fermé : chaque variante a un code stable
/// ([`StripeCsvError::code`]).
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum StripeCsvError {
    /// Fichier vide, ou en-tête sans ligne de données.
    #[error("fichier Stripe vide")]
    Empty,
    /// Encodage non reconnu ou décodage impossible.
    #[error("encodage du fichier Stripe : {0}")]
    Encoding(CsvError),
    /// Colonnes requises absentes, toutes, dans l'ordre de l'en-tête attendu ;
    /// l'objet lié s'y nomme [`LINKED_OBJECT_COLUMNS`].
    #[error("colonnes Stripe requises absentes : {}", .0.join(", "))]
    MissingColumns(Vec<String>),
    /// Une colonne lue apparaît deux fois dans l'en-tête, ou `created` et
    /// `created_utc` ensemble.
    #[error("colonne Stripe en double : {0}")]
    DuplicateColumn(String),
    /// Erreurs de ligne, au plus [`MAX_CSV_LINE_ERRORS`].
    #[error("{} ligne(s) Stripe en erreur", .0.len())]
    Lines(Vec<StripeLineError>),
}

impl StripeCsvError {
    /// Code stable de la variante (placé par l'import dans `details`).
    pub fn code(&self) -> &'static str {
        match self {
            Self::Empty => "STRIPE_CSV_EMPTY",
            Self::Encoding(_) => "STRIPE_CSV_ENCODING",
            Self::MissingColumns(_) => "STRIPE_CSV_MISSING_COLUMNS",
            Self::DuplicateColumn(_) => "STRIPE_CSV_DUPLICATE_COLUMN",
            Self::Lines(_) => "STRIPE_CSV_LINES",
        }
    }
}

/// Collecte plafonnée des erreurs de ligne.
#[derive(Default)]
struct Errors(Vec<StripeLineError>);

impl Errors {
    fn push(&mut self, e: StripeLineError) {
        if self.0.len() < MAX_CSV_LINE_ERRORS {
            self.0.push(e);
        }
    }
}

/// Lit un montant Stripe : forme fermée de [`parse_dot_decimal_amount`] **sans**
/// virgule des milliers (Stripe n'en écrit jamais) — `12`, `-3.2`, `1234.56`.
/// `12,50` est refusé, jamais lu `1250`.
fn parse_amount(raw: &str) -> Option<Decimal> {
    if raw.contains(',') {
        return None;
    }
    parse_dot_decimal_amount(raw)
}

/// Date d'une cellule Stripe : `AAAA-MM-JJ` ou `AAAA-MM-JJ HH:MM:SS`. Seule la
/// date est gardée, sans conversion de fuseau.
fn parse_date(raw: &str) -> Option<NaiveDate> {
    let raw = raw.trim();
    let (day, time) = match raw.split_once(' ') {
        Some((d, t)) => (d, Some(t)),
        None => (raw, None),
    };
    if let Some(t) = time {
        NaiveTime::parse_from_str(t.trim(), "%H:%M:%S").ok()?;
    }
    let b = day.as_bytes();
    let shape_ok = b.len() == 10
        && b.iter().enumerate().all(|(i, c)| {
            if i == 4 || i == 7 {
                *c == b'-'
            } else {
                c.is_ascii_digit()
            }
        });
    if !shape_ok {
        return None;
    }
    NaiveDate::parse_from_str(day, "%Y-%m-%d")
        .ok()
        .filter(|d| crate::annee_recevable(*d))
}

/// Identifiant Stripe : non vide, `[A-Za-z0-9_]` seulement (pas de `:`, que
/// l'import accole à la clé de doublon).
fn valid_id(raw: &str) -> bool {
    !raw.is_empty() && raw.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// Les requises absentes dans l'ordre de l'en-tête attendu, graphies
/// alternatives comprises.
fn missing_columns(header: &csv::StringRecord, plain_missing: &[String]) -> Vec<String> {
    let has = |name: &str| header.iter().any(|h| normalize(h) == name);
    let mut out = Vec::new();
    for name in REQUIRED_PLAIN {
        // La date de création vient après `balance_transaction_id`.
        if name == COL_CURRENCY && !has(COL_CREATED) && !has(COL_CREATED_UTC) {
            out.push(CREATED_COLUMNS.to_string());
        }
        if plain_missing.iter().any(|m| m == name) {
            out.push(name.to_string());
        }
    }
    if !has(COL_SOURCE_ID) && !has(COL_CHARGE_ID) {
        out.push(LINKED_OBJECT_COLUMNS.to_string());
    }
    out
}

/// Lit le CSV Stripe « Itemized payout reconciliation ».
///
/// Décodage UTF-8 (BOM retranché) ou ISO-8859-1, en-tête par noms, contrôles
/// de chaque ligne ; une seule erreur refuse le fichier entier. Voir le module
/// pour les règles.
///
/// # Erreurs
///
/// [`StripeCsvError`], dont [`StripeCsvError::Lines`] agrège les erreurs de
/// ligne jusqu'à [`MAX_CSV_LINE_ERRORS`].
pub fn parse_stripe_payout_csv(bytes: &[u8]) -> Result<StripePayoutCsv, StripeCsvError> {
    let (encoding, bom_len) = detect_encoding(bytes).map_err(|e| match e {
        CsvError::EmptyFile { .. } => StripeCsvError::Empty,
        other => StripeCsvError::Encoding(other),
    })?;
    let text = decode_bytes(&bytes[bom_len..], encoding).map_err(StripeCsvError::Encoding)?;

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
                    Some(h) if record.len() != h.len() => errors.push(StripeLineError::new(
                        line,
                        None,
                        StripeLineErrorCode::FieldCount,
                        Some(&record.len().to_string()),
                    )),
                    Some(_) => records.push((line, record)),
                }
            }
            Err(e) => {
                let line = e.position().map_or(0, |p| p.line() as usize);
                errors.push(StripeLineError::new(
                    line,
                    None,
                    StripeLineErrorCode::FieldCount,
                    Some(&e.to_string()),
                ));
            }
        }
    }

    let Some(header) = header else {
        return Err(StripeCsvError::Empty);
    };
    let columns = match ColumnsByName::locate(header.iter(), &REQUIRED_PLAIN, &OPTIONAL) {
        Ok(c) => c,
        Err(HeaderByNameError::Duplicate(name)) => {
            return Err(StripeCsvError::DuplicateColumn(name));
        }
        Err(HeaderByNameError::Missing(plain)) => {
            return Err(StripeCsvError::MissingColumns(missing_columns(
                &header, &plain,
            )));
        }
    };
    // Requises « l'une ou l'autre » : l'aide commune ne les connaît pas.
    let absent = missing_columns(&header, &[]);
    if !absent.is_empty() {
        return Err(StripeCsvError::MissingColumns(absent));
    }
    if columns.has(COL_CREATED) && columns.has(COL_CREATED_UTC) {
        return Err(StripeCsvError::DuplicateColumn(COL_CREATED.to_string()));
    }
    let created_col = if columns.has(COL_CREATED) {
        COL_CREATED
    } else {
        COL_CREATED_UTC
    };
    let available_col = if columns.has(COL_AVAILABLE_ON) {
        COL_AVAILABLE_ON
    } else {
        COL_AVAILABLE_ON_UTC
    };
    if records.is_empty() && errors.0.is_empty() {
        return Err(StripeCsvError::Empty);
    }

    let mut payout_currency: HashMap<String, String> = HashMap::new();
    let mut rows = Vec::with_capacity(records.len());
    for (line, record) in &records {
        if let Some(row) = read_row(
            *line,
            record,
            &columns,
            created_col,
            available_col,
            &mut payout_currency,
            &mut errors,
        ) {
            rows.push(row);
        }
    }

    if !errors.0.is_empty() {
        errors.0.sort_by_key(|e| e.line);
        return Err(StripeCsvError::Lines(errors.0));
    }
    Ok(StripePayoutCsv { rows })
}

/// Lit et contrôle une ligne ; rend `None` (erreurs poussées) si elle est
/// fautive.
fn read_row(
    line: usize,
    record: &csv::StringRecord,
    columns: &ColumnsByName,
    created_col: &'static str,
    available_col: &'static str,
    payout_currency: &mut HashMap<String, String>,
    errors: &mut Errors,
) -> Option<StripePayoutRow> {
    let mut push = |field, code, value: Option<&str>| {
        errors.push(StripeLineError::new(line, Some(field), code, value));
    };
    let cell = |name| columns.cell(record, name);
    let text = |name| cell(name).map(str::to_string);

    let mut id = |name: &'static str| -> Option<String> {
        let raw = cell(name);
        let value = raw.filter(|v| valid_id(v)).map(str::to_string);
        if value.is_none() {
            push(name, StripeLineErrorCode::Id, raw);
        }
        value
    };
    let payout_id = id(COL_PAYOUT_ID);
    let balance_transaction_id = id(COL_BALANCE_TXN_ID);

    let mut date = |name: &'static str| -> Option<NaiveDate> {
        let raw = cell(name);
        let value = raw.and_then(parse_date);
        if value.is_none() {
            push(name, StripeLineErrorCode::Date, raw);
        }
        value
    };
    let payout_date = date(COL_PAYOUT_DATE);
    let created = date(created_col);
    // `available_on` est facultatif : vide, il vaut `None` sans erreur.
    let available_on = cell(available_col).map(|_| date(available_col));
    let available_ok = !matches!(available_on, Some(None));

    let raw_currency = cell(COL_CURRENCY);
    let currency = raw_currency
        .filter(|c| c.len() == 3 && c.bytes().all(|b| b.is_ascii_alphabetic()))
        .map(str::to_ascii_uppercase);
    let mut mixed = false;
    match (&currency, &payout_id) {
        (None, _) => push(COL_CURRENCY, StripeLineErrorCode::Currency, raw_currency),
        (Some(cur), Some(po)) => match payout_currency.get(po) {
            Some(first) if first != cur => {
                mixed = true;
                push(
                    COL_CURRENCY,
                    StripeLineErrorCode::PayoutCurrencyMixed,
                    raw_currency,
                );
            }
            Some(_) => {}
            None => {
                payout_currency.insert(po.clone(), cur.clone());
            }
        },
        (Some(_), None) => {}
    }

    let mut amount = |name: &'static str| -> Option<Decimal> {
        let raw = cell(name);
        let value = raw.and_then(parse_amount);
        if value.is_none() {
            push(name, StripeLineErrorCode::Amount, raw);
        }
        value
    };
    let gross = amount(COL_GROSS);
    let fee = amount(COL_FEE);
    let net = amount(COL_NET);

    // Catégorie vide : même code que les identifiants vides (liste fermée).
    let category = cell(COL_CATEGORY).map(str::to_lowercase);
    if category.is_none() {
        push(COL_CATEGORY, StripeLineErrorCode::Id, None);
    }

    let mut incoherent = mixed;
    // `checked_sub` : `gross - fee` panique au débordement de `Decimal`
    // (montants extrêmes admis par la forme fermée) ; un débordement ne peut
    // pas égaler `net`, c'est donc aussi une incohérence de ligne.
    if let (Some(gross), Some(fee), Some(net)) = (gross, fee, net)
        && gross.checked_sub(fee) != Some(net)
    {
        incoherent = true;
        push(COL_NET, StripeLineErrorCode::NetMismatch, cell(COL_NET));
    }
    if incoherent || !available_ok {
        return None;
    }

    Some(StripePayoutRow {
        line,
        payout_id: payout_id?,
        payout_date: payout_date?,
        balance_transaction_id: balance_transaction_id?,
        date: created?,
        available_on: available_on.flatten(),
        currency: currency?,
        gross: gross?,
        fee: fee?,
        net: net?,
        category: category?,
        description: text(COL_DESCRIPTION),
        source_id: text(COL_SOURCE_ID),
        charge_id: text(COL_CHARGE_ID),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn amount_closed_form_without_thousands() {
        assert_eq!(parse_amount("12"), Some(dec!(12)));
        assert_eq!(parse_amount(" -3.2 "), Some(dec!(-3.2)));
        assert_eq!(parse_amount("1234.56"), Some(dec!(1234.56)));
        for bad in ["12,50", "1,234.56", "1.234", "+1", "", "1 2"] {
            assert_eq!(parse_amount(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn date_forms() {
        let d = NaiveDate::from_ymd_opt(2020, 1, 29).unwrap();
        assert_eq!(parse_date("2020-01-29 10:15:04"), Some(d));
        assert_eq!(parse_date(" 2020-01-29 "), Some(d));
        for bad in [
            "",
            "2020-1-29",
            "29.01.2020",
            "2020-01-29 25:00:00",
            "2020-01-29T10:15:04",
            "1999-02-30",
            "0999-01-01",
        ] {
            assert_eq!(parse_date(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn ids() {
        assert!(valid_id("txn_1Abc"));
        for bad in ["", "a:b", "a b", "é"] {
            assert!(!valid_id(bad), "{bad:?}");
        }
    }

    #[test]
    fn every_variant_has_its_code() {
        assert_eq!(
            StripeCsvError::Encoding(CsvError::EmptyFile {
                reason: String::new()
            })
            .code(),
            "STRIPE_CSV_ENCODING"
        );
        assert_eq!(
            StripeCsvError::DuplicateColumn(String::new()).code(),
            "STRIPE_CSV_DUPLICATE_COLUMN"
        );
        assert_eq!(StripeCsvError::Lines(vec![]).code(), "STRIPE_CSV_LINES");
        for (c, s) in [
            (StripeLineErrorCode::FieldCount, "FIELD_COUNT"),
            (StripeLineErrorCode::Id, "ID"),
            (StripeLineErrorCode::Date, "DATE"),
            (StripeLineErrorCode::Currency, "CURRENCY"),
            (StripeLineErrorCode::Amount, "AMOUNT"),
            (StripeLineErrorCode::NetMismatch, "NET_MISMATCH"),
            (
                StripeLineErrorCode::PayoutCurrencyMixed,
                "PAYOUT_CURRENCY_MIXED",
            ),
        ] {
            assert_eq!(c.as_str(), s);
            assert_eq!(serde_json::to_string(&c).unwrap(), format!("\"{s}\""));
        }
    }
}
