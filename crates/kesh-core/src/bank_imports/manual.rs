//! Relevé bancaire saisi à la main — Story v016-2a (#816).
//!
//! Un relevé saisi est un import de format [`SourceFormatTag::Manual`] : même
//! entête, mêmes transactions, même circuit (rapprochement, suppression, tuile,
//! contrôle K3) qu'un relevé importé. Ce module, **sans aucune I/O**, porte :
//!
//! - le corps reçu par la route, champs en chaînes ([`ManualStatementRaw`],
//!   [`ManualLineRaw`]), et son analyse vers les types du domaine
//!   ([`parse_manual_statement`]) — toute chaîne absente, vide ou mal formée rend
//!   [`ManualStatementError::Invalid`] avec son rang, son champ et son motif ;
//! - les règles du relevé ([`validate_manual_statement`]) : période, nombre de
//!   lignes, dates dans la période, longueurs, montants, et l'**équilibre exact**
//!   `départ + crédits − débits = arrivée`, sans tolérance (C-v016-2-4) ;
//! - l'empreinte canonique ([`manual_statement_canonical_bytes`]), dont le SHA-256
//!   tient lieu de `file_hash` : chaque champ encodé par sa longueur sur quatre
//!   octets gros-boutiens suivie de ses octets UTF-8, si bien qu'aucun contenu ne
//!   peut imiter une frontière de champ (C-v016-2-6, C-v016-2-17) ;
//! - la conversion en brouillons ([`manual_statement_to_drafts`]).
//!
//! Les deux ensembles fermés de l'erreur — [`ManualStatementReason`] et
//! [`ManualStatementField`] — sont épinglés par les tests de ce module sur les
//! chaînes de la fiche (§ *Ensembles fermés*) ; la sous-fiche 2b compare sa table
//! de libellés à [`ManualStatementReason::ALL`].

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::Deserialize;

use super::{BankImportDraft, BankTransactionDraft, SourceFormatTag};
use crate::types::Money;

/// Nombre maximal de lignes d'un relevé saisi (C-v016-2-3).
pub const MANUAL_STATEMENT_MAX_LINES: usize = 500;
/// Longueur maximale du libellé d'une ligne, en caractères, après `trim`.
pub const MANUAL_DETAILS_MAX_CHARS: usize = 500;
/// Longueur maximale de la contrepartie, de la référence et du numéro du
/// relevé, en caractères après `trim` (`VARCHAR(255)` compte des caractères).
pub const MANUAL_SHORT_TEXT_MAX_CHARS: usize = 255;
/// Nombre maximal de chiffres de la partie entière d'un montant ou d'un solde
/// (`DECIMAL(18,2)` en tient seize ; treize gardent la marge des sommes).
pub const MANUAL_AMOUNT_MAX_INTEGER_DIGITS: u32 = 13;
/// Nombre maximal de décimales d'un montant, après `normalize()`.
pub const MANUAL_AMOUNT_MAX_DECIMALS: u32 = 2;
/// Préfixe de domaine de l'empreinte canonique.
pub const MANUAL_STATEMENT_HASH_DOMAIN: &str = "kesh-manual-statement-v1";

// ---------------------------------------------------------------------------
// Ensembles fermés
// ---------------------------------------------------------------------------

/// Motif d'un refus `BANK_STATEMENT_INVALID` — ensemble **fermé** (fiche
/// parente, § *Ensembles fermés*). La route sérialise [`Self::code`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ManualStatementReason {
    /// Champ absent, vide ou fait de blancs.
    Required,
    /// Chaîne qui n'est pas une date `AAAA-MM-JJ`.
    InvalidDate,
    /// Chaîne qui n'est pas un nombre décimal.
    InvalidNumber,
    /// Règle 1 : `periodFrom > periodTo`.
    PeriodInverted,
    /// Règle 2 : plus de 500 lignes.
    TooManyLines,
    /// Règle 3 : date de comptabilisation hors de la période.
    OutOfPeriod,
    /// Règle 4 : texte trop long.
    TooLong,
    /// Règle 5 : montant d'une ligne nul ou négatif.
    NotPositive,
    /// Règle 5 : plus de deux décimales après `normalize()`.
    TooManyDecimals,
    /// Règle 5 : plus de treize chiffres entiers.
    TooManyDigits,
    /// Sens autre que `credit` ou `debit`.
    UnknownDirection,
}

impl ManualStatementReason {
    /// Nombre de variantes — tenu **à côté** du `match` de [`Self::index`] :
    /// une variante neuve impose un bras d'`index`, et le test d'exhaustivité
    /// rougit à l'exécution si `COUNT` ou [`Self::ALL`] ne suivent pas
    /// (C-v016-2-20).
    pub const COUNT: usize = 11;

    /// Toutes les variantes, dans l'ordre de [`Self::index`].
    pub const ALL: [Self; 11] = [
        Self::Required,
        Self::InvalidDate,
        Self::InvalidNumber,
        Self::PeriodInverted,
        Self::TooManyLines,
        Self::OutOfPeriod,
        Self::TooLong,
        Self::NotPositive,
        Self::TooManyDecimals,
        Self::TooManyDigits,
        Self::UnknownDirection,
    ];

    /// Position de la variante — `match` exhaustif **sans joker**.
    pub fn index(self) -> usize {
        match self {
            Self::Required => 0,
            Self::InvalidDate => 1,
            Self::InvalidNumber => 2,
            Self::PeriodInverted => 3,
            Self::TooManyLines => 4,
            Self::OutOfPeriod => 5,
            Self::TooLong => 6,
            Self::NotPositive => 7,
            Self::TooManyDecimals => 8,
            Self::TooManyDigits => 9,
            Self::UnknownDirection => 10,
        }
    }

    /// Code stable sérialisé dans `details.reason`.
    pub fn code(self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::InvalidDate => "invalid_date",
            Self::InvalidNumber => "invalid_number",
            Self::PeriodInverted => "period_inverted",
            Self::TooManyLines => "too_many_lines",
            Self::OutOfPeriod => "out_of_period",
            Self::TooLong => "too_long",
            Self::NotPositive => "not_positive",
            Self::TooManyDecimals => "too_many_decimals",
            Self::TooManyDigits => "too_many_digits",
            Self::UnknownDirection => "unknown_direction",
        }
    }
}

/// Champ en cause d'un refus `BANK_STATEMENT_INVALID` — ensemble **fermé**,
/// codes = noms JSON du corps. Les sept derniers portent un rang de ligne.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ManualStatementField {
    PeriodFrom,
    PeriodTo,
    StatementNumber,
    OpeningBalance,
    ClosingBalance,
    Lines,
    BookingDate,
    ValueDate,
    Details,
    CounterpartyName,
    Reference,
    Amount,
    Direction,
}

impl ManualStatementField {
    /// Nombre de variantes — même dispositif que [`ManualStatementReason::COUNT`].
    pub const COUNT: usize = 13;

    /// Toutes les variantes, dans l'ordre de [`Self::index`].
    pub const ALL: [Self; 13] = [
        Self::PeriodFrom,
        Self::PeriodTo,
        Self::StatementNumber,
        Self::OpeningBalance,
        Self::ClosingBalance,
        Self::Lines,
        Self::BookingDate,
        Self::ValueDate,
        Self::Details,
        Self::CounterpartyName,
        Self::Reference,
        Self::Amount,
        Self::Direction,
    ];

    /// Position de la variante — `match` exhaustif **sans joker**.
    pub fn index(self) -> usize {
        match self {
            Self::PeriodFrom => 0,
            Self::PeriodTo => 1,
            Self::StatementNumber => 2,
            Self::OpeningBalance => 3,
            Self::ClosingBalance => 4,
            Self::Lines => 5,
            Self::BookingDate => 6,
            Self::ValueDate => 7,
            Self::Details => 8,
            Self::CounterpartyName => 9,
            Self::Reference => 10,
            Self::Amount => 11,
            Self::Direction => 12,
        }
    }

    /// Code stable sérialisé dans `details.field` (nom JSON du champ).
    pub fn code(self) -> &'static str {
        match self {
            Self::PeriodFrom => "periodFrom",
            Self::PeriodTo => "periodTo",
            Self::StatementNumber => "statementNumber",
            Self::OpeningBalance => "openingBalance",
            Self::ClosingBalance => "closingBalance",
            Self::Lines => "lines",
            Self::BookingDate => "bookingDate",
            Self::ValueDate => "valueDate",
            Self::Details => "details",
            Self::CounterpartyName => "counterpartyName",
            Self::Reference => "reference",
            Self::Amount => "amount",
            Self::Direction => "direction",
        }
    }
}

// ---------------------------------------------------------------------------
// Corps reçu et types du domaine
// ---------------------------------------------------------------------------

/// Corps JSON des deux routes de saisie, champs en chaînes : une faute de forme
/// d'une **chaîne** rend `BANK_STATEMENT_INVALID` avec son rang, non un rejet
/// d'Axum. Seul `bankAccountId` est requis par la désérialisation ; un champ
/// d'un autre type JSON que chaîne est un rejet du corps (`400
/// VALIDATION_ERROR`, sans `field`).
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualStatementRaw {
    pub bank_account_id: i64,
    #[serde(default)]
    pub period_from: Option<String>,
    #[serde(default)]
    pub period_to: Option<String>,
    #[serde(default)]
    pub statement_number: Option<String>,
    #[serde(default)]
    pub opening_balance: Option<String>,
    #[serde(default)]
    pub closing_balance: Option<String>,
    /// Au plus `MANUAL_STATEMENT_MAX_LINES + 1` lignes sont **retenues** à la
    /// désérialisation ([`deserialize_capped_lines`]) : au-delà, les éléments
    /// sont lus et jetés, et la 501e suffit à `too_many_lines`.
    #[serde(default, deserialize_with = "deserialize_capped_lines")]
    pub lines: Option<Vec<ManualLineRaw>>,
    /// Ignoré par l'aperçu.
    #[serde(default)]
    pub confirm_duplicate_file: bool,
    /// Ignoré par l'aperçu.
    #[serde(default)]
    pub confirm_duplicate_lines: bool,
    /// Ignoré par l'aperçu.
    #[serde(default)]
    pub confirm_opening_balance_gap: bool,
}

/// Une ligne du corps, champs en chaînes.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualLineRaw {
    #[serde(default)]
    pub booking_date: Option<String>,
    #[serde(default)]
    pub value_date: Option<String>,
    #[serde(default)]
    pub details: Option<String>,
    #[serde(default)]
    pub counterparty_name: Option<String>,
    #[serde(default)]
    pub reference: Option<String>,
    #[serde(default)]
    pub amount: Option<String>,
    #[serde(default)]
    pub direction: Option<String>,
}

/// Sens d'une ligne : `credit` = entrée sur le compte, `debit` = sortie.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManualDirection {
    Credit,
    Debit,
}

impl ManualDirection {
    /// Code du corps JSON (`credit` / `debit`).
    pub fn code(self) -> &'static str {
        match self {
            Self::Credit => "credit",
            Self::Debit => "debit",
        }
    }
}

/// Relevé analysé. Les textes sont déjà `trim`-és ; un texte facultatif vide
/// vaut `None`.
#[derive(Clone, Debug, PartialEq)]
pub struct ManualStatementInput {
    pub bank_account_id: i64,
    pub period_from: NaiveDate,
    pub period_to: NaiveDate,
    pub statement_number: Option<String>,
    pub opening_balance: Decimal,
    pub closing_balance: Decimal,
    pub lines: Vec<ManualStatementLine>,
}

/// Ligne analysée. `amount` est le montant **positif** saisi ; le signe vient
/// de `direction`.
#[derive(Clone, Debug, PartialEq)]
pub struct ManualStatementLine {
    pub booking_date: NaiveDate,
    pub value_date: Option<NaiveDate>,
    pub details: String,
    pub counterparty_name: Option<String>,
    pub reference: Option<String>,
    pub amount: Decimal,
    pub direction: ManualDirection,
}

/// Sommes d'un relevé équilibré, rendues par l'aperçu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManualStatementSums {
    pub total_credits: Decimal,
    pub total_debits: Decimal,
    pub computed_closing_balance: Decimal,
    pub transaction_count: usize,
}

/// Refus d'un relevé saisi.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManualStatementError {
    /// `400 BANK_STATEMENT_INVALID` — `line` est le rang 1-based dans le
    /// tableau envoyé, `None` pour un champ de l'entête.
    Invalid {
        line: Option<usize>,
        field: ManualStatementField,
        reason: ManualStatementReason,
    },
    /// `422 BANK_STATEMENT_UNBALANCED` — `départ + crédits − débits ≠ arrivée`.
    Unbalanced {
        opening: Decimal,
        closing: Decimal,
        computed_closing: Decimal,
        diff: Decimal,
    },
}

impl ManualStatementError {
    fn header(field: ManualStatementField, reason: ManualStatementReason) -> Self {
        Self::Invalid {
            line: None,
            field,
            reason,
        }
    }

    fn at_line(index0: usize, field: ManualStatementField, reason: ManualStatementReason) -> Self {
        Self::Invalid {
            line: Some(index0 + 1),
            field,
            reason,
        }
    }
}

// ---------------------------------------------------------------------------
// Analyse et validation
// ---------------------------------------------------------------------------

/// Analyse le corps reçu vers [`ManualStatementInput`].
///
/// Refuse, dans l'ordre de l'entête puis des lignes : une chaîne requise
/// absente ou vide (`required`), une date qui n'est pas `AAAA-MM-JJ`
/// (`invalid_date`), un nombre illisible (`invalid_number`), un sens inconnu
/// (`unknown_direction`), et plus de 500 lignes (`too_many_lines`, avant
/// l'analyse des lignes). Les autres règles sont celles de
/// [`validate_manual_statement`].
pub fn parse_manual_statement(
    raw: &ManualStatementRaw,
) -> Result<ManualStatementInput, ManualStatementError> {
    use ManualStatementField as F;

    let period_from = parse_required_date(raw.period_from.as_deref())
        .map_err(|r| ManualStatementError::header(F::PeriodFrom, r))?;
    let period_to = parse_required_date(raw.period_to.as_deref())
        .map_err(|r| ManualStatementError::header(F::PeriodTo, r))?;
    let statement_number = optional_text(raw.statement_number.as_deref());
    let opening_balance = parse_required_decimal(raw.opening_balance.as_deref())
        .map_err(|r| ManualStatementError::header(F::OpeningBalance, r))?;
    let closing_balance = parse_required_decimal(raw.closing_balance.as_deref())
        .map_err(|r| ManualStatementError::header(F::ClosingBalance, r))?;
    let raw_lines = raw.lines.as_ref().ok_or(ManualStatementError::header(
        F::Lines,
        ManualStatementReason::Required,
    ))?;
    if raw_lines.len() > MANUAL_STATEMENT_MAX_LINES {
        return Err(ManualStatementError::header(
            F::Lines,
            ManualStatementReason::TooManyLines,
        ));
    }

    let mut lines = Vec::with_capacity(raw_lines.len());
    for (i, l) in raw_lines.iter().enumerate() {
        let booking_date = parse_required_date(l.booking_date.as_deref())
            .map_err(|r| ManualStatementError::at_line(i, F::BookingDate, r))?;
        let value_date = match optional_text(l.value_date.as_deref()) {
            None => None,
            Some(s) => Some(
                parse_date(&s).map_err(|r| ManualStatementError::at_line(i, F::ValueDate, r))?,
            ),
        };
        let details = optional_text(l.details.as_deref()).ok_or(ManualStatementError::at_line(
            i,
            F::Details,
            ManualStatementReason::Required,
        ))?;
        let amount = parse_required_decimal(l.amount.as_deref())
            .map_err(|r| ManualStatementError::at_line(i, F::Amount, r))?;
        let direction = match optional_text(l.direction.as_deref()).as_deref() {
            None => {
                return Err(ManualStatementError::at_line(
                    i,
                    F::Direction,
                    ManualStatementReason::Required,
                ));
            }
            Some("credit") => ManualDirection::Credit,
            Some("debit") => ManualDirection::Debit,
            Some(_) => {
                return Err(ManualStatementError::at_line(
                    i,
                    F::Direction,
                    ManualStatementReason::UnknownDirection,
                ));
            }
        };
        lines.push(ManualStatementLine {
            booking_date,
            value_date,
            details,
            counterparty_name: optional_text(l.counterparty_name.as_deref()),
            reference: optional_text(l.reference.as_deref()),
            amount,
            direction,
        });
    }

    Ok(ManualStatementInput {
        bank_account_id: raw.bank_account_id,
        period_from,
        period_to,
        statement_number,
        opening_balance,
        closing_balance,
        lines,
    })
}

/// Contrôle les règles d'un relevé analysé (AC-4, règles 1 à 6) et rend ses
/// sommes. Toutes les longueurs se comptent en caractères, après `trim`.
///
/// # Erreurs
///
/// [`ManualStatementError::Invalid`] pour les règles 1 à 5, puis
/// [`ManualStatementError::Unbalanced`] pour la règle 6 (égalité exacte).
pub fn validate_manual_statement(
    input: &ManualStatementInput,
) -> Result<ManualStatementSums, ManualStatementError> {
    use ManualStatementField as F;
    use ManualStatementReason as R;

    // Règle 1.
    if input.period_from > input.period_to {
        return Err(ManualStatementError::header(F::PeriodTo, R::PeriodInverted));
    }
    // Règle 4 (entête).
    if let Some(n) = &input.statement_number
        && n.trim().chars().count() > MANUAL_SHORT_TEXT_MAX_CHARS
    {
        return Err(ManualStatementError::header(F::StatementNumber, R::TooLong));
    }
    // Règle 5 (soldes : signe libre).
    check_amount_scale(input.opening_balance)
        .map_err(|r| ManualStatementError::header(F::OpeningBalance, r))?;
    check_amount_scale(input.closing_balance)
        .map_err(|r| ManualStatementError::header(F::ClosingBalance, r))?;
    // Règle 2.
    if input.lines.len() > MANUAL_STATEMENT_MAX_LINES {
        return Err(ManualStatementError::header(F::Lines, R::TooManyLines));
    }

    let mut total_credits = Decimal::ZERO;
    let mut total_debits = Decimal::ZERO;
    for (i, l) in input.lines.iter().enumerate() {
        // Règle 3.
        if l.booking_date < input.period_from || l.booking_date > input.period_to {
            return Err(ManualStatementError::at_line(
                i,
                F::BookingDate,
                R::OutOfPeriod,
            ));
        }
        // Règle 4.
        let details_len = l.details.trim().chars().count();
        if details_len == 0 {
            return Err(ManualStatementError::at_line(i, F::Details, R::Required));
        }
        if details_len > MANUAL_DETAILS_MAX_CHARS {
            return Err(ManualStatementError::at_line(i, F::Details, R::TooLong));
        }
        if let Some(c) = &l.counterparty_name
            && c.trim().chars().count() > MANUAL_SHORT_TEXT_MAX_CHARS
        {
            return Err(ManualStatementError::at_line(
                i,
                F::CounterpartyName,
                R::TooLong,
            ));
        }
        if let Some(r) = &l.reference
            && r.trim().chars().count() > MANUAL_SHORT_TEXT_MAX_CHARS
        {
            return Err(ManualStatementError::at_line(i, F::Reference, R::TooLong));
        }
        // Règle 5.
        if l.amount <= Decimal::ZERO {
            return Err(ManualStatementError::at_line(i, F::Amount, R::NotPositive));
        }
        check_amount_scale(l.amount).map_err(|r| ManualStatementError::at_line(i, F::Amount, r))?;

        match l.direction {
            ManualDirection::Credit => total_credits += l.amount,
            ManualDirection::Debit => total_debits += l.amount,
        }
    }

    // Règle 6 — égalité exacte, sans tolérance (C-v016-2-4).
    let computed = input.opening_balance + total_credits - total_debits;
    if computed != input.closing_balance {
        return Err(ManualStatementError::Unbalanced {
            opening: input.opening_balance,
            closing: input.closing_balance,
            computed_closing: computed,
            diff: input.closing_balance - computed,
        });
    }

    Ok(ManualStatementSums {
        total_credits,
        total_debits,
        computed_closing_balance: computed,
        transaction_count: input.lines.len(),
    })
}

/// Contenu canonique du relevé, dont le SHA-256 tient lieu de `file_hash`.
///
/// Préfixe de domaine, compte, période, numéro, soldes, nombre de lignes, puis
/// chaque ligne dans l'ordre avec tous ses champs. Textes `trim`-és, absence ≡
/// chaîne vide, montants `normalize()`-és (`"50"` ≡ `"50.00"`). **Chaque champ
/// est encodé par sa longueur sur quatre octets gros-boutiens suivie de ses
/// octets UTF-8** : aucun contenu ne peut imiter une frontière de champ.
pub fn manual_statement_canonical_bytes(input: &ManualStatementInput) -> Vec<u8> {
    let mut out = Vec::new();
    push_field(&mut out, MANUAL_STATEMENT_HASH_DOMAIN);
    push_field(&mut out, &input.bank_account_id.to_string());
    push_field(&mut out, &input.period_from.to_string());
    push_field(&mut out, &input.period_to.to_string());
    push_field(
        &mut out,
        input.statement_number.as_deref().unwrap_or("").trim(),
    );
    push_field(&mut out, &input.opening_balance.normalize().to_string());
    push_field(&mut out, &input.closing_balance.normalize().to_string());
    push_field(&mut out, &input.lines.len().to_string());
    for l in &input.lines {
        push_field(&mut out, &l.booking_date.to_string());
        push_field(
            &mut out,
            &l.value_date.map(|d| d.to_string()).unwrap_or_default(),
        );
        push_field(&mut out, l.details.trim());
        push_field(
            &mut out,
            l.counterparty_name.as_deref().unwrap_or("").trim(),
        );
        push_field(&mut out, l.reference.as_deref().unwrap_or("").trim());
        push_field(&mut out, &l.amount.normalize().to_string());
        push_field(&mut out, l.direction.code());
    }
    out
}

/// Nom stocké d'un relevé saisi : `saisie-manuelle-<du>-<au>` (C-v016-2-11).
pub fn manual_statement_filename(period_from: NaiveDate, period_to: NaiveDate) -> String {
    format!("saisie-manuelle-{period_from}-{period_to}")
}

/// Convertit un relevé validé en brouillons d'entête et de transactions.
///
/// `source_format = Manual`, soldes à `Some`, `statement_id` = numéro, montants
/// signés selon le sens (`credit` positif, `debit` négatif), devise CHF ;
/// identifiants de bout en bout, de transaction et IBAN de contrepartie absents.
pub fn manual_statement_to_drafts(
    input: &ManualStatementInput,
    bank_account_id: i64,
    company_id: i64,
    file_hash: String,
    filename: String,
    imported_at: NaiveDateTime,
    user_id: i64,
) -> (BankImportDraft, Vec<BankTransactionDraft>) {
    let transactions: Vec<BankTransactionDraft> = input
        .lines
        .iter()
        .map(|l| BankTransactionDraft {
            booking_date: l.booking_date,
            value_date: l.value_date,
            amount: Money::new(match l.direction {
                ManualDirection::Credit => l.amount,
                ManualDirection::Debit => -l.amount,
            }),
            currency: "CHF".to_string(),
            reference: l.reference.clone(),
            details: l.details.trim().to_string(),
            end_to_end_id: None,
            transaction_id: None,
            counterparty_iban: None,
            counterparty_name: l.counterparty_name.clone(),
        })
        .collect();
    let draft = BankImportDraft {
        company_id,
        bank_account_id,
        filename,
        file_hash,
        source_format: SourceFormatTag::Manual,
        statement_id: input.statement_number.clone(),
        period_from: input.period_from,
        period_to: input.period_to,
        opening_balance: Some(Money::new(input.opening_balance)),
        closing_balance: Some(Money::new(input.closing_balance)),
        transaction_count: i32::try_from(transactions.len()).unwrap_or(i32::MAX),
        imported_at,
        imported_by_user_id: user_id,
    };
    (draft, transactions)
}

// ---------------------------------------------------------------------------
// Aides privées
// ---------------------------------------------------------------------------

/// Désérialise `lines` en ne **retenant** que `MANUAL_STATEMENT_MAX_LINES + 1`
/// éléments : les suivants sont lus et jetés (`IgnoredAny`), si bien que la
/// mémoire ne dépend plus de la taille du corps — un corps fait de milliers de
/// `{}` ne se matérialise plus en autant de `ManualLineRaw` avant le refus
/// `too_many_lines` (revue de code P1 de la v016-2a, E1). Un élément jeté
/// d'une autre forme qu'un objet n'est pas contrôlé : le relevé est de toute
/// façon refusé (`too_many_lines`).
fn deserialize_capped_lines<'de, D>(deserializer: D) -> Result<Option<Vec<ManualLineRaw>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{IgnoredAny, SeqAccess, Visitor};

    struct Capped;

    impl<'de> Visitor<'de> for Capped {
        type Value = Option<Vec<ManualLineRaw>>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("un tableau de lignes ou null")
        }

        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_some<D2: serde::Deserializer<'de>>(self, d: D2) -> Result<Self::Value, D2::Error> {
            d.deserialize_seq(Capped)
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let cap = MANUAL_STATEMENT_MAX_LINES + 1;
            let mut lines = Vec::with_capacity(seq.size_hint().unwrap_or(0).min(cap));
            while lines.len() < cap {
                match seq.next_element::<ManualLineRaw>()? {
                    Some(l) => lines.push(l),
                    None => return Ok(Some(lines)),
                }
            }
            while seq.next_element::<IgnoredAny>()?.is_some() {}
            Ok(Some(lines))
        }
    }

    deserializer.deserialize_option(Capped)
}

/// Texte facultatif : `trim`, et une chaîne vide ou faite de blancs vaut absence.
fn optional_text(s: Option<&str>) -> Option<String> {
    s.map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_string)
}

fn parse_required_date(s: Option<&str>) -> Result<NaiveDate, ManualStatementReason> {
    match optional_text(s) {
        None => Err(ManualStatementReason::Required),
        Some(t) => parse_date(&t),
    }
}

/// Date stricte `AAAA-MM-JJ` (dix caractères, chiffres aux bonnes places).
fn parse_date(s: &str) -> Result<NaiveDate, ManualStatementReason> {
    let b = s.as_bytes();
    let shape_ok = b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit());
    if !shape_ok {
        return Err(ManualStatementReason::InvalidDate);
    }
    NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| ManualStatementReason::InvalidDate)
}

fn parse_required_decimal(s: Option<&str>) -> Result<Decimal, ManualStatementReason> {
    match optional_text(s) {
        None => Err(ManualStatementReason::Required),
        Some(t) => parse_decimal(&t),
    }
}

/// Nombre décimal strict : un `-` facultatif, des chiffres, et au plus une
/// partie décimale introduite par un point (`+5`, `5.`, `.5`, `1e3` refusés).
/// Une partie entière de plus de treize chiffres significatifs rend
/// `too_many_digits`, une partie décimale de plus de deux chiffres
/// significatifs (zéros finaux ôtés) `too_many_decimals` — l'une et l'autre
/// **avant** l'analyse numérique.
///
/// ⚠️ Revue de code P1 de la v016-2a (B1 = A1 = E3) : `str::parse::<Decimal>`
/// **arrondit** au-delà de 28 décimales (`rust_decimal`, `parse_str_radix_10`
/// avec `ROUND = true`) — `"50.00000000000000000000000000001"` y devenait `50`
/// et passait la règle 5. La partie décimale est donc bornée sur le **texte**,
/// puis le nombre est lu par `Decimal::from_str_exact`, qui refuse tout
/// arrondi : aucun montant n'est altéré en silence.
fn parse_decimal(s: &str) -> Result<Decimal, ManualStatementReason> {
    let (negative, unsigned) = match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s),
    };
    let (int_part, frac_part) = match unsigned.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (unsigned, None),
    };
    let digits_only = |p: &str| !p.is_empty() && p.bytes().all(|c| c.is_ascii_digit());
    if !digits_only(int_part) || frac_part.is_some_and(|f| !digits_only(f)) {
        return Err(ManualStatementReason::InvalidNumber);
    }
    let significant = int_part.trim_start_matches('0').len();
    if significant > MANUAL_AMOUNT_MAX_INTEGER_DIGITS as usize {
        return Err(ManualStatementReason::TooManyDigits);
    }
    let frac = frac_part.unwrap_or("").trim_end_matches('0');
    if frac.len() > MANUAL_AMOUNT_MAX_DECIMALS as usize {
        return Err(ManualStatementReason::TooManyDecimals);
    }
    // Au plus treize chiffres entiers et deux décimales : la lecture exacte ne
    // peut ni déborder ni arrondir.
    let int = match int_part.trim_start_matches('0') {
        "" => "0",
        i => i,
    };
    let canonical = if frac.is_empty() {
        int.to_string()
    } else {
        format!("{int}.{frac}")
    };
    let value =
        Decimal::from_str_exact(&canonical).map_err(|_| ManualStatementReason::InvalidNumber)?;
    Ok(if negative { -value } else { value })
}

/// Règle 5 : au plus deux décimales **après `normalize()`**, au plus treize
/// chiffres entiers.
fn check_amount_scale(n: Decimal) -> Result<(), ManualStatementReason> {
    if n.normalize().scale() > MANUAL_AMOUNT_MAX_DECIMALS {
        return Err(ManualStatementReason::TooManyDecimals);
    }
    let limit = Decimal::from(10_i64.pow(MANUAL_AMOUNT_MAX_INTEGER_DIGITS));
    if n.abs().trunc() >= limit {
        return Err(ManualStatementReason::TooManyDigits);
    }
    Ok(())
}

/// Encode un champ : longueur en octets sur quatre octets gros-boutiens, puis
/// les octets UTF-8.
fn push_field(out: &mut Vec<u8>, s: &str) {
    let len = u32::try_from(s.len()).unwrap_or(u32::MAX);
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(s.as_bytes());
}

#[cfg(test)]
#[path = "manual_tests.rs"]
mod tests;
