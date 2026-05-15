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
}
