//! Erreurs émises par les parseurs `kesh-import`.
//!
//! `CamtError` représente uniquement les défaillances à la frontière du
//! parseur CAMT.053 (XML mal formé, version non supportée, champ requis
//! manquant). Les violations métier (devise non supportée par la v0.1,
//! balance check échoué) appartiennent à `kesh-core::CoreError` et ne
//! transitent pas par cette enum — la séparation reflète la décision
//! architecture #7 : `kesh-import` est publiable indépendamment et ne
//! connaît rien du domaine Kesh.

use thiserror::Error;

/// Erreurs détectées lors du parsing d'un fichier CAMT.053.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum CamtError {
    /// Le contenu XML est mal formé (balise non fermée, encodage invalide,
    /// caractère illégal, etc.). Le message contient le détail remonté par
    /// `quick-xml` pour faciliter le diagnostic côté logs serveur.
    #[error("XML mal formé : {0}")]
    MalformedXml(String),

    /// Le namespace racine de `<Document>` ne correspond à aucune version
    /// CAMT.053 supportée par le parseur (`camt.053.001.04` ou `.08`).
    #[error("Version CAMT.053 non supportée : {0}")]
    UnsupportedVersion(String),

    /// Un champ requis pour construire un `ImportedStatement` ou une
    /// `ImportedTransaction` est absent du document. Le paramètre porte
    /// le chemin du champ manquant en notation dot-path indexée pour
    /// faciliter le diagnostic sur fichiers multi-`<Stmt>` / multi-`<Ntry>`
    /// (ex. `"stmt[2].ntry[5].amount"`, `"stmt[0].account_iban"`).
    #[error("Champ requis manquant : {0}")]
    MissingRequiredField(String),

    /// Un montant `<Amt>` n'a pas pu être parsé comme `Decimal`. Le message
    /// contient la valeur brute incriminée.
    #[error("Montant invalide : {0}")]
    InvalidAmount(String),

    /// Une date (`<Dt>`, `<DtTm>`, `<FrDtTm>`, `<ToDtTm>`) n'a pas pu être
    /// parsée. Le message contient la valeur brute incriminée.
    #[error("Date invalide : {0}")]
    InvalidDate(String),
}

/// Cap anti-DoS sur le Vec d'erreurs ligne (Pass 2 H'1) : au-delà
/// de cette limite, le parser arrête la collecte mais continue à
/// incrémenter `total_errors` pour signaler la troncature.
pub const MAX_CSV_LINE_ERRORS: usize = 100;

/// Cap anti-DoS sur la taille de la valeur fautive stockée dans
/// `CsvLineError.value` (Pass 3 M''3). Truncation UTF-8-aware via
/// `chars().take(N).collect()` (pas slice octets), suffix `…` si
/// dépassement.
pub const MAX_CSV_LINE_ERROR_VALUE_CHARS: usize = 100;

/// Erreurs détectées lors du parsing d'un fichier CSV bancaire.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum CsvError {
    /// Le fichier est vide ou ne contient aucune ligne de données après
    /// header skip. `reason` discrimine "0 bytes" de "0 data rows after
    /// header skip" pour le diagnostic.
    #[error("Fichier CSV vide : {reason}")]
    EmptyFile { reason: String },

    /// Encoding détecté n'est pas dans `{UTF-8, ISO-8859-1}` (v0.1).
    /// `detected = None` si fichier trop court (< 64 bytes) pour
    /// chardetng fiable.
    #[error("Encoding non supporté v0.1 : {detected:?}")]
    UnsupportedEncoding { detected: Option<String> },

    /// L'encoding détecté diverge de celui spécifié dans le profil.
    /// Non-bloquant en preview (warning), bloquant en final sauf si
    /// l'utilisateur confirme via `confirmEncodingMismatch=true`.
    #[error("Encoding mismatch profil={profile} vs détecté={detected}")]
    EncodingMismatch { profile: String, detected: String },

    /// Le décodage avec l'encoding sélectionné a échoué (bytes invalides
    /// pour cet encoding).
    #[error("Décodage {encoding} échoué à l'offset {byte_offset}")]
    DecodingFailed {
        encoding: String,
        byte_offset: usize,
    },

    /// Le profil exige `header_row_count >= 1` mais le fichier ne
    /// contient pas suffisamment de lignes pour le header.
    #[error("Header CSV manquant ou tronqué")]
    MissingHeader,

    /// Une cellule date n'a pas pu être parsée avec le format chrono
    /// du profil. Variant retourné inline pour les erreurs uniques ;
    /// pour les rejets partiels, voir `PartialFailure`.
    #[error("Date invalide ligne {line} : {value} (format {format})")]
    InvalidDate {
        line: usize,
        value: String,
        format: String,
    },

    /// Une cellule montant n'a pas pu être parsée comme Decimal.
    #[error("Montant invalide ligne {line} : {value}")]
    InvalidAmount { line: usize, value: String },

    /// Sur un profil avec `debit_credit_split`, les deux colonnes
    /// débit ET crédit contiennent une valeur non-vide simultanément
    /// (ambiguïté du sens de la transaction).
    #[error("Débit ET crédit non-vides ligne {line} : ambigu")]
    AmbiguousDebitCredit { line: usize },

    /// Un champ obligatoire (date, amount avec debit+credit empty)
    /// est vide ligne `line`.
    #[error("Champ obligatoire vide ligne {line} : {field}")]
    EmptyMandatoryField { line: usize, field: &'static str },

    /// La ligne contient moins de colonnes que requis par le profil.
    #[error("Ligne {line} trop courte : {got} colonnes vs {expected_cols} attendues")]
    RowTooShort {
        line: usize,
        expected_cols: usize,
        got: usize,
    },

    /// Le profil est mal configuré (indices hors-borne, ou `column_mapping`
    /// avec à la fois `amount` ET `debit_credit_split` Some/None
    /// inattendus). Pass 1 M16 + Pass 2 M'1 + Pass 3 M''2 early-reject
    /// parser-side sur le 1er record.
    #[error("Profil CSV mal configuré : {0}")]
    ProfileMisconfigured(String),

    /// Strict reject FR51 v0.1 : si **une seule** ligne échoue au parsing,
    /// l'import entier est rejeté. La collecte est cappée à
    /// `MAX_CSV_LINE_ERRORS` (Pass 2 H'1 anti-DoS) avec compteur full
    /// `total_errors` et flag `truncated`.
    #[error("Échec parsing partiel : {} erreur(s){}", total_errors, if *truncated { " (tronqué à 100)" } else { "" })]
    PartialFailure {
        errors: Vec<CsvLineError>,
        total_errors: usize,
        truncated: bool,
    },

    /// Erreur I/O sous-jacente (lecture buffer, etc.).
    #[error("I/O CSV : {0}")]
    Io(String),
}

/// Discriminant pour `CsvLineError.code`. Stable vs `CsvError` qui
/// peut évoluer — clé i18n + tri UI dépendent de ces valeurs string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CsvLineErrorCode {
    InvalidDate,
    InvalidAmount,
    AmbiguousDebitCredit,
    EmptyMandatoryField,
    RowTooShort,
}

impl CsvLineErrorCode {
    /// Clé i18n associée au code (préfixe `bank-csv-errors-`).
    pub fn i18n_key(&self) -> &'static str {
        match self {
            CsvLineErrorCode::InvalidDate => "bank-csv-errors-invalid-date",
            CsvLineErrorCode::InvalidAmount => "bank-csv-errors-invalid-amount",
            CsvLineErrorCode::AmbiguousDebitCredit => "bank-csv-errors-ambiguous-debit-credit",
            CsvLineErrorCode::EmptyMandatoryField => "bank-csv-errors-empty-mandatory-field",
            CsvLineErrorCode::RowTooShort => "bank-csv-errors-row-too-short",
        }
    }
}

/// Erreur ligne-par-ligne pour `CsvError::PartialFailure`. (Pass 1 H7)
///
/// `value` est tronqué à `MAX_CSV_LINE_ERROR_VALUE_CHARS` chars
/// UTF-8-aware (Pass 3 M''3 anti-DoS payload-size).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CsvLineError {
    /// Numéro de ligne absolu dans le fichier (1-based, incluant header
    /// et lignes embarquées dans des champs quoted multi-lignes).
    pub line: usize,
    /// Code d'erreur stable (clé i18n via `i18n_key()`).
    pub code: CsvLineErrorCode,
    /// Valeur fautive si applicable, tronquée à 100 chars UTF-8-aware
    /// + suffix `…` si dépassement.
    pub value: Option<String>,
    /// Clé i18n frontend, ex. `"bank-csv-errors-invalid-date"`.
    pub message_i18n_key: String,
}

impl CsvLineError {
    /// Construit un `CsvLineError` en appliquant la troncature
    /// UTF-8-aware sur `value` (Pass 3 M''3).
    pub fn new(line: usize, code: CsvLineErrorCode, value: Option<String>) -> Self {
        let truncated_value = value.map(|v| {
            let chars: Vec<char> = v.chars().collect();
            if chars.len() <= MAX_CSV_LINE_ERROR_VALUE_CHARS {
                v
            } else {
                let mut s: String = chars
                    .into_iter()
                    .take(MAX_CSV_LINE_ERROR_VALUE_CHARS)
                    .collect();
                s.push('…');
                s
            }
        });
        Self {
            line,
            code,
            value: truncated_value,
            message_i18n_key: code.i18n_key().to_string(),
        }
    }
}

/// Erreurs unifiées pour les parseurs `kesh-import` (CAMT.053 + CSV).
///
/// Permet à la couche supérieure (kesh-api) de gérer un seul type
/// d'erreur via les conversions `From<CamtError>` et `From<CsvError>`.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ImportError {
    /// Erreur du parser CAMT.053.
    #[error(transparent)]
    Camt(#[from] CamtError),

    /// Erreur du parser CSV.
    #[error(transparent)]
    Csv(#[from] CsvError),
}
