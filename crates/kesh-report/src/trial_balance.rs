//! Balance des comptes : pour chaque compte, **solde d'ouverture, mouvements de la
//! période, solde de clôture** — et une ligne calculée du résultat reporté.
//!
//! # Pourquoi l'ouverture (Story 25-5-b, #385)
//!
//! Jusqu'à la 25-5-b, la balance ne portait que les mouvements de la période :
//! le compte « clients » de la balance 2026 ne valait pas celui du bilan 2026,
//! qui est cumulatif depuis l'origine. Désormais `closing_balance` d'un compte de
//! bilan **est** son solde au bilan, et celui d'un compte de résultat son montant
//! au compte de résultat. La règle d'ouverture vit dans [`crate::opening`],
//! partagée avec le grand livre.
//!
//! # Règle d'inclusion
//!
//! Un compte est rendu s'il est **actif**, **ou** s'il a un mouvement dans la
//! période, **ou** si son **solde d'ouverture est non nul** — un compte archivé
//! qui porte encore un solde doit apparaître, sinon la clôture ne concorderait
//! plus avec le bilan. Un compte rendu archivé porte `active: false`.
//!
//! # La ligne calculée du résultat reporté
//!
//! Kesh ne passe aucune écriture de clôture : le résultat des exercices
//! antérieurs n'est porté par **aucun compte**. Sans lui, ni la colonne
//! d'ouverture ni celle de clôture ne s'équilibrent. [`TrialBalance::retained_earnings`]
//! le porte — la même valeur que le bilan, par la même fonction.
//!
//! # Deux contrôles
//!
//! - **Mouvements** : `total_debit == total_credit` (partie double, vérifiée à
//!   l'INSERT des écritures dans `kesh-db::repositories::journal_entries::create_in_tx`).
//!   Si déséquilibré → `ReportError::TrialBalanceUnbalanced` + log error! (defense
//!   in depth).
//! - **Ouverture** : `opening_balanced` (voir le champ). Un déséquilibre n'est pas
//!   une erreur : il est journalisé et affiché, comme `equation_holds` au bilan.
//!   Il n'y a pas de contrôle de clôture : la clôture vaut l'ouverture plus des
//!   mouvements équilibrés, ce serait la copie du premier.
//!
//! # Limite connue
//!
//! Le signe est lu sur le `account_type` **courant** du compte (issues #274, #382).

use kesh_db::entities::AccountType;
use kesh_db::repositories::fiscal_years;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::MySqlPool;

use crate::balance_sheet::fetch_retained_earnings;
use crate::errors::ReportError;
use crate::opening::{debit_sense, opening_from, signed};
use crate::period::ReportPeriod;

/// Balance des comptes complète.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrialBalance {
    pub period: ReportPeriod,
    pub rows: Vec<TrialBalanceRow>,
    /// Total des mouvements débit de la période.
    pub total_debit: Decimal,
    /// Total des mouvements crédit de la période.
    pub total_credit: Decimal,
    /// Mouvements équilibrés — toujours vrai dans un rapport rendu (sinon
    /// `TrialBalanceUnbalanced`).
    pub balanced: bool,
    /// Résultat reporté **calculé** : cumul des résultats des exercices
    /// strictement antérieurs à celui de la période (P&L `entry_date < fy_start`),
    /// signe crédit-positif — négatif = pertes cumulées. Même valeur que
    /// `BalanceSheet::retained_earnings`. Ouverture = clôture, sans mouvement.
    pub retained_earnings: Decimal,
    /// `Σ debit_sense(opening_balance) − retained_earnings == 0`. Le résultat
    /// reporté étant calculé **indépendamment**, c'est une vérification réelle :
    /// une ligne d'écriture déséquilibrée antérieure à la période la fait tomber.
    pub opening_balanced: bool,
}

/// Ligne de la balance des comptes. Tous les soldes suivent la convention du
/// dépôt : positifs du côté naturel du compte (`Asset`/`Expense` : débit).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrialBalanceRow {
    pub account_id: i64,
    /// `accounts.number`.
    pub account_number: String,
    /// `accounts.name`.
    pub account_name: String,
    pub account_type: AccountType,
    pub active: bool,
    /// Solde au début de la période — règle de [`crate::opening`].
    pub opening_balance: Decimal,
    /// Mouvements débit de la période.
    pub total_debit: Decimal,
    /// Mouvements crédit de la période.
    pub total_credit: Decimal,
    /// **Net des mouvements** de la période (pas un solde de compte).
    pub balance: Decimal,
    /// `opening_balance + balance` — pour un compte de bilan, son solde au bilan.
    pub closing_balance: Decimal,
}

/// Ligne brute, telle que la lit SQL : trois paires de sommes, aucun choix de
/// règle ni de signe (ils sont en Rust, dans [`crate::opening`]).
#[derive(Debug, sqlx::FromRow)]
struct RawRow {
    account_id: i64,
    number: String,
    name: String,
    account_type: AccountType,
    active: bool,
    before_debit: Decimal,
    before_credit: Decimal,
    since_fy_debit: Decimal,
    since_fy_credit: Decimal,
    period_debit: Decimal,
    period_credit: Decimal,
    period_lines: i64,
}

/// Génère la balance des comptes pour la période donnée.
pub async fn generate(
    pool: &MySqlPool,
    company_id: i64,
    period: &ReportPeriod,
) -> Result<TrialBalance, ReportError> {
    let fy = fiscal_years::find_by_id_in_company(pool, company_id, period.fiscal_year_id)
        .await?
        .ok_or(ReportError::FiscalYearNotFound {
            fiscal_year_id: period.fiscal_year_id,
        })?;
    let fy_start = fy.start_date;

    // Une requête agrégée : par compte, les sommes avant le début, depuis le début
    // de l'exercice, et sur la période. Les mouvements gardent leur filtre
    // d'origine (exercice ET dates).
    let sql = "
        SELECT
            a.id AS account_id,
            a.number,
            a.name,
            a.account_type,
            a.active,
            COALESCE(SUM(CASE WHEN x.entry_date < ? THEN x.debit END), 0) AS before_debit,
            COALESCE(SUM(CASE WHEN x.entry_date < ? THEN x.credit END), 0) AS before_credit,
            COALESCE(SUM(CASE WHEN x.entry_date >= ? AND x.entry_date < ? THEN x.debit END), 0)
                AS since_fy_debit,
            COALESCE(SUM(CASE WHEN x.entry_date >= ? AND x.entry_date < ? THEN x.credit END), 0)
                AS since_fy_credit,
            COALESCE(SUM(CASE WHEN x.fiscal_year_id = ? AND x.entry_date BETWEEN ? AND ?
                THEN x.debit END), 0) AS period_debit,
            COALESCE(SUM(CASE WHEN x.fiscal_year_id = ? AND x.entry_date BETWEEN ? AND ?
                THEN x.credit END), 0) AS period_credit,
            CAST(COALESCE(SUM(CASE WHEN x.fiscal_year_id = ? AND x.entry_date BETWEEN ? AND ?
                THEN 1 ELSE 0 END), 0) AS SIGNED) AS period_lines
        FROM accounts a
        LEFT JOIN (
            SELECT jel.account_id, jel.debit, jel.credit, je.entry_date, je.fiscal_year_id
            FROM journal_entry_lines jel
            INNER JOIN journal_entries je ON je.id = jel.entry_id
            WHERE je.company_id = ?
              AND je.entry_date <= ?
        ) AS x ON x.account_id = a.id
        WHERE a.company_id = ?
        GROUP BY a.id, a.number, a.name, a.account_type, a.active
        ORDER BY a.number ASC
    ";

    let (start, end) = (period.start_date, period.end_date);
    let raw = sqlx::query_as::<_, RawRow>(sql)
        .bind(start) // before_debit
        .bind(start) // before_credit
        .bind(fy_start)
        .bind(start) // since_fy_debit
        .bind(fy_start)
        .bind(start) // since_fy_credit
        .bind(period.fiscal_year_id)
        .bind(start)
        .bind(end) // period_debit
        .bind(period.fiscal_year_id)
        .bind(start)
        .bind(end) // period_credit
        .bind(period.fiscal_year_id)
        .bind(start)
        .bind(end) // period_lines
        .bind(company_id) // x: je.company_id
        .bind(end) // x: borne haute
        .bind(company_id) // a.company_id
        .fetch_all(pool)
        .await
        .map_err(kesh_db::errors::map_db_error)?;

    let rows: Vec<TrialBalanceRow> = raw
        .into_iter()
        .filter_map(|r| {
            let opening = opening_from(
                r.account_type,
                (r.before_debit, r.before_credit),
                (r.since_fy_debit, r.since_fy_credit),
            );
            // Règle d'inclusion : actif, ou mouvementé, ou porteur d'un solde.
            if !r.active && r.period_lines == 0 && opening.is_zero() {
                return None;
            }
            let balance = signed(r.account_type, r.period_debit, r.period_credit);
            Some(TrialBalanceRow {
                account_id: r.account_id,
                account_number: r.number,
                account_name: r.name,
                account_type: r.account_type,
                active: r.active,
                opening_balance: opening,
                total_debit: r.period_debit,
                total_credit: r.period_credit,
                balance,
                closing_balance: opening + balance,
            })
        })
        .collect();

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

    let retained_earnings = fetch_retained_earnings(pool, company_id, fy_start).await?;
    let opening_balanced = is_opening_balanced(&rows, retained_earnings);
    if !opening_balanced {
        tracing::warn!(
            company_id,
            fiscal_year_id = period.fiscal_year_id,
            retained_earnings = %retained_earnings,
            "trial_balance : les soldes d'ouverture ne s'équilibrent pas avec le résultat reporté"
        );
    }

    Ok(TrialBalance {
        period: period.clone(),
        rows,
        total_debit,
        total_credit,
        balanced,
        retained_earnings,
        opening_balanced,
    })
}

/// `Σ debit_sense(opening_balance) − retained_earnings == 0` : les soldes signés
/// par type sont ramenés en sens débit avant d'être additionnés.
fn is_opening_balanced(rows: &[TrialBalanceRow], retained_earnings: Decimal) -> bool {
    let sum: Decimal = rows
        .iter()
        .map(|r| debit_sense(r.account_type, r.opening_balance))
        .sum();
    sum - retained_earnings == Decimal::ZERO
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn row(account_type: AccountType, opening: Decimal) -> TrialBalanceRow {
        TrialBalanceRow {
            account_id: 0,
            account_number: String::new(),
            account_name: String::new(),
            account_type,
            active: true,
            opening_balance: opening,
            total_debit: Decimal::ZERO,
            total_credit: Decimal::ZERO,
            balance: Decimal::ZERO,
            closing_balance: opening,
        }
    }

    /// Ventes 1000 et charge 400 en exercice 1 (clients 1000, caisse −400, tous
    /// deux à l'actif) : ouvertures en sens débit 600 = résultat reporté 600.
    #[test]
    fn l_ouverture_s_equilibre_avec_le_resultat_reporte() {
        let rows = [
            row(AccountType::Asset, dec!(1000)),
            row(AccountType::Asset, dec!(-400)),
        ];
        assert!(is_opening_balanced(&rows, dec!(600)));
        assert!(!is_opening_balanced(&rows, dec!(500)));
    }

    /// Un passif créditeur de 5000 (report d'un migrant) face à une banque de
    /// 5000 : sans le re-signe en sens débit, la somme vaudrait 10000 et le
    /// contrôle rougirait sur des livres exacts.
    #[test]
    fn les_comptes_a_nature_creditrice_sont_re_signes() {
        let rows = [
            row(AccountType::Asset, dec!(5000)),
            row(AccountType::Liability, dec!(5000)),
        ];
        assert!(is_opening_balanced(&rows, Decimal::ZERO));
    }

    /// Exercice en cours : un produit de 200 avant la période (janvier–février)
    /// est porté par l'ouverture du compte de produits, PAS par le résultat
    /// reporté — compté une fois.
    #[test]
    fn le_resultat_de_l_exercice_avant_la_periode_est_compte_une_fois() {
        let rows = [
            row(AccountType::Asset, dec!(1200)),
            row(AccountType::Asset, dec!(-400)),
            row(AccountType::Revenue, dec!(200)),
        ];
        assert!(is_opening_balanced(&rows, dec!(600)));
    }
}
