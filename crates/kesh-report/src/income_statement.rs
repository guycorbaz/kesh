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

use crate::balance_sheet::{AccountBalance, sign_exprs};
use crate::errors::ReportError;
use crate::opening::is_bilan;
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
    // Sign convention : Revenue = credit-debit ; Expense = debit-credit — la même
    // fonction que le bilan, qui construit aussi le solde en devise (Story 37-4).
    assert!(
        !is_bilan(account_type),
        "income_statement ne traite que Revenue/Expense"
    );
    let (sign_expr, sign_expr_fx) = sign_exprs(account_type);

    let sql = format!(
        // `a.role` inclus car `AccountBalance` (struct partagée avec le bilan) porte le
        // champ `role` depuis Story 14-3c ; son `FromRow` exige la colonne dans le
        // result-set (le compte de résultat ne l'exploite pas, mais doit la décoder).
        // Story 37-4 : `a.currency` et le solde en devise, bornés comme le CHF
        // (exercice ET dates) ; un compte à CHF 0 et devise ≠ 0 reste listé.
        "SELECT a.id AS account_id, a.number, a.name, a.account_type, a.active, a.role, \
                a.currency, {sign_expr} AS balance, {sign_expr_fx} AS balance_currency \
         FROM accounts a \
         INNER JOIN journal_entry_lines jel ON jel.account_id = a.id \
         INNER JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE a.company_id = ? \
           AND a.account_type = ? \
           AND je.company_id = ? \
           AND je.fiscal_year_id = ? \
           AND je.entry_date BETWEEN ? AND ? \
         GROUP BY a.id, a.number, a.name, a.account_type, a.active, a.role, a.currency \
         HAVING balance != 0 OR balance_currency != 0 \
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

    /// Code review Pass 1 patch P10 — comblement gap spec AC #3 / T3.4.
    ///
    /// AC #3 garantit le tri `accountNumber ASC` côté SQL via la clause
    /// `ORDER BY a.number ASC` dans la requête SQL de `fetch_section`. Ce test
    /// vérifie de manière prouvable (au niveau code source) que cette clause
    /// reste présente — toute régression qui supprimerait ce ORDER BY ferait
    /// échouer le test au compile-time via `include_str!`.
    ///
    /// La couverture fonctionnelle DB-side (vérifier que MariaDB retourne
    /// effectivement les rows triés) est faite par `balance_sheet_orders_accounts_by_number`
    /// dans `crates/kesh-api/tests/reports_e2e.rs` (T10).
    #[test]
    fn net_result_ordering_by_account_number() {
        const SRC: &str = include_str!("income_statement.rs");
        assert!(
            SRC.contains("ORDER BY a.number ASC"),
            "fetch_section SQL must keep `ORDER BY a.number ASC` clause (AC #3 — tri stable)"
        );
    }
}
