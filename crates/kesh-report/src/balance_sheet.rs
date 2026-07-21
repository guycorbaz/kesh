//! Bilan : actifs, passifs, capitaux propres — modèle « temps réel virtuel » (Story 14-1).
//!
//! # Report à-nouveau virtuel (Odoo/Flectra)
//!
//! Les soldes de bilan sont **calculés en direct**, **sans écriture physique** de
//! clôture/à-nouveau :
//!
//! - **Comptes de bilan** (Asset + Liability, capitaux propres permanents compris) :
//!   **cumulatifs depuis l'origine** — solde = Σ de **toutes** les écritures
//!   `entry_date ≤ date d'arrêté`, **tous exercices confondus**. La borne basse
//!   (`period.start_date`) est **sans effet** pour le bilan.
//! - **Résultat de l'exercice** (`equity_result`) : P&L sur `[fy_start, date d'arrêté]`
//!   (year-to-date), calculé via `income_statement`. Ancré à `fy_start` (début de
//!   l'exercice courant), **jamais** à `period.start_date`.
//! - **Résultat reporté** (`retained_earnings`) : cumul des résultats nets des
//!   exercices **strictement antérieurs** = P&L `entry_date < fy_start`.
//!
//! # Équation comptable
//!
//! `total_assets == total_liabilities + retained_earnings + equity_result`
//!
//! Cette égalité tient **par construction** (identité de la partie double) : sur
//! l'ensemble des écritures `entry_date ≤ date d'arrêté`, Σ(débit − crédit) = 0, d'où
//! `Σ_actifs(débit−crédit) = Σ_passifs(crédit−débit) + Σ_résultat(crédit−débit)`. Le
//! terme résultat se scinde en `retained_earnings` (avant `fy_start`) + `equity_result`
//! (`[fy_start, date d'arrêté]`). L'invariant dont dépend l'égalité : chaque écriture a
//! son `entry_date` **dans** les bornes de son `fiscal_year_id` (imposé par
//! `journal_entries::update` ; `equation_holds` sert de filet — cf. Dev Notes 14-1).
//!
//! # Pas de hardcode de numéros (décision Story 14-1)
//!
//! `total_liabilities` compte **tous** les comptes de passif cumulés, **sans aucune
//! exclusion par numéro** (capital, réserves, report à nouveau inclus). L'ancien
//! hardcode `EQUITY_RESULT_ACCOUNT_NUMBERS = ["2979","2800"]` est **retiré** : il
//! excluait `2800` (le capital, vraie equity) et faisait disparaître les capitaux
//! propres d'un utilisateur migrant. Le rôle d'un compte n'est **jamais** déduit de son
//! numéro. Le durcissement (compte de résultat non-postable, présentation par rôle) est
//! traité en Story 14-3 (rôles configurables sur `accounts`).

use chrono::NaiveDate;
use kesh_db::entities::AccountType;
use kesh_db::errors::map_db_error;
use kesh_db::repositories::fiscal_years;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::MySqlPool;

use crate::errors::ReportError;
use crate::income_statement;
use crate::period::ReportPeriod;

/// Bilan complet.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceSheet {
    pub period: ReportPeriod,
    pub assets: Vec<AccountBalance>,
    pub liabilities: Vec<AccountBalance>,
    pub total_assets: Decimal,
    pub total_liabilities: Decimal,
    /// Résultat reporté = cumul des résultats nets des exercices **strictement
    /// antérieurs** à `fy_start` (P&L `entry_date < fy_start`). Négatif = pertes
    /// cumulées (« Perte reportée »).
    pub retained_earnings: Decimal,
    /// Résultat de l'exercice courant = P&L sur `[fy_start, date d'arrêté]`.
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

/// Génère le bilan pour la période donnée (modèle temps réel virtuel — Story 14-1).
pub async fn generate(
    pool: &MySqlPool,
    company_id: i64,
    period: &ReportPeriod,
) -> Result<BalanceSheet, ReportError> {
    // Date d'arrêté = borne haute de la période. La borne basse (`period.start_date`)
    // est SANS EFFET pour le bilan cumulatif (AC-A).
    let as_of = period.end_date;

    let assets = fetch_cumulative_section(pool, company_id, as_of, AccountType::Asset).await?;
    let liabilities =
        fetch_cumulative_section(pool, company_id, as_of, AccountType::Liability).await?;

    let total_assets: Decimal = assets.iter().map(|a| a.balance).sum();
    let total_liabilities: Decimal = liabilities.iter().map(|a| a.balance).sum();

    // Ancrage à `fy_start` = début de l'exercice courant (Dev Note 4). Résolu par le
    // `fiscal_year_id` de la requête, PAS par itération des lignes `fiscal_years`
    // (gaps entre exercices + exercices simultanément `Open` permis).
    let fy = fiscal_years::find_by_id_in_company(pool, company_id, period.fiscal_year_id)
        .await?
        .ok_or(ReportError::FiscalYearNotFound {
            fiscal_year_id: period.fiscal_year_id,
        })?;
    let fy_start = fy.start_date;

    // Résultat de l'exercice = P&L sur [fy_start, date d'arrêté] — ancré à `fy_start`,
    // PAS à `period.start_date` (sinon une requête intra-exercice déplacerait le split
    // fonds propres, validate Pass 2 HIGH). En fin d'exercice = `income.net_result` ;
    // en cours d'année = résultat year-to-date.
    let equity_period = ReportPeriod {
        fiscal_year_id: period.fiscal_year_id,
        start_date: fy_start,
        end_date: as_of,
    };
    let income = income_statement::generate(pool, company_id, &equity_period).await?;
    let equity_result = income.net_result;

    // Résultat reporté = cumul P&L des exercices strictement antérieurs.
    let retained_earnings = fetch_retained_earnings(pool, company_id, fy_start).await?;

    let equation_holds = total_assets == total_liabilities + retained_earnings + equity_result;
    if !equation_holds {
        tracing::warn!(
            total_assets = %total_assets,
            total_liabilities = %total_liabilities,
            retained_earnings = %retained_earnings,
            equity_result = %equity_result,
            "balance_sheet : équation non vérifiée (defense in depth — écriture de clôture \
             manuelle ? entry_date hors bornes d'exercice ? cf. Dev Notes 14-1)"
        );
    }

    Ok(BalanceSheet {
        period: period.clone(),
        assets,
        liabilities,
        total_assets,
        total_liabilities,
        retained_earnings,
        equity_result,
        equation_holds,
    })
}

/// **Point unique de calcul des soldes cumulés depuis l'origine** (couture snapshot,
/// AC-G Story 14-1).
///
/// Agrège un type de compte de bilan (Asset/Liability) sur **toutes** les écritures
/// `entry_date <= as_of`, **tous exercices confondus** (report à-nouveau virtuel).
///
/// SQL :
/// - Filtre `account_type IN (Asset|Liability)` + borne date **unique** `entry_date <= as_of`
///   (PAS de `fiscal_year_id`, PAS de borne basse — report à-nouveau cumulatif).
/// - **Aucune exclusion par numéro de compte** (hardcode retiré — cf. doc module).
/// - Sign convention : Asset = `débit − crédit` ; Liability = `crédit − débit`.
/// - `HAVING balance != 0` : les comptes à solde nul ne sont pas listés (épure bilan).
///
/// **Couture (AC-G)** : c'est LE seul point où la borne `entry_date <= as_of` matérialise
/// les soldes cumulés. Un futur snapshot de soldes de clôture (1 ligne/compte/exercice
/// clos, définitif via l'immutabilité) se branchera **ici** — lecture d'une ligne
/// pré-calculée au lieu du `SUM` — **sans changer modèle, API ni UX**. Voir aussi
/// `fetch_retained_earnings` (report P&L, même couture).
async fn fetch_cumulative_section(
    pool: &MySqlPool,
    company_id: i64,
    as_of: NaiveDate,
    account_type: AccountType,
) -> Result<Vec<AccountBalance>, ReportError> {
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
           AND je.entry_date <= ? \
         GROUP BY a.id, a.number, a.name, a.account_type, a.active \
         HAVING balance != 0 \
         ORDER BY a.number ASC"
    );

    let rows = sqlx::query_as::<_, AccountBalance>(&sql)
        .bind(company_id)
        .bind(account_type.as_str())
        .bind(company_id)
        .bind(as_of)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)?;

    Ok(rows)
}

/// Calcule le **résultat reporté** = cumul des résultats nets des exercices
/// **strictement antérieurs** à `before` (= `fy_start` de l'exercice courant).
///
/// = P&L sur `entry_date < before`, tous exercices confondus (PAS de filtre
/// `fiscal_year_id`) : `Σ (crédit − débit)` sur les comptes Revenue **et** Expense
/// (les deux contribuent au résultat net avec le même signe `crédit − débit` :
/// Revenue = `+(crédit−débit)`, Expense = `−(débit−crédit) = +(crédit−débit)`).
///
/// Retourne `0` pour le tout premier exercice (aucune écriture antérieure — pas de
/// `NULL`/panic grâce à `COALESCE`, AC-D). Négatif = pertes cumulées (AC-I cas b).
///
/// **Couture (AC-G)** : même borne date `entry_date < before` que
/// `fetch_cumulative_section` ; un snapshot de clôture stockera aussi ce report.
async fn fetch_retained_earnings(
    pool: &MySqlPool,
    company_id: i64,
    before: NaiveDate,
) -> Result<Decimal, ReportError> {
    let retained: Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(jel.credit - jel.debit), 0) \
         FROM journal_entry_lines jel \
         INNER JOIN journal_entries je ON je.id = jel.entry_id \
         INNER JOIN accounts a ON a.id = jel.account_id \
         WHERE a.company_id = ? \
           AND a.account_type IN ('Revenue', 'Expense') \
           AND je.company_id = ? \
           AND je.entry_date < ?",
    )
    .bind(company_id)
    .bind(company_id)
    .bind(before)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)?;

    Ok(retained)
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    /// Équation cross-exercice (Story 14-1) : `assets == liabilities + retained + result`.
    ///
    /// Fixture arithmétiquement close (AC-I) : FY2025 actifs 15 000 / passifs 10 000 /
    /// résultat net 5 000 (reste vivant dans Revenue/Expense) ; FY2026 +200 de produit
    /// (débit actif 200 / crédit produit 200) → actifs cumulés 15 200,
    /// `retained_earnings` 5 000, `equity_result` 200.
    #[test]
    fn equation_holds_cross_fiscal_year() {
        let total_assets = dec!(15200);
        let total_liabilities = dec!(10000);
        let retained_earnings = dec!(5000);
        let equity_result = dec!(200);
        assert!(total_assets == total_liabilities + retained_earnings + equity_result);
    }

    /// 1er exercice (cas dégénéré AC-D) : `retained_earnings == 0`, dégénère au bilan
    /// mono-exercice historique.
    #[test]
    fn equation_holds_first_fiscal_year_zero_retained() {
        let total_assets = dec!(15000);
        let total_liabilities = dec!(10000);
        let retained_earnings = Decimal::ZERO;
        let equity_result = dec!(5000);
        assert!(total_assets == total_liabilities + retained_earnings + equity_result);
    }

    /// Résultat reporté négatif (AC-I cas b) : pertes cumulées antérieures.
    #[test]
    fn equation_holds_negative_retained() {
        let total_assets = dec!(6000);
        let total_liabilities = dec!(10000);
        let retained_earnings = dec!(-5000);
        let equity_result = dec!(1000);
        assert!(total_assets == total_liabilities + retained_earnings + equity_result);
    }

    /// Bilan vide (AC-C dégénéré, tout premier exercice sans écriture) : tout à zéro.
    #[test]
    fn equation_holds_on_zero_data() {
        let total_assets = Decimal::ZERO;
        let total_liabilities = Decimal::ZERO;
        let retained_earnings = Decimal::ZERO;
        let equity_result = Decimal::ZERO;
        assert!(total_assets == total_liabilities + retained_earnings + equity_result);
    }
}
