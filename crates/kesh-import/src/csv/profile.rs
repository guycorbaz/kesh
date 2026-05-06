//! Profil CSV bancaire : mapping colonnes + format date + séparateurs.
//!
//! Type autonome `kesh-import` (zéro dépendance `kesh-core`/`kesh-db`).
//! Les conversions vers `kesh-db::entities::BankProfile` se font côté
//! `kesh-core` via `From`/`Into` (cf. décision archi #7).
//!
//! ## Validation
//!
//! [`CsvProfile::validate`] applique **toutes** les règles
//! métier avant insertion DB ou usage parser :
//! - XOR `amount` vs `debit_credit_split` (Pass 1 H6 + M13)
//! - Indices `column_mapping` deux à deux distincts (Pass 1 M13 collision)
//! - `date_format` chrono compilable
//! - `field_separator` ∈ {',', ';', '\t'}
//! - `decimal_separator` ∈ {'.', ','}
//! - Séparateurs distincts (Pass 2 M'6 anti-ambiguïté)
//! - `header_row_count` ≤ 5
//! - `filename_pattern` regex compilable + ≤ 200 chars
//! - `bank_name` 1..=100 chars

use crate::error::CsvError;
use chrono::format::Item as ChronoItem;
use chrono::format::strftime::StrftimeItems;
use serde::{Deserialize, Serialize};

/// Mapping colonnes vers les champs `ImportedTransaction`. Indices
/// 0-based sur les colonnes du CSV (post header skip).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColumnMapping {
    /// Index colonne date (obligatoire).
    pub date: usize,
    /// Index colonne amount (signe préservé). Pass 1 H6 : `Option`,
    /// XOR avec `debit_credit_split`.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub amount: Option<usize>,
    /// Indices (debit, credit) pour les CSV qui exposent débit et
    /// crédit séparément. Pass 1 H6 : XOR avec `amount`.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub debit_credit_split: Option<(usize, usize)>,
    /// Index colonne référence (Optionnel).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub reference: Option<usize>,
    /// Index colonne détails (Optionnel ; si None, `details` produit
    /// par parser sera `String::new()`).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub details: Option<usize>,
    /// Index colonne contrepartie (Optionnel ; mappé vers
    /// `ImportedTransaction.counterparty_name`).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub counterparty: Option<usize>,
}

impl ColumnMapping {
    /// Tous les indices spécifiés (incluant les 2 valeurs de
    /// `debit_credit_split` si Some). Sert à la collision check.
    pub fn all_indices(&self) -> Vec<(&'static str, usize)> {
        let mut out = vec![("date", self.date)];
        if let Some(i) = self.amount {
            out.push(("amount", i));
        }
        if let Some((d, c)) = self.debit_credit_split {
            out.push(("debit_credit_split.0", d));
            out.push(("debit_credit_split.1", c));
        }
        if let Some(i) = self.reference {
            out.push(("reference", i));
        }
        if let Some(i) = self.details {
            out.push(("details", i));
        }
        if let Some(i) = self.counterparty {
            out.push(("counterparty", i));
        }
        out
    }

    /// Index maximum utilisé (pour validation hors-borne post-parse
    /// du 1er record). 0 si seulement `date = 0`.
    pub fn max_index(&self) -> usize {
        self.all_indices()
            .into_iter()
            .map(|(_, i)| i)
            .max()
            .unwrap_or(0)
    }
}

/// Profil de format CSV par banque. Type autonome `kesh-import`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CsvProfile {
    /// Nom de la banque (1..=100 chars). Doit être unique par company
    /// côté DB (UNIQUE constraint).
    pub bank_name: String,
    /// Pattern regex compilable pour auto-match par filename.
    /// `None` = profil non-éligible à l'auto-match.
    pub filename_pattern: Option<String>,
    /// Mapping colonnes (cf. [`ColumnMapping`]).
    pub column_mapping: ColumnMapping,
    /// Format chrono de la date (ex. `"%d.%m.%Y"`, `"%Y-%m-%d"`).
    pub date_format: String,
    /// Séparateur décimal (`'.'` ou `','`).
    pub decimal_separator: char,
    /// Séparateur de champs (`','`, `';'` ou `'\t'`).
    pub field_separator: char,
    /// Encoding optionnel ; `None` = auto-détection.
    /// Pass 1 H5 + Pass 3 anti-bypass : si non-null et diverge de la
    /// détection auto → mismatch handling spec §encoding-detection.
    pub encoding: Option<String>,
    /// Nombre de lignes de header à skipper (0..=5).
    pub header_row_count: u8,
}

impl CsvProfile {
    /// Valide le profil contre toutes les règles métier (cf. doc module).
    pub fn validate(&self) -> Result<(), CsvError> {
        // bank_name length
        let len = self.bank_name.chars().count();
        if !(1..=100).contains(&len) {
            return Err(CsvError::ProfileMisconfigured(format!(
                "bank_name length {} doit être 1..=100",
                len
            )));
        }

        // XOR amount / debit_credit_split (Pass 1 H6)
        match (
            self.column_mapping.amount,
            self.column_mapping.debit_credit_split,
        ) {
            (Some(_), Some(_)) => {
                return Err(CsvError::ProfileMisconfigured(
                    "column_mapping doit contenir exactement un de amount XOR debit_credit_split"
                        .to_string(),
                ));
            }
            (None, None) => {
                return Err(CsvError::ProfileMisconfigured(
                    "column_mapping doit contenir exactement un de amount XOR debit_credit_split"
                        .to_string(),
                ));
            }
            _ => {}
        }

        // Collision indices (Pass 1 M13)
        let indices = self.column_mapping.all_indices();
        for i in 0..indices.len() {
            for j in (i + 1)..indices.len() {
                let (name_a, val_a) = indices[i];
                let (name_b, val_b) = indices[j];
                if val_a == val_b {
                    return Err(CsvError::ProfileMisconfigured(format!(
                        "column_mapping.{} ({}) conflicts with column_mapping.{} ({})",
                        name_a, val_a, name_b, val_b
                    )));
                }
            }
        }

        // date_format chrono compilable
        if self.date_format.is_empty() || self.date_format.len() > 50 {
            return Err(CsvError::ProfileMisconfigured(format!(
                "date_format length {} doit être 1..=50",
                self.date_format.len()
            )));
        }
        for item in StrftimeItems::new(&self.date_format) {
            if matches!(item, ChronoItem::Error) {
                return Err(CsvError::ProfileMisconfigured(format!(
                    "date_format \"{}\" contient un token chrono invalide",
                    self.date_format
                )));
            }
        }

        // field_separator ∈ {',', ';', '\t'}
        if !matches!(self.field_separator, ',' | ';' | '\t') {
            return Err(CsvError::ProfileMisconfigured(format!(
                "field_separator '{}' invalide (attendu ',', ';' ou '\\t')",
                self.field_separator.escape_default()
            )));
        }

        // decimal_separator ∈ {'.', ','}
        if !matches!(self.decimal_separator, '.' | ',') {
            return Err(CsvError::ProfileMisconfigured(format!(
                "decimal_separator '{}' invalide (attendu '.' ou ',')",
                self.decimal_separator
            )));
        }

        // Séparateurs distincts (Pass 2 M'6)
        if self.field_separator == self.decimal_separator {
            return Err(CsvError::ProfileMisconfigured(format!(
                "field_separator ('{}') doit être différent de decimal_separator ('{}')",
                self.field_separator, self.decimal_separator
            )));
        }

        // header_row_count
        if self.header_row_count > 5 {
            return Err(CsvError::ProfileMisconfigured(format!(
                "header_row_count {} > 5",
                self.header_row_count
            )));
        }

        // filename_pattern : regex compilable + length ≤ 200
        if let Some(ref pat) = self.filename_pattern {
            if pat.is_empty() || pat.len() > 200 {
                return Err(CsvError::ProfileMisconfigured(format!(
                    "filename_pattern length {} doit être 1..=200",
                    pat.len()
                )));
            }
            if let Err(e) = regex::Regex::new(pat) {
                return Err(CsvError::ProfileMisconfigured(format!(
                    "filename_pattern regex invalide : {}",
                    e
                )));
            }
        }

        // encoding (si Some) ∈ {"UTF-8", "ISO-8859-1"}
        if let Some(ref enc) = self.encoding {
            if !matches!(enc.as_str(), "UTF-8" | "ISO-8859-1") {
                return Err(CsvError::ProfileMisconfigured(format!(
                    "encoding \"{}\" non supporté v0.1 (attendu \"UTF-8\" ou \"ISO-8859-1\")",
                    enc
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_profile() -> CsvProfile {
        CsvProfile {
            bank_name: "UBS".to_string(),
            filename_pattern: Some(r"^export-\d+\.csv$".to_string()),
            column_mapping: ColumnMapping {
                date: 0,
                amount: Some(1),
                debit_credit_split: None,
                reference: Some(2),
                details: Some(3),
                counterparty: None,
            },
            date_format: "%d.%m.%Y".to_string(),
            decimal_separator: '.',
            field_separator: ';',
            encoding: None,
            header_row_count: 1,
        }
    }

    #[test]
    fn valid_profile_passes() {
        valid_profile().validate().unwrap();
    }

    #[test]
    fn rejects_xor_violation_both_present() {
        let mut p = valid_profile();
        p.column_mapping.debit_credit_split = Some((4, 5));
        let err = p.validate().unwrap_err();
        assert!(matches!(err, CsvError::ProfileMisconfigured(_)));
    }

    #[test]
    fn rejects_xor_violation_neither_present() {
        let mut p = valid_profile();
        p.column_mapping.amount = None;
        p.column_mapping.debit_credit_split = None;
        let err = p.validate().unwrap_err();
        assert!(matches!(err, CsvError::ProfileMisconfigured(_)));
    }

    #[test]
    fn rejects_collision_date_amount_same_index() {
        let mut p = valid_profile();
        p.column_mapping.date = 1;
        p.column_mapping.amount = Some(1);
        let err = p.validate().unwrap_err();
        match err {
            CsvError::ProfileMisconfigured(msg) => {
                assert!(msg.contains("date"));
                assert!(msg.contains("amount"));
            }
            _ => panic!("expected ProfileMisconfigured"),
        }
    }

    #[test]
    fn rejects_collision_debit_credit_same_index() {
        let mut p = valid_profile();
        p.column_mapping.amount = None;
        p.column_mapping.debit_credit_split = Some((3, 3));
        let err = p.validate().unwrap_err();
        assert!(matches!(err, CsvError::ProfileMisconfigured(_)));
    }

    #[test]
    fn rejects_invalid_chrono_format() {
        let mut p = valid_profile();
        p.date_format = "%Q".to_string(); // %Q n'existe pas en chrono
        let err = p.validate().unwrap_err();
        assert!(matches!(err, CsvError::ProfileMisconfigured(_)));
    }

    #[test]
    fn rejects_equal_separators() {
        let mut p = valid_profile();
        p.field_separator = ',';
        p.decimal_separator = ',';
        let err = p.validate().unwrap_err();
        match err {
            CsvError::ProfileMisconfigured(msg) => assert!(msg.contains("différent")),
            _ => panic!("expected ProfileMisconfigured"),
        }
    }

    #[test]
    fn rejects_invalid_field_separator() {
        let mut p = valid_profile();
        p.field_separator = '|';
        let err = p.validate().unwrap_err();
        assert!(matches!(err, CsvError::ProfileMisconfigured(_)));
    }

    #[test]
    fn rejects_header_row_count_over_5() {
        let mut p = valid_profile();
        p.header_row_count = 6;
        let err = p.validate().unwrap_err();
        assert!(matches!(err, CsvError::ProfileMisconfigured(_)));
    }

    #[test]
    fn rejects_pattern_over_200_chars() {
        let mut p = valid_profile();
        p.filename_pattern = Some("a".repeat(201));
        let err = p.validate().unwrap_err();
        assert!(matches!(err, CsvError::ProfileMisconfigured(_)));
    }

    #[test]
    fn rejects_invalid_regex() {
        let mut p = valid_profile();
        p.filename_pattern = Some("(unclosed".to_string());
        let err = p.validate().unwrap_err();
        assert!(matches!(err, CsvError::ProfileMisconfigured(_)));
    }

    #[test]
    fn rejects_unsupported_encoding() {
        let mut p = valid_profile();
        p.encoding = Some("UTF-16LE".to_string());
        let err = p.validate().unwrap_err();
        assert!(matches!(err, CsvError::ProfileMisconfigured(_)));
    }

    #[test]
    fn debit_credit_split_variant_validates() {
        let mut p = valid_profile();
        p.column_mapping.amount = None;
        p.column_mapping.debit_credit_split = Some((1, 2));
        p.column_mapping.reference = Some(3);
        p.column_mapping.details = Some(4);
        p.validate().unwrap();
    }

    #[test]
    fn max_index_returns_highest() {
        let p = valid_profile();
        // date=0, amount=1, ref=2, details=3
        assert_eq!(p.column_mapping.max_index(), 3);
    }

    #[test]
    fn column_mapping_serde_roundtrip_amount() {
        let cm = ColumnMapping {
            date: 0,
            amount: Some(1),
            debit_credit_split: None,
            reference: Some(2),
            details: Some(3),
            counterparty: None,
        };
        let json = serde_json::to_string(&cm).unwrap();
        // amount présent, debit_credit_split + counterparty omis (skip None)
        assert!(json.contains("\"amount\":1"));
        assert!(!json.contains("debit_credit_split"));
        assert!(!json.contains("counterparty"));
        let cm2: ColumnMapping = serde_json::from_str(&json).unwrap();
        assert_eq!(cm, cm2);
    }

    #[test]
    fn column_mapping_serde_roundtrip_debit_credit() {
        let cm = ColumnMapping {
            date: 0,
            amount: None,
            debit_credit_split: Some((1, 2)),
            reference: None,
            details: None,
            counterparty: None,
        };
        let json = serde_json::to_string(&cm).unwrap();
        assert!(json.contains("\"debit_credit_split\":[1,2]"));
        assert!(!json.contains("\"amount\""));
        let cm2: ColumnMapping = serde_json::from_str(&json).unwrap();
        assert_eq!(cm, cm2);
    }
}
