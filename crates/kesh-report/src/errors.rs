//! Erreurs métier de génération de rapports.

use chrono::NaiveDate;
use kesh_db::errors::DbError;
use rust_decimal::Decimal;
use thiserror::Error;

/// Erreur métier de génération de rapport.
///
/// Mappée vers `kesh_api::errors::AppError` côté handler (cf. spec §error-shapes).
#[derive(Debug, Error)]
pub enum ReportError {
    /// Erreur DB sous-jacente.
    #[error("erreur DB : {0}")]
    Db(#[from] DbError),

    /// L'exercice fiscal demandé n'existe pas pour cette company.
    #[error("fiscal_year introuvable (fiscal_year_id={fiscal_year_id})")]
    FiscalYearNotFound { fiscal_year_id: i64 },

    /// La période fournie est invalide (start > end, etc.).
    #[error("période invalide : {reason}")]
    PeriodInvalid { reason: String },

    /// La période demandée dépasse les bornes de l'exercice.
    #[error(
        "période hors exercice : start={requested_start} end={requested_end} fy=[{fy_start};{fy_end}]"
    )]
    PeriodOutOfFiscalYear {
        fy_start: NaiveDate,
        fy_end: NaiveDate,
        requested_start: NaiveDate,
        requested_end: NaiveDate,
    },

    /// La balance des comptes est déséquilibrée (invariant cassé — log error!).
    #[error("trial balance déséquilibrée : débit={total_debit} crédit={total_credit}")]
    TrialBalanceUnbalanced {
        total_debit: Decimal,
        total_credit: Decimal,
    },

    /// Échec interne de la génération PDF (`printpdf::save_to_bytes`, I/O, etc.).
    ///
    /// Story 9-2a T2.0 + Pass 3 ECH3-H2 : mappé vers `AppError::PdfGenerationFailed`
    /// (variant dédié `kesh-api/src/errors.rs:184`, code HTTP 500, i18n key
    /// `error-pdf-generation-failed`). Cohérent avec `kesh-qrbill::QrBillError::PdfGeneration`
    /// (DD-14 pattern). Le `String` porte le détail technique (loggé mais jamais
    /// exposé brut au client).
    #[error("échec génération PDF : {0}")]
    PdfGeneration(String),

    /// Échec interne de la génération CSV (`csv::Writer`, I/O flush, BOM write).
    ///
    /// Pass 1 code-review H1 (BH1-H1 + ECH1-M2 + AA1-M2) : variant dédié pour
    /// éviter qu'un échec d'export CSV soit présenté comme « Échec génération
    /// PDF » côté UI. Mappé vers `AppError::CsvGenerationFailed` (code HTTP 500,
    /// i18n key `error-csv-generation-failed`).
    #[error("échec génération CSV : {0}")]
    CsvGeneration(String),
}
