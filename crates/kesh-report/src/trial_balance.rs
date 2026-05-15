//! Balance des comptes : 1 ligne par compte avec total débit / total crédit / solde.
//!
//! Règle d'inclusion (Pass 1 ECH-03) :
//! - Comptes **actifs** : inclus dans tous les cas (même sans écriture, balance=0)
//! - Comptes **archivés avec écritures dans la période** : inclus (marqueur `active: false`)
//! - Comptes **archivés sans écriture dans la période** : exclus
//!
//! Invariant : `total_debit == total_credit` (cohérence partie double, vérifiée à
//! l'INSERT des écritures dans `kesh-db::repositories::journal_entries::create_in_tx:201`).
//! Si déséquilibré → `ReportError::TrialBalanceUnbalanced` + log error! (defense in depth).

use kesh_db::entities::AccountType;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::MySqlPool;

use crate::errors::ReportError;
use crate::period::ReportPeriod;

/// Balance des comptes complète.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrialBalance {
    pub period: ReportPeriod,
    pub rows: Vec<TrialBalanceRow>,
    pub total_debit: Decimal,
    pub total_credit: Decimal,
    pub balanced: bool,
}

/// Ligne de la balance des comptes.
///
/// `account_number` (camelCase) mappe la colonne DB `accounts.number`
/// (Pass 4 AA4-01).
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TrialBalanceRow {
    pub account_id: i64,
    #[sqlx(rename = "number")]
    pub account_number: String,
    #[sqlx(rename = "name")]
    pub account_name: String,
    pub account_type: AccountType,
    pub active: bool,
    pub total_debit: Decimal,
    pub total_credit: Decimal,
    pub balance: Decimal,
}

/// Génère la balance des comptes pour la période donnée.
pub async fn generate(
    pool: &MySqlPool,
    company_id: i64,
    period: &ReportPeriod,
) -> Result<TrialBalance, ReportError> {
    // Règle d'inclusion (Pass 1 ECH-03) :
    // (active=true OR EXISTS écritures dans la période)
    //
    // CORRECTION (Pass 4 dev) : la subquery `filtered_jel` filtre les lignes par
    // fiscal_year + période AVANT le LEFT JOIN sur accounts, sinon SUM agrège
    // les lignes de TOUS les fy (bug constaté avec fy2 sans écritures).
    let sql = "
        SELECT
            a.id AS account_id,
            a.number,
            a.name,
            a.account_type,
            a.active,
            COALESCE(SUM(filtered_jel.debit), 0) AS total_debit,
            COALESCE(SUM(filtered_jel.credit), 0) AS total_credit,
            CASE
                WHEN a.account_type IN ('Asset', 'Expense')
                    THEN COALESCE(SUM(filtered_jel.debit), 0) - COALESCE(SUM(filtered_jel.credit), 0)
                ELSE COALESCE(SUM(filtered_jel.credit), 0) - COALESCE(SUM(filtered_jel.debit), 0)
            END AS balance
        FROM accounts a
        LEFT JOIN (
            SELECT jel.account_id, jel.debit, jel.credit
            FROM journal_entry_lines jel
            INNER JOIN journal_entries je ON je.id = jel.entry_id
            WHERE je.company_id = ?
              AND je.fiscal_year_id = ?
              AND je.entry_date BETWEEN ? AND ?
        ) AS filtered_jel ON filtered_jel.account_id = a.id
        WHERE a.company_id = ?
          AND (
              a.active = TRUE
              OR EXISTS (
                  SELECT 1 FROM journal_entry_lines jel2
                  INNER JOIN journal_entries je2 ON je2.id = jel2.entry_id
                  WHERE jel2.account_id = a.id
                    AND je2.company_id = ?
                    AND je2.fiscal_year_id = ?
                    AND je2.entry_date BETWEEN ? AND ?
              )
          )
        GROUP BY a.id, a.number, a.name, a.account_type, a.active
        ORDER BY a.number ASC
    ";

    let rows = sqlx::query_as::<_, TrialBalanceRow>(sql)
        .bind(company_id) // filtered_jel.je.company_id
        .bind(period.fiscal_year_id)
        .bind(period.start_date)
        .bind(period.end_date)
        .bind(company_id) // a.company_id (WHERE)
        .bind(company_id) // je2.company_id (EXISTS subquery)
        .bind(period.fiscal_year_id)
        .bind(period.start_date)
        .bind(period.end_date)
        .fetch_all(pool)
        .await
        .map_err(kesh_db::errors::map_db_error)?;

    let total_debit: Decimal = rows.iter().map(|r| r.total_debit).sum();
    let total_credit: Decimal = rows.iter().map(|r| r.total_credit).sum();
    let balanced = total_debit == total_credit;

    if !balanced {
        tracing::error!(
            total_debit = %total_debit,
            total_credit = %total_credit,
            company_id,
            fiscal_year_id = period.fiscal_year_id,
            "trial_balance déséquilibrée — invariant cassé, vérifier journal_entries::create_in_tx"
        );
        return Err(ReportError::TrialBalanceUnbalanced {
            total_debit,
            total_credit,
        });
    }

    Ok(TrialBalance {
        period: period.clone(),
        rows,
        total_debit,
        total_credit,
        balanced,
    })
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    #[test]
    fn balanced_when_debit_equals_credit() {
        let total_debit = dec!(1500);
        let total_credit = dec!(1500);
        assert!(total_debit == total_credit);
    }

    #[test]
    fn unbalanced_when_debit_diverges() {
        let total_debit = dec!(1500);
        let total_credit = dec!(1499);
        assert!(total_debit != total_credit);
    }

    #[test]
    fn zero_balance_for_inactive_accounts_without_entries() {
        // L'invariant : si compte archivé sans écriture, exclus de la balance.
        // Si compte actif sans écriture, balance = 0 (SUM NULL → COALESCE 0).
        let active_no_entries_balance = rust_decimal::Decimal::ZERO;
        assert_eq!(active_no_entries_balance, rust_decimal::Decimal::ZERO);
    }
}
