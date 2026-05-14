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

/// Helper pur (sans DB) qui applique la résolution asymétrique + validations sur
/// des bornes de fiscal_year déjà connues. Extrait pour permettre les unit tests
/// inline T1.5 (code review Pass 1 patch P9).
///
/// Table asymétrique 4 cas (Pass 1 ECH-02) :
/// - `(None, None)` → `(fy_start, fy_end)`
/// - `(Some(s), None)` → `(s, fy_end)`
/// - `(None, Some(e))` → `(fy_start, e)`
/// - `(Some(s), Some(e))` → `(s, e)`
///
/// Validations (Pass 3 ECH3-03) :
/// - chaque borne explicite doit être dans `[fy_start, fy_end]`
/// - après résolution, `start ≤ end` obligatoire
pub(crate) fn resolve_dates(
    fy_start: NaiveDate,
    fy_end: NaiveDate,
    period_start: Option<NaiveDate>,
    period_end: Option<NaiveDate>,
) -> Result<(NaiveDate, NaiveDate), ReportError> {
    if let Some(s) = period_start {
        if s < fy_start || s > fy_end {
            return Err(ReportError::PeriodOutOfFiscalYear {
                fy_start,
                fy_end,
                requested_start: s,
                requested_end: period_end.unwrap_or(fy_end),
            });
        }
    }
    if let Some(e) = period_end {
        if e < fy_start || e > fy_end {
            return Err(ReportError::PeriodOutOfFiscalYear {
                fy_start,
                fy_end,
                requested_start: period_start.unwrap_or(fy_start),
                requested_end: e,
            });
        }
    }

    let start = period_start.unwrap_or(fy_start);
    let end = period_end.unwrap_or(fy_end);

    if start > end {
        return Err(ReportError::PeriodInvalid {
            reason: format!("periodStart ({start}) doit être ≤ periodEnd ({end})"),
        });
    }

    Ok((start, end))
}

impl ReportPeriod {
    /// Résout la période effective d'un rapport.
    ///
    /// Table de résolution asymétrique 4 cas (Pass 1 ECH-02) — cf. `resolve_dates`.
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
        .ok_or(ReportError::FiscalYearNotFound { fiscal_year_id })?;

        let (start, end) = resolve_dates(fy.start_date, fy.end_date, period_start, period_end)?;

        Ok(Self {
            fiscal_year_id,
            start_date: start,
            end_date: end,
        })
    }
}

#[cfg(test)]
mod tests {
    //! 7 unit tests T1.5 (code review Pass 1 P9 — comblement gap spec).
    //! Tous testent `resolve_dates` (helper pur, sans DB). La résolution complète
    //! `ReportPeriod::resolve` est couverte par T11 sqlx + T10 E2E HTTP.
    use super::*;

    fn fy() -> (NaiveDate, NaiveDate) {
        (
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        )
    }

    #[test]
    fn default_period() {
        let (fs, fe) = fy();
        let (s, e) = resolve_dates(fs, fe, None, None).unwrap();
        assert_eq!(s, fs);
        assert_eq!(e, fe);
    }

    #[test]
    fn partial_period_both() {
        let (fs, fe) = fy();
        let ps = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
        let pe = NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();
        let (s, e) = resolve_dates(fs, fe, Some(ps), Some(pe)).unwrap();
        assert_eq!(s, ps);
        assert_eq!(e, pe);
    }

    #[test]
    fn partial_period_start_only() {
        let (fs, fe) = fy();
        let ps = NaiveDate::from_ymd_opt(2026, 4, 1).unwrap();
        let (s, e) = resolve_dates(fs, fe, Some(ps), None).unwrap();
        assert_eq!(s, ps);
        assert_eq!(e, fe);
    }

    #[test]
    fn partial_period_end_only() {
        let (fs, fe) = fy();
        let pe = NaiveDate::from_ymd_opt(2026, 9, 30).unwrap();
        let (s, e) = resolve_dates(fs, fe, None, Some(pe)).unwrap();
        assert_eq!(s, fs);
        assert_eq!(e, pe);
    }

    #[test]
    fn period_out_of_fy_end() {
        let (fs, fe) = fy();
        let pe = NaiveDate::from_ymd_opt(2027, 1, 15).unwrap();
        let err = resolve_dates(fs, fe, None, Some(pe)).unwrap_err();
        assert!(matches!(err, ReportError::PeriodOutOfFiscalYear { .. }));
    }

    #[test]
    fn period_out_of_fy_start() {
        let (fs, fe) = fy();
        let ps = NaiveDate::from_ymd_opt(2025, 12, 1).unwrap();
        let err = resolve_dates(fs, fe, Some(ps), None).unwrap_err();
        assert!(matches!(err, ReportError::PeriodOutOfFiscalYear { .. }));
    }

    #[test]
    fn period_inversed() {
        let (fs, fe) = fy();
        let ps = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let pe = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
        let err = resolve_dates(fs, fe, Some(ps), Some(pe)).unwrap_err();
        assert!(matches!(err, ReportError::PeriodInvalid { .. }));
    }

    #[test]
    fn period_same_day_is_valid() {
        let (fs, fe) = fy();
        let d = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let (s, e) = resolve_dates(fs, fe, Some(d), Some(d)).unwrap();
        assert_eq!(s, d);
        assert_eq!(e, d);
    }
}
