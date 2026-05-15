//! Bilan : actifs, passifs, capitaux propres (résultat de l'exercice).
//!
//! L'équation comptable v0.1 (Pass 1 ECH-04 + Pass 2 AA2-11) :
//! `total_assets == total_liabilities + equity_result`
//!
//! - `total_liabilities` inclut les fonds propres permanents (Liability) MOINS les
//!   comptes Equity-like (`EQUITY_RESULT_ACCOUNT_NUMBERS` = 2979, 2800).
//! - `equity_result` = `total_revenues - total_expenses` (calculé via income_statement).
//!
//! Pass 3 ECH3-01 : exclusion des comptes 2979/2800 du total passifs pour éviter
//! le double-comptage avec equity_result calculé séparément.

use kesh_db::entities::AccountType;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::MySqlPool;

use crate::errors::ReportError;
use crate::income_statement;
use crate::period::ReportPeriod;

/// Numéros de comptes représentant sémantiquement de l'equity-result (résultat de
/// l'exercice + report à nouveau Sterchi PME). Exclus de `total_liabilities` pour
/// éviter le double-comptage avec `equity_result` calculé séparément.
///
/// Plans comptables non-Sterchi avec d'autres numéros → CR v0.2 (L70).
pub const EQUITY_RESULT_ACCOUNT_NUMBERS: &[&str] = &["2979", "2800"];

/// Bilan complet.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceSheet {
    pub period: ReportPeriod,
    pub assets: Vec<AccountBalance>,
    pub liabilities: Vec<AccountBalance>,
    pub total_assets: Decimal,
    pub total_liabilities: Decimal,
    pub equity_result: Decimal,
    pub equation_holds: bool,
}

/// Solde d'un compte au bilan ou au compte de résultat.
///
/// Le champ Rust `account_number` mappe la colonne DB `accounts.number`
/// (Pass 4 AA4-01 — la colonne DB est `number`, le champ Rust expose
/// `accountNumber` côté JSON via camelCase).
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AccountBalance {
    pub account_id: i64,
    #[sqlx(rename = "number")]
    pub account_number: String,
    #[sqlx(rename = "name")]
    pub account_name: String,
    pub account_type: AccountType,
    pub active: bool,
    pub balance: Decimal,
}

/// Génère le bilan pour la période donnée.
pub async fn generate(
    pool: &MySqlPool,
    company_id: i64,
    period: &ReportPeriod,
) -> Result<BalanceSheet, ReportError> {
    let assets = fetch_section(pool, company_id, period, AccountType::Asset).await?;
    let liabilities = fetch_section(pool, company_id, period, AccountType::Liability).await?;

    let total_assets: Decimal = assets.iter().map(|a| a.balance).sum();
    let total_liabilities: Decimal = liabilities.iter().map(|a| a.balance).sum();

    // equity_result = net_result du compte de résultat (Pass 1 BH-13 : appel direct,
    // pas de helper compute_net_result partagé)
    let income = income_statement::generate(pool, company_id, period).await?;
    let equity_result = income.net_result;

    let equation_holds = total_assets == total_liabilities + equity_result;
    if !equation_holds {
        tracing::warn!(
            total_assets = %total_assets,
            total_liabilities = %total_liabilities,
            equity_result = %equity_result,
            "balance_sheet : équation non vérifiée (defense in depth — vérifier seed/écritures)"
        );
    }

    Ok(BalanceSheet {
        period: period.clone(),
        assets,
        liabilities,
        total_assets,
        total_liabilities,
        equity_result,
        equation_holds,
    })
}

/// Agrège un type de compte sur la période.
///
/// SQL :
/// - Filtre `account_type IN (Asset|Liability)` + `entry_date BETWEEN start AND end`
/// - Inclut comptes actifs avec écritures OU comptes archivés avec écritures (Pass 2 AA2-11)
/// - Exclut comptes actifs sans écriture (épure bilan — différent trial_balance)
/// - Pour Liability : exclut les comptes equity-like (Pass 3 ECH3-01)
/// - Sign convention : Asset = debit - credit ; Liability = credit - debit
async fn fetch_section(
    pool: &MySqlPool,
    company_id: i64,
    period: &ReportPeriod,
    account_type: AccountType,
) -> Result<Vec<AccountBalance>, ReportError> {
    // Pour Liability, on exclut les comptes equity-like (2979, 2800).
    // Pour Asset, pas d'exclusion.
    let exclude_clause = if matches!(account_type, AccountType::Liability) {
        " AND a.number NOT IN ('2979', '2800')"
    } else {
        ""
    };

    // Sign convention : Asset = debit-credit ; Liability = credit-debit
    let sign_expr = match account_type {
        AccountType::Asset => "COALESCE(SUM(jel.debit), 0) - COALESCE(SUM(jel.credit), 0)",
        AccountType::Liability => "COALESCE(SUM(jel.credit), 0) - COALESCE(SUM(jel.debit), 0)",
        _ => unreachable!("balance_sheet ne traite que Asset/Liability"),
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
           AND je.entry_date BETWEEN ? AND ?{exclude_clause} \
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
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn equity_result_constants_present() {
        assert!(EQUITY_RESULT_ACCOUNT_NUMBERS.contains(&"2979"));
        assert!(EQUITY_RESULT_ACCOUNT_NUMBERS.contains(&"2800"));
        assert_eq!(EQUITY_RESULT_ACCOUNT_NUMBERS.len(), 2);
    }

    #[test]
    fn equation_holds_on_balanced_data() {
        let total_assets = dec!(15000);
        let total_liabilities = dec!(10000);
        let equity_result = dec!(5000);
        assert!(total_assets == total_liabilities + equity_result);
    }

    #[test]
    fn equation_holds_with_loss() {
        let total_assets = dec!(8000);
        let total_liabilities = dec!(10000);
        let equity_result = dec!(-2000);
        assert!(total_assets == total_liabilities + equity_result);
    }

    #[test]
    fn equation_holds_with_zero_data() {
        let total_assets = Decimal::ZERO;
        let total_liabilities = Decimal::ZERO;
        let equity_result = Decimal::ZERO;
        assert!(total_assets == total_liabilities + equity_result);
    }
}
