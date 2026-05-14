//! Résolution de la période d'un rapport.
//!
//! `ReportPeriod` représente la fenêtre temporelle effective d'un rapport, dérivée
//! d'un `fiscal_year_id` (obligatoire) + `period_start`/`period_end` (optionnels).
//! La résolution applique une table asymétrique 4 cas (Pass 1 ECH-02).

use chrono::NaiveDate;
use serde::Serialize;
use sqlx::MySqlPool;

use crate::errors::ReportError;

/// Période effective d'un rapport, résolue depuis le fiscal_year + les bornes optionnelles.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportPeriod {
    pub fiscal_year_id: i64,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

impl ReportPeriod {
    /// Résout la période effective d'un rapport.
    ///
    /// Table de résolution asymétrique 4 cas (Pass 1 ECH-02) :
    /// - `(None, None)` → `(fy.start_date, fy.end_date)` (exercice complet)
    /// - `(Some(s), None)` → `(s, fy.end_date)`
    /// - `(None, Some(e))` → `(fy.start_date, e)`
    /// - `(Some(s), Some(e))` → `(s, e)` après validations
    ///
    /// Validations : bornes incluses dans fy, ordre `start ≤ end`.
    pub async fn resolve(
        pool: &MySqlPool,
        company_id: i64,
        fiscal_year_id: i64,
        period_start: Option<NaiveDate>,
        period_end: Option<NaiveDate>,
    ) -> Result<Self, ReportError> {
        // Pass 1 BH-02 : ordre params `(pool, company_id, id)` cohérent avec
        // crates/kesh-db/src/repositories/fiscal_years.rs:399
        let fy = kesh_db::repositories::fiscal_years::find_by_id_in_company(
            pool,
            company_id,
            fiscal_year_id,
        )
        .await?
        .ok_or(ReportError::FiscalYearNotFound)?;

        // Pré-validation : si une borne explicite est hors fy, rejeter
        // (Pass 3 ECH3-03 : message d'erreur plus précis avant la résolution asymétrique)
        if let Some(s) = period_start {
            if s < fy.start_date || s > fy.end_date {
                return Err(ReportError::PeriodOutOfFiscalYear {
                    fy_start: fy.start_date,
                    fy_end: fy.end_date,
                    requested_start: s,
                    requested_end: period_end.unwrap_or(fy.end_date),
                });
            }
        }
        if let Some(e) = period_end {
            if e < fy.start_date || e > fy.end_date {
                return Err(ReportError::PeriodOutOfFiscalYear {
                    fy_start: fy.start_date,
                    fy_end: fy.end_date,
                    requested_start: period_start.unwrap_or(fy.start_date),
                    requested_end: e,
                });
            }
        }

        // Résolution asymétrique 4 cas
        let start = period_start.unwrap_or(fy.start_date);
        let end = period_end.unwrap_or(fy.end_date);

        // Ordre start ≤ end
        if start > end {
            return Err(ReportError::PeriodInvalid {
                reason: format!("periodStart ({start}) doit être ≤ periodEnd ({end})"),
            });
        }

        Ok(Self {
            fiscal_year_id,
            start_date: start,
            end_date: end,
        })
    }
}
