//! Erreurs émises par les parseurs `kesh-import`.
//!
//! `CamtError` représente uniquement les défaillances à la frontière du
//! parseur CAMT.053 (XML mal formé, version non supportée, champ requis
//! manquant). Les violations métier (devise non supportée par cette version,
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

    /// Au moins une ligne porte un montant de **signe contraire à sa
    /// colonne** (Story 26-1, #620) : un débit négatif sous
    /// [`DebitSign::Positive`](crate::csv::DebitSign::Positive), un débit
    /// positif sous [`DebitSign::Negative`](crate::csv::DebitSign::Negative),
    /// ou un crédit négatif quel que soit le réglage.
    ///
    /// **Refus de l'import entier, non confirmable** : contrairement à
    /// [`CsvError::PartialFailure`], aucune option « importer les lignes
    /// valides quand même » ne s'applique. Sous un mauvais réglage du sens,
    /// les seules lignes « valides » sont celles de l'autre colonne ; les
    /// importer seules reproduirait #620 (des dépenses absentes, un solde
    /// faux, sans un mot). Avant la 0.13.1, un débit déjà négatif était
    /// inversé en silence : ce refus rend visible le défaut à qui n'a pas
    /// réglé son profil.
    ///
    /// - `errors` : les lignes en cause, **propres au signe** (plafonnées
    ///   à [`MAX_CSV_LINE_ERRORS`], indépendamment des autres erreurs de
    ///   ligne du fichier, qui ne sont pas rapportées — régler le profil
    ///   d'abord) ;
    /// - `total_errors` : nombre total de lignes en défaut de signe ;
    /// - `truncated` : `total_errors > MAX_CSV_LINE_ERRORS` ;
    /// - `whole_debit_column` : parmi les lignes dont le montant Débit a été
    ///   lu, au moins **deux** débits non nuls et **tous** en défaut de
    ///   signe. Vrai, il désigne un mauvais réglage du profil ; faux (dont
    ///   le cas d'un seul débit en défaut, qui peut être un remboursement),
    ///   il désigne une ou quelques lignes — une extourne, par exemple —
    ///   qu'aucun réglage ne rend importables dans cette version.
    #[error("Montant de signe contraire à sa colonne : {total_errors} ligne(s)")]
    AmountSignMismatch {
        errors: Vec<CsvLineError>,
        total_errors: usize,
        truncated: bool,
        whole_debit_column: bool,
    },

    /// La **dernière ligne ignorée** (`header_row_count` du profil) porte
    /// déjà une date lisible au format du profil dans la colonne date : le
    /// nombre de lignes à ignorer est trop grand, et une transaction
    /// aurait disparu sans un mot (Story 26-1, #621).
    ///
    /// `line` suit la sémantique des numéros de ligne du lecteur
    /// (cf. [`CsvLineError::line`]).
    ///
    /// ⚠️ **Angle mort assumé** : la garde ne voit qu'une ligne avalée dont
    /// la cellule date **se lit**. Une ligne avalée à date vide, illisible,
    /// ou trop courte pour porter la colonne date, reste muette — ce n'est
    /// pas une détection de la ligne de titres, le nombre reste celui du
    /// profil.
    #[error(
        "La ligne {line}, ignorée, porte déjà une date : le nombre de lignes à ignorer est trop grand"
    )]
    HeaderSkipsData { line: usize },

    /// Le relevé porte **plusieurs devises** parmi ses lignes valides
    /// (Story 27-2, #624) — seulement quand le profil lit une colonne
    /// Devise (`ColumnMapping::currency`).
    ///
    /// **Refus de l'import entier, non confirmable**, comme
    /// [`CsvError::AmountSignMismatch`] : un relevé de compte est dans une
    /// seule devise ; importer les seules lignes de l'une d'elles ferait
    /// mentir le solde sans un mot.
    ///
    /// `currencies` : les devises distinctes, **triées**, au plus
    /// [`MAX_REPORTED_CURRENCIES`](crate::csv::parser::MAX_REPORTED_CURRENCIES)
    /// — les premières de l'ensemble complet trié, sans indicateur de
    /// troncature (C-27-2-13). Seules les lignes **valides** révèlent leur
    /// devise : une ligne écartée pour sa date ou son montant n'est pas
    /// lue jusqu'à sa devise (C-27-2-14).
    #[error("Le relevé porte plusieurs devises ({}) : un relevé CSV doit être dans une seule devise", currencies.join(", "))]
    MixedCurrencies { currencies: Vec<String> },

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
    /// Montant de signe contraire à sa colonne (Story 26-1, #620) : débit
    /// négatif sous `DebitSign::Positive`, débit positif sous
    /// `DebitSign::Negative`, crédit négatif. Jamais rapporté dans un
    /// [`CsvError::PartialFailure`] : il refuse l'import entier par
    /// [`CsvError::AmountSignMismatch`] (pourquoi : cf. ce variant).
    AmountSignMismatch,
    /// Cellule Devise illisible (Story 27-2, #624) : autre chose que trois
    /// lettres ASCII (`Fr.`, `€`, `CH`, `CHF1`). Erreur de ligne ordinaire
    /// (rejet partiel, confirmable).
    InvalidCurrency,
}

impl CsvLineErrorCode {
    /// Tous les variants, dans l'ordre de déclaration. La liste est tenue
    /// à la main ; sa complétude est gardée par le test `all_is_complete`,
    /// qui la compare à la liste des variants que la dérivation serde tire
    /// **de l'enum lui-même** (`deserialize_enum`) : un variant neuf oublié
    /// ici fait rougir ce test. (Le `match` exhaustif de [`Self::as_str`]
    /// ne garde que `as_str`, pas cette liste.)
    pub fn all() -> &'static [Self] {
        &[
            CsvLineErrorCode::InvalidDate,
            CsvLineErrorCode::InvalidAmount,
            CsvLineErrorCode::AmbiguousDebitCredit,
            CsvLineErrorCode::EmptyMandatoryField,
            CsvLineErrorCode::RowTooShort,
            CsvLineErrorCode::AmountSignMismatch,
            CsvLineErrorCode::InvalidCurrency,
        ]
    }

    /// Graphie canonique `SCREAMING_SNAKE_CASE` du code, identique à sa
    /// sérialisation serde (`INVALID_DATE`, `AMOUNT_SIGN_MISMATCH`…). C'est
    /// elle qui sort sur le fil (`details.lines[].code`,
    /// `warnings.invalidLines.lines[].code`) — Story 26-1, AC3-bis : avant,
    /// l'API émettait `format!("{:?}").to_uppercase()` (`INVALIDDATE`).
    pub fn as_str(&self) -> &'static str {
        match self {
            CsvLineErrorCode::InvalidDate => "INVALID_DATE",
            CsvLineErrorCode::InvalidAmount => "INVALID_AMOUNT",
            CsvLineErrorCode::AmbiguousDebitCredit => "AMBIGUOUS_DEBIT_CREDIT",
            CsvLineErrorCode::EmptyMandatoryField => "EMPTY_MANDATORY_FIELD",
            CsvLineErrorCode::RowTooShort => "ROW_TOO_SHORT",
            CsvLineErrorCode::AmountSignMismatch => "AMOUNT_SIGN_MISMATCH",
            CsvLineErrorCode::InvalidCurrency => "INVALID_CURRENCY",
        }
    }

    /// Clé i18n associée au code (préfixe `bank-import-csv-errors-`).
    pub fn i18n_key(&self) -> &'static str {
        match self {
            CsvLineErrorCode::InvalidDate => "bank-import-csv-errors-invalid-date",
            CsvLineErrorCode::InvalidAmount => "bank-import-csv-errors-invalid-amount",
            CsvLineErrorCode::AmbiguousDebitCredit => {
                "bank-import-csv-errors-ambiguous-debit-credit"
            }
            CsvLineErrorCode::EmptyMandatoryField => "bank-import-csv-errors-empty-mandatory-field",
            CsvLineErrorCode::RowTooShort => "bank-import-csv-errors-row-too-short",
            CsvLineErrorCode::AmountSignMismatch => "bank-import-csv-errors-amount-sign-mismatch",
            CsvLineErrorCode::InvalidCurrency => "bank-import-csv-errors-invalid-currency",
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
    ///
    /// C'est la ligne **où commence la lecture** de l'enregistrement (le
    /// lecteur `csv` pose la position avant de lire) : un enregistrement
    /// **précédé d'une ou de plusieurs lignes vides** est rapporté au
    /// numéro de la **première** de ces lignes vides, le lecteur ne les
    /// avalant qu'en lisant l'enregistrement suivant. Tout autre
    /// enregistrement porte son numéro physique, celui d'un éditeur
    /// (Story 26-1, AC4 — fixé par test, non corrigé).
    pub line: usize,
    /// Code d'erreur stable (clé i18n via `i18n_key()`).
    pub code: CsvLineErrorCode,
    /// Valeur fautive si applicable, tronquée à 100 chars UTF-8-aware
    /// + suffix `…` si dépassement.
    pub value: Option<String>,
    /// Clé i18n frontend, ex. `"bank-import-csv-errors-invalid-date"`.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Noms de variants que la dérivation `serde::Deserialize` tire de
    /// l'enum lui-même : elle les passe à `Deserializer::deserialize_enum`,
    /// que ce désérialiseur intercepte. Aucune liste tenue à la main.
    fn serde_variant_names() -> &'static [&'static str] {
        use serde::de::{self, Deserialize, Deserializer, Visitor};
        use std::cell::Cell;

        struct Capture<'a>(&'a Cell<Option<&'static [&'static str]>>);
        impl<'de> Deserializer<'de> for Capture<'_> {
            type Error = de::value::Error;
            fn deserialize_any<V: Visitor<'de>>(self, _: V) -> Result<V::Value, Self::Error> {
                Err(de::Error::custom("seul deserialize_enum est attendu"))
            }
            fn deserialize_enum<V: Visitor<'de>>(
                self,
                _name: &'static str,
                variants: &'static [&'static str],
                _visitor: V,
            ) -> Result<V::Value, Self::Error> {
                self.0.set(Some(variants));
                Err(de::Error::custom("variants capturés"))
            }
            serde::forward_to_deserialize_any! {
                bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
                bytes byte_buf option unit unit_struct newtype_struct seq tuple
                tuple_struct map struct identifier ignored_any
            }
        }

        let cell = Cell::new(None);
        let _ = CsvLineErrorCode::deserialize(Capture(&cell));
        cell.get()
            .expect("la dérivation serde de CsvLineErrorCode passe par deserialize_enum")
    }

    /// Garde de complétude de [`CsvLineErrorCode::all`] (AC14-9, revue P1
    /// F1/E-3) : `all()`, rendue par `as_str`, doit égaler, dans l'ordre,
    /// la liste des variants que serde tire de l'enum. Un variant ajouté à
    /// l'enum et oublié dans `all()` fait rougir ce test — l'ancienne
    /// version comptait les éléments de `all()` elle-même et restait verte.
    #[test]
    fn all_is_complete() {
        let from_all: Vec<&str> = CsvLineErrorCode::all()
            .iter()
            .map(CsvLineErrorCode::as_str)
            .collect();
        assert_eq!(from_all, serde_variant_names());
    }

    /// AC3-bis : pour chaque variant, la graphie `as_str()` est celle de
    /// serde — les deux ne peuvent plus diverger.
    #[test]
    fn as_str_matches_serde_for_every_variant() {
        for code in CsvLineErrorCode::all() {
            let json = serde_json::to_value(code).unwrap();
            assert_eq!(
                json,
                serde_json::Value::String(code.as_str().to_string()),
                "{code:?}"
            );
        }
    }

    #[test]
    fn amount_sign_mismatch_i18n_key() {
        assert_eq!(
            CsvLineErrorCode::AmountSignMismatch.i18n_key(),
            "bank-import-csv-errors-amount-sign-mismatch"
        );
    }

    #[test]
    fn invalid_currency_i18n_key() {
        assert_eq!(
            CsvLineErrorCode::InvalidCurrency.i18n_key(),
            "bank-import-csv-errors-invalid-currency"
        );
        assert_eq!(
            CsvLineErrorCode::InvalidCurrency.as_str(),
            "INVALID_CURRENCY"
        );
    }
}
