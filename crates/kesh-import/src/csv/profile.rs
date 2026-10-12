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
//! - Indices `column_mapping` deux à deux distincts (Pass 1 M13 collision),
//!   colonne Devise facultative comprise (Story 27-2, #624)
//! - `date_format` chrono compilable
//! - `field_separator` ∈ {',', ';', '\t'}
//! - `decimal_separator` ∈ {'.', ','}
//! - Séparateurs distincts (Pass 2 M'6 anti-ambiguïté)
//! - `header_row_count` ≤ 20 (Story 26-1, #621 ; 5 auparavant)
//! - `debit_sign = Negative` seulement en mode `debit_credit_split`
//!   (Story 26-1, #620 : en colonne unique, le signe du fichier est gardé
//!   tel quel et le réglage n'a pas de sens)
//! - `filename_pattern` regex compilable + ≤ 200 chars
//! - `bank_name` 1..=100 chars

use crate::error::CsvError;
use chrono::format::Item as ChronoItem;
use chrono::format::strftime::StrftimeItems;
use serde::{Deserialize, Serialize};

/// Plafond de [`CsvProfile::header_row_count`] (Story 26-1, #621 : le
/// relevé UBS en demande 9 ; 5 auparavant). Le `CHECK` de la colonne
/// `bank_profiles.header_row_count` porte la même borne.
pub const MAX_HEADER_ROW_COUNT: u8 = 20;

/// Mapping colonnes vers les champs `ImportedTransaction`. Indices
/// 0-based sur les colonnes du CSV (post header skip).
///
/// ⛔ **`deny_unknown_fields` est un contrat de compatibilité, pas un
/// détail** (Story 27-2, choix C-27-2-1 et C-27-2-15). Ajouter un champ
/// rend les profils qui s'en servent illisibles par les binaires
/// antérieurs : refus bruyant, sans bump. Retirer `deny_unknown_fields`
/// rendrait **silencieux** le retour depuis une version ultérieure — un
/// champ inconnu serait ignoré, et c'est le downgrade corrupteur que la
/// politique de migration interdit. Le test
/// `unknown_key_is_rejected_so_older_binaries_refuse_loudly` le garde.
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
    /// Index de la colonne Devise (facultatif) ; sans elle, le relevé est
    /// supposé en [`DEFAULT_CSV_CURRENCY`](crate::csv::DEFAULT_CSV_CURRENCY)
    /// (Story 27-2, #624). Réglée, la cellule de chaque ligne est lue
    /// (trois lettres ASCII, casse normalisée) et le relevé doit être dans
    /// une seule devise.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub currency: Option<usize>,
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
        if let Some(i) = self.currency {
            out.push(("currency", i));
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

/// Comment la banque écrit les montants de la colonne **Débit**, en mode
/// débit/crédit séparés (Story 26-1, #620) :
///
/// - [`DebitSign::Positive`] — sans signe (`500.00`) : Kesh les rend
///   négatifs (comportement d'avant la 0.13.1, valeur par défaut) ;
/// - [`DebitSign::Negative`] — déjà précédés d'un signe moins (`-500.00`,
///   UBS) : Kesh les garde tels quels.
///
/// C'est un **réglage explicite** du profil, jamais une déduction de Kesh
/// (décision du Project Lead). Un montant de signe contraire au réglage
/// refuse l'import entier ([`CsvError::AmountSignMismatch`]).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DebitSign {
    /// Débits écrits sans signe ; Kesh les rend négatifs.
    #[default]
    Positive,
    /// Débits écrits avec un signe moins ; Kesh les garde.
    Negative,
}

impl DebitSign {
    /// Graphie stockée en base et échangée sur le fil : `"positive"` /
    /// `"negative"` (identique à serde).
    pub fn as_str(&self) -> &'static str {
        match self {
            DebitSign::Positive => "positive",
            DebitSign::Negative => "negative",
        }
    }
}

impl std::str::FromStr for DebitSign {
    type Err = String;

    /// Accepte exactement `"positive"` ou `"negative"` (casse comprise) ;
    /// toute autre chaîne est refusée — jamais de repli silencieux, qui
    /// rouvrirait #620.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "positive" => Ok(DebitSign::Positive),
            "negative" => Ok(DebitSign::Negative),
            other => Err(format!(
                "debit_sign \"{other}\" invalide (attendu \"positive\" ou \"negative\")"
            )),
        }
    }
}

impl TryFrom<&str> for DebitSign {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
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
    /// Nombre de lignes à ignorer avant les données (0..=20), **ligne de
    /// titres comprise**. Seules les lignes **non vides** comptent : une
    /// ligne vide (aucun caractère) est ignorée par le lecteur `csv` et
    /// n'entre pas dans le compte — une ligne faite d'espaces ou de `;`
    /// n'est pas vide, et compte. Relevé UBS : 9 (8 lignes de préambule et
    /// la ligne de titres ; la ligne vide qui les sépare ne compte pas).
    pub header_row_count: u8,
    /// Sens des montants de la colonne Débit en mode débit/crédit séparés
    /// (cf. [`DebitSign`]). Doit valoir [`DebitSign::Positive`] en mode
    /// colonne unique (`amount`), où le signe du fichier est gardé tel quel.
    pub debit_sign: DebitSign,
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

        // header_row_count (Story 26-1, #621 : plafond 5 → 20)
        if self.header_row_count > MAX_HEADER_ROW_COUNT {
            return Err(CsvError::ProfileMisconfigured(format!(
                "header_row_count {} > {}",
                self.header_row_count, MAX_HEADER_ROW_COUNT
            )));
        }

        // debit_sign (Story 26-1, #620) : `negative` n'a de sens qu'en
        // mode débit/crédit séparés.
        if self.debit_sign == DebitSign::Negative && self.column_mapping.amount.is_some() {
            return Err(CsvError::ProfileMisconfigured(
                "debit_sign = negative ne s'applique qu'en mode débit/crédit séparés".to_string(),
            ));
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
        if let Some(ref enc) = self.encoding
            && !matches!(enc.as_str(), "UTF-8" | "ISO-8859-1")
        {
            return Err(CsvError::ProfileMisconfigured(format!(
                "encoding \"{}\" non supporté v0.1 (attendu \"UTF-8\" ou \"ISO-8859-1\")",
                enc
            )));
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
                currency: None,
            },
            date_format: "%d.%m.%Y".to_string(),
            decimal_separator: '.',
            field_separator: ';',
            encoding: None,
            header_row_count: 1,
            debit_sign: DebitSign::Positive,
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
    fn rejects_header_row_count_over_20() {
        let mut p = valid_profile();
        p.header_row_count = 21;
        let err = p.validate().unwrap_err();
        match err {
            CsvError::ProfileMisconfigured(msg) => assert_eq!(msg, "header_row_count 21 > 20"),
            other => panic!("expected ProfileMisconfigured, got {other:?}"),
        }
    }

    /// Story 26-1 (#621) : 20 (la borne) et 9 (le relevé UBS) passent.
    #[test]
    fn accepts_header_row_count_20_and_9() {
        for n in [9u8, 20] {
            let mut p = valid_profile();
            p.header_row_count = n;
            p.validate().unwrap();
        }
    }

    /// Story 26-1 (#620), AC1 : `negative` refusé en colonne unique.
    #[test]
    fn rejects_negative_debit_sign_in_single_column_mode() {
        let mut p = valid_profile();
        p.debit_sign = DebitSign::Negative;
        match p.validate().unwrap_err() {
            CsvError::ProfileMisconfigured(msg) => assert_eq!(
                msg,
                "debit_sign = negative ne s'applique qu'en mode débit/crédit séparés"
            ),
            other => panic!("expected ProfileMisconfigured, got {other:?}"),
        }
    }

    /// AC1 : `positive` en colonne unique, `negative` et `positive` en mode
    /// séparé sont acceptés.
    #[test]
    fn accepts_debit_sign_where_meaningful() {
        valid_profile().validate().unwrap();
        for sign in [DebitSign::Positive, DebitSign::Negative] {
            let mut p = valid_profile();
            p.column_mapping.amount = None;
            p.column_mapping.debit_credit_split = Some((1, 4));
            p.debit_sign = sign;
            p.validate().unwrap();
        }
    }

    /// AC1 : serde `"positive"` / `"negative"` ; toute autre graphie refusée.
    #[test]
    fn debit_sign_serde_and_from_str() {
        assert_eq!(
            serde_json::from_str::<DebitSign>("\"positive\"").unwrap(),
            DebitSign::Positive
        );
        assert_eq!(
            serde_json::from_str::<DebitSign>("\"negative\"").unwrap(),
            DebitSign::Negative
        );
        assert!(serde_json::from_str::<DebitSign>("\"Negative\"").is_err());
        assert!(serde_json::from_str::<DebitSign>("\"autre\"").is_err());
        assert_eq!(
            serde_json::to_string(&DebitSign::Negative).unwrap(),
            "\"negative\""
        );
        assert_eq!(DebitSign::default(), DebitSign::Positive);
        for sign in [DebitSign::Positive, DebitSign::Negative] {
            assert_eq!(sign.as_str().parse::<DebitSign>().unwrap(), sign);
            assert_eq!(DebitSign::try_from(sign.as_str()).unwrap(), sign);
        }
        assert!("Negative".parse::<DebitSign>().is_err());
        assert!(DebitSign::try_from("autre").is_err());
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
            currency: None,
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
            currency: None,
        };
        let json = serde_json::to_string(&cm).unwrap();
        assert!(json.contains("\"debit_credit_split\":[1,2]"));
        assert!(!json.contains("\"amount\""));
        let cm2: ColumnMapping = serde_json::from_str(&json).unwrap();
        assert_eq!(cm, cm2);
    }

    // ── Story 27-2 (#624) : colonne Devise facultative ─────────────────

    #[test]
    fn rejects_collision_currency_date_same_index() {
        let mut p = valid_profile();
        p.column_mapping.currency = Some(0);
        match p.validate().unwrap_err() {
            CsvError::ProfileMisconfigured(msg) => {
                assert!(msg.contains("column_mapping.currency"), "{msg}");
                assert!(msg.contains("column_mapping.date"), "{msg}");
            }
            other => panic!("expected ProfileMisconfigured, got {other:?}"),
        }
    }

    /// Un mapping sans devise se sérialise SANS la clé : c'est ce qui garde
    /// le retour à une 0.13.1 indolore pour les profils qui ne lisent pas la
    /// devise (C-27-2-1).
    #[test]
    fn mapping_without_currency_serializes_without_the_key() {
        let m = valid_profile().column_mapping;
        assert!(!serde_json::to_string(&m).unwrap().contains("currency"));
    }

    #[test]
    fn currency_key_deserializes() {
        let m: ColumnMapping =
            serde_json::from_str(r#"{"date":0,"amount":1,"currency":2}"#).unwrap();
        assert_eq!(m.currency, Some(2));
        assert!(m.all_indices().contains(&("currency", 2)));
    }

    #[test]
    fn legacy_json_without_currency_gives_none() {
        let m: ColumnMapping =
            serde_json::from_str(r#"{"date":2,"debit_credit_split":[5,6]}"#).unwrap();
        assert_eq!(m.currency, None);
    }

    /// ⛔ `deny_unknown_fields` est la prémisse de C-27-2-1 (aucune
    /// migration, aucun bump pour la colonne Devise) : un binaire antérieur
    /// doit REFUSER une clé qu'il ne connaît pas, jamais l'ignorer. Ce test
    /// le garde pour la clé suivante (C-27-2-15).
    #[test]
    fn unknown_key_is_rejected_so_older_binaries_refuse_loudly() {
        assert!(
            serde_json::from_str::<ColumnMapping>(r#"{"date":0,"amount":1,"bogus":2}"#).is_err()
        );
    }
}
