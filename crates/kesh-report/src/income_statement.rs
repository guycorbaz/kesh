//! Compte de résultat : produits (Revenue), charges (Expense), résultat net.
//!
//! Sign convention :
//! - Revenue : `credit - debit`
//! - Expense : `debit - credit`
//! - `net_result = total_revenues - total_expenses`

use kesh_db::entities::AccountType;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::MySqlPool;

use crate::balance_sheet::AccountBalance;
use crate::errors::ReportError;
use crate::period::ReportPeriod;

/// Compte de résultat complet.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomeStatement {
    pub period: ReportPeriod,
    pub revenues: Vec<AccountBalance>,
    pub expenses: Vec<AccountBalance>,
    pub total_revenues: Decimal,
    pub total_expenses: Decimal,
    pub net_result: Decimal,
}

/// Génère le compte de résultat pour la période donnée.
pub async fn generate(
    pool: &MySqlPool,
    company_id: i64,
    period: &ReportPeriod,
) -> Result<IncomeStatement, ReportError> {
    let revenues = fetch_section(pool, company_id, period, AccountType::Revenue).await?;
    let expenses = fetch_section(pool, company_id, period, AccountType::Expense).await?;

    let total_revenues: Decimal = revenues.iter().map(|a| a.balance).sum();
    let total_expenses: Decimal = expenses.iter().map(|a| a.balance).sum();
    let net_result = total_revenues - total_expenses;

    Ok(IncomeStatement {
        period: period.clone(),
        revenues,
        expenses,
        total_revenues,
        total_expenses,
        net_result,
    })
}

/// Agrège un type de compte sur la période.
///
/// SQL : `ORDER BY a.number ASC` (Pass 1 AA-01 — tri stable AC #3).
async fn fetch_section(
    pool: &MySqlPool,
    company_id: i64,
    period: &ReportPeriod,
    account_type: AccountType,
) -> Result<Vec<AccountBalance>, ReportError> {
    // Sign convention : Revenue = credit-debit ; Expense = debit-credit
    let sign_expr = match account_type {
        AccountType::Revenue => "COALESCE(SUM(jel.credit), 0) - COALESCE(SUM(jel.debit), 0)",
        AccountType::Expense => "COALESCE(SUM(jel.debit), 0) - COALESCE(SUM(jel.credit), 0)",
        _ => unreachable!("income_statement ne traite que Revenue/Expense"),
    };

    let sql = format!(
        "SELECT a.id AS account_id, a.number, a.name, a.account_type, a.active, \
                {sign_expr} AS balance \
         FROM accounts a \
         INNER JOIN journal_entry_lines jel ON jel.account_id = a.id \
         INNER JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE a.company_id = ? \
           AND a.account_type = ? \
           AND je.company_id = ? \
           AND je.fiscal_year_id = ? \
           AND je.entry_date BETWEEN ? AND ? \
         GROUP BY a.id, a.number, a.name, a.account_type, a.active \
         HAVING balance != 0 \
         ORDER BY a.number ASC"
    );

    let rows = sqlx::query_as::<_, AccountBalance>(&sql)
        .bind(company_id)
        .bind(account_type.as_str())
        .bind(company_id)
        .bind(period.fiscal_year_id)
        .bind(period.start_date)
        .bind(period.end_date)
        .fetch_all(pool)
        .await
        .map_err(kesh_db::errors::map_db_error)?;

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    #[test]
    fn net_result_positive() {
        let total_revenues = dec!(10000);
        let total_expenses = dec!(7000);
        assert_eq!(total_revenues - total_expenses, dec!(3000));
    }

    #[test]
    fn net_result_negative() {
        let total_revenues = dec!(5000);
        let total_expenses = dec!(7000);
        assert_eq!(total_revenues - total_expenses, dec!(-2000));
    }

    #[test]
    fn net_result_zero() {
        let total_revenues = Decimal::ZERO;
        let total_expenses = Decimal::ZERO;
        assert_eq!(total_revenues - total_expenses, Decimal::ZERO);
    }
}
