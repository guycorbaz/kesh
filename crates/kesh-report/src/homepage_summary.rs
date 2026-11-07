//! La synthèse de la page d'accueil : revenus et charges par mois, postes de
//! charges, pour l'exercice ouvert à la date du jour (Story 33-3a, #918). Elle
//! nourrit les deux graphiques de la Story 33-3b.
//!
//! # L'invariant (C-e332-10)
//!
//! `total_revenues` et `total_expenses` égalent ceux du compte de résultat de
//! l'exercice ([`crate::income_statement`]) : mêmes jointures, mêmes filtres, même
//! sens des montants (`Revenue` = crédit − débit, `Expense` = débit − crédit). Un
//! chiffre d'accueil qu'on ne peut pas recouper avec un rapport ne se défend pas.
//!
//! # Ce qui est écarté, et ce qui ne l'est pas
//!
//! - **Une écriture et sa contre-passation du même exercice** s'écartent
//!   **ensemble** (C-e332-9). Écarter un seul côté fausserait un mois ; une paire
//!   vaut zéro, compte par compte (miroir figé, pas de chaîne), si bien que
//!   l'invariant tient. Un côté daté hors des bornes de l'exercice n'appartient pas à
//!   la lecture : l'autre reste compté, comme au compte de résultat.
//! - **Une paire à cheval sur deux exercices reste comptée** de part et d'autre :
//!   c'est l'usage de `reverse_at_next_fiscal_year_start` (les transitoires), et
//!   rien d'autre que l'exercice ne distingue une transitoire d'une correction.
//! - **Un avoir reste compté** (C-e332-12) : il a sa propre écriture, sans
//!   `reverses_entry_id`, et peut être partiel. Les montants mensuels sont donc
//!   **signés**.
//! - **Aucune exclusion au titre de la clôture** (C-e332-8) : Kesh n'en passe
//!   aucune, `fiscal_years::close` est un verrou. Un test de caractérisation
//!   (`closing_a_year_writes_no_entry_and_changes_nothing`) tombera si cela change.
//!
//! # Le camembert
//!
//! Seuls les comptes de solde strictement positif sont des parts (C-e332-13). Un
//! compte créditeur est compté à part ; un compte à solde nul ne paraît nulle
//! part, comme au compte de résultat (`HAVING balance != 0`).
//!
//! Aucune écriture, aucune trace d'audit : l'accueil se charge à chaque visite
//! (C-e332-11).
//!
//! ⚠️ **Limite** : les deux requêtes se suivent sans instantané commun, comme les deux
//! sections du compte de résultat. Une écriture passée entre elles sépare, pour une
//! lecture, les barres du camembert ; les totaux, tirés de la seule requête mensuelle,
//! restent cohérents avec leurs mois, et la lecture suivante rattrape.

use chrono::{Datelike, Months, NaiveDate};
use kesh_db::entities::fiscal_year::FiscalYearStatus;
use kesh_db::errors::map_db_error;
use kesh_db::repositories::fiscal_years;
use rust_decimal::{Decimal, RoundingStrategy};
use serde::Serialize;
use sqlx::MySqlPool;

use crate::errors::ReportError;

/// Nombre de postes nommés du camembert ; le reste va dans « Autres ».
const NAMED_POSTS: usize = 5;

/// Écarte une écriture et sa contre-passation **du même exercice**, des deux côtés
/// (C-e332-9). Fragment partagé par les deux requêtes de ce module, qui lient deux
/// fois les bornes de l'exercice à sa suite.
///
/// La paire se juge comme le compte de résultat juge une écriture : exercice **et**
/// date dans ses bornes. Si l'un des côtés porte une date anormale (l'anomalie que
/// signale la préparation de clôture), le compte de résultat ne retient que l'autre :
/// la synthèse le retient aussi, sans quoi l'invariant tomberait.
const REVERSAL_PAIR_EXCLUSION: &str = "\
    AND NOT EXISTS (SELECT 1 FROM journal_entries o \
                     WHERE o.id = je.reverses_entry_id AND o.fiscal_year_id = je.fiscal_year_id \
                       AND o.entry_date BETWEEN ? AND ?) \
    AND NOT EXISTS (SELECT 1 FROM journal_entries r \
                     WHERE r.reverses_entry_id = je.id AND r.fiscal_year_id = je.fiscal_year_id \
                       AND r.entry_date BETWEEN ? AND ?) ";

/// L'état de la synthèse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SummaryState {
    /// Un exercice ouvert couvre la date du jour.
    Open,
    /// Aucun exercice ne couvre le jour, ou celui qui le couvre est clos.
    NoOpenFiscalYear,
}

/// L'exercice lu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryFiscalYear {
    pub id: i64,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

/// Les revenus et les dépenses d'un mois civil, montants signés.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthlySummary {
    /// `AAAA-MM`.
    pub month: String,
    /// Premier jour du mois, borné à l'exercice.
    pub from: NaiveDate,
    /// Dernier jour du mois, borné à l'exercice.
    pub to: NaiveDate,
    pub revenues: Decimal,
    pub expenses: Decimal,
}

/// La synthèse complète. Tout est à zéro et vide si `state` vaut
/// [`SummaryState::NoOpenFiscalYear`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HomepageSummary {
    pub state: SummaryState,
    pub as_of: NaiveDate,
    pub fiscal_year: Option<SummaryFiscalYear>,
    pub months: Vec<MonthlySummary>,
    pub total_revenues: Decimal,
    pub total_expenses: Decimal,
    /// Au plus [`NAMED_POSTS`], du plus gros au plus petit ; à égalité, par numéro.
    pub expense_posts: Vec<ExpensePost>,
    /// `None` si au plus [`NAMED_POSTS`] comptes ont un solde positif.
    pub other_expenses: Option<OtherExpenses>,
    /// Somme des soldes positifs : la base des pourcentages.
    pub expense_posts_base: Decimal,
    /// Comptes de charges au solde créditeur, écartés du camembert.
    pub credit_balance_expense_accounts: i64,
}

/// Les bornes d'un mois civil touché par un exercice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonthBounds {
    /// `AAAA-MM`.
    pub month: String,
    pub from: NaiveDate,
    pub to: NaiveDate,
}

/// Le solde d'un compte de charges sur l'exercice, tel que lu.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct ExpenseBalance {
    pub account_id: i64,
    pub number: String,
    pub name: String,
    pub balance: Decimal,
}

/// Une part du camembert : un compte.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpensePost {
    pub account_id: i64,
    pub number: String,
    pub name: String,
    pub amount: Decimal,
    /// Pourcentage de la base, une décimale toujours écrite.
    pub share: Decimal,
}

/// La part « Autres » : les comptes au-delà des postes nommés.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OtherExpenses {
    pub amount: Decimal,
    pub share: Decimal,
    pub account_count: i64,
}

/// Le classement rendu par [`rank_expense_posts`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RankedPosts {
    pub posts: Vec<ExpensePost>,
    pub others: Option<OtherExpenses>,
    pub base: Decimal,
    pub credit_balance_accounts: i64,
}

/// La ligne de la requête mensuelle.
#[derive(Debug, sqlx::FromRow)]
struct MonthRow {
    month: String,
    revenues: Decimal,
    expenses: Decimal,
}

/// Génère la synthèse de `company_id` à la date `today`.
///
/// `today` est un paramètre pour les tests et pour que le service, seul, lise
/// l'horloge. L'exercice se lit par `find_covering_date`, sans verrou : **pas** par
/// `find_open_covering_date`, qui prend un `FOR UPDATE`. Un exercice qui couvre le
/// jour mais qui est clos mène à [`SummaryState::NoOpenFiscalYear`], sans bascule
/// sur un autre exercice (C-e332-15).
pub async fn generate(
    pool: &MySqlPool,
    company_id: i64,
    today: NaiveDate,
) -> Result<HomepageSummary, ReportError> {
    let covering = fiscal_years::find_covering_date(pool, company_id, today).await?;
    let Some(fy) = covering.filter(|fy| fy.status == FiscalYearStatus::Open) else {
        return Ok(HomepageSummary {
            state: SummaryState::NoOpenFiscalYear,
            as_of: today,
            fiscal_year: None,
            months: Vec::new(),
            total_revenues: Decimal::ZERO,
            total_expenses: Decimal::ZERO,
            expense_posts: Vec::new(),
            other_expenses: None,
            expense_posts_base: Decimal::ZERO,
            credit_balance_expense_accounts: 0,
        });
    };

    let rows = fetch_months(pool, company_id, fy.id, fy.start_date, fy.end_date).await?;
    let months: Vec<MonthlySummary> = calendar_months(fy.start_date, fy.end_date)
        .into_iter()
        .map(|b| {
            let row = rows.iter().find(|r| r.month == b.month);
            MonthlySummary {
                revenues: row.map_or(Decimal::ZERO, |r| r.revenues),
                expenses: row.map_or(Decimal::ZERO, |r| r.expenses),
                month: b.month,
                from: b.from,
                to: b.to,
            }
        })
        .collect();
    let total_revenues = months.iter().map(|m| m.revenues).sum();
    let total_expenses = months.iter().map(|m| m.expenses).sum();

    let balances =
        fetch_expense_balances(pool, company_id, fy.id, fy.start_date, fy.end_date).await?;
    let ranked = rank_expense_posts(balances);

    Ok(HomepageSummary {
        state: SummaryState::Open,
        as_of: today,
        fiscal_year: Some(SummaryFiscalYear {
            id: fy.id,
            name: fy.name,
            start_date: fy.start_date,
            end_date: fy.end_date,
        }),
        months,
        total_revenues,
        total_expenses,
        expense_posts: ranked.posts,
        other_expenses: ranked.others,
        expense_posts_base: ranked.base,
        credit_balance_expense_accounts: ranked.credit_balance_accounts,
    })
}

/// Revenus et dépenses par mois de `entry_date`, hors paires de l'exercice.
async fn fetch_months(
    pool: &MySqlPool,
    company_id: i64,
    fiscal_year_id: i64,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<Vec<MonthRow>, ReportError> {
    let sql = format!(
        "SELECT DATE_FORMAT(je.entry_date, '%Y-%m') AS month, \
                COALESCE(SUM(CASE WHEN a.account_type = 'Revenue' \
                                  THEN jel.credit - jel.debit ELSE 0 END), 0) AS revenues, \
                COALESCE(SUM(CASE WHEN a.account_type = 'Expense' \
                                  THEN jel.debit - jel.credit ELSE 0 END), 0) AS expenses \
         FROM accounts a \
         INNER JOIN journal_entry_lines jel ON jel.account_id = a.id \
         INNER JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE a.company_id = ? \
           AND a.account_type IN ('Revenue', 'Expense') \
           AND je.company_id = ? \
           AND je.fiscal_year_id = ? \
           AND je.entry_date BETWEEN ? AND ? \
           {REVERSAL_PAIR_EXCLUSION}\
         GROUP BY month"
    );
    let rows = sqlx::query_as::<_, MonthRow>(&sql)
        .bind(company_id)
        .bind(company_id)
        .bind(fiscal_year_id)
        .bind(start)
        .bind(end)
        .bind(start)
        .bind(end)
        .bind(start)
        .bind(end)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)?;
    Ok(rows)
}

/// Le solde de chaque compte de charges sur l'exercice, hors paires. Sans filtre
/// `active` : un compte archivé qui porte des écritures reste un poste, comme au
/// compte de résultat.
async fn fetch_expense_balances(
    pool: &MySqlPool,
    company_id: i64,
    fiscal_year_id: i64,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<Vec<ExpenseBalance>, ReportError> {
    let sql = format!(
        "SELECT a.id AS account_id, a.number, a.name, \
                COALESCE(SUM(jel.debit), 0) - COALESCE(SUM(jel.credit), 0) AS balance \
         FROM accounts a \
         INNER JOIN journal_entry_lines jel ON jel.account_id = a.id \
         INNER JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE a.company_id = ? \
           AND a.account_type = 'Expense' \
           AND je.company_id = ? \
           AND je.fiscal_year_id = ? \
           AND je.entry_date BETWEEN ? AND ? \
           {REVERSAL_PAIR_EXCLUSION}\
         GROUP BY a.id, a.number, a.name \
         HAVING balance != 0"
    );
    let rows = sqlx::query_as::<_, ExpenseBalance>(&sql)
        .bind(company_id)
        .bind(company_id)
        .bind(fiscal_year_id)
        .bind(start)
        .bind(end)
        .bind(start)
        .bind(end)
        .bind(start)
        .bind(end)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)?;
    Ok(rows)
}

/// Un élément par mois civil touché par `[start, end]`, bornes rabattues sur
/// l'exercice. Un exercice n'a pas forcément douze mois (C-e332-14).
pub fn calendar_months(start: NaiveDate, end: NaiveDate) -> Vec<MonthBounds> {
    let mut months = Vec::new();
    let mut first = start.with_day(1).expect("le 1er existe toujours");
    while first <= end {
        let next = first
            .checked_add_months(Months::new(1))
            .expect("date dans les bornes de chrono");
        let last = next.pred_opt().expect("date dans les bornes de chrono");
        months.push(MonthBounds {
            month: format!("{:04}-{:02}", first.year(), first.month()),
            from: first.max(start),
            to: last.min(end),
        });
        first = next;
    }
    months
}

/// Classe les soldes : un solde `> 0` est une part, **tout le reste** compte comme
/// créditeur (la clause `HAVING balance != 0` a déjà écarté les zéros). Ordre : du
/// plus gros au plus petit ; à égalité, par numéro de compte.
pub fn rank_expense_posts(rows: Vec<ExpenseBalance>) -> RankedPosts {
    let (mut positive, others): (Vec<_>, Vec<_>) =
        rows.into_iter().partition(|r| r.balance > Decimal::ZERO);
    positive.sort_by(|a, b| {
        b.balance
            .cmp(&a.balance)
            .then_with(|| a.number.cmp(&b.number))
    });
    let base: Decimal = positive.iter().map(|r| r.balance).sum();
    let share_of = |amount: Decimal| share(amount, base);

    let tail = positive.split_off(positive.len().min(NAMED_POSTS));
    let others_amount: Decimal = tail.iter().map(|r| r.balance).sum();
    RankedPosts {
        others: (!tail.is_empty()).then(|| OtherExpenses {
            amount: others_amount,
            share: share_of(others_amount),
            account_count: tail.len() as i64,
        }),
        posts: positive
            .into_iter()
            .map(|r| ExpensePost {
                share: share_of(r.balance),
                account_id: r.account_id,
                number: r.number,
                name: r.name,
                amount: r.balance,
            })
            .collect(),
        base,
        credit_balance_accounts: others.len() as i64,
    }
}

/// `amount / base × 100`, arrondi une fois, à une décimale **toujours écrite** :
/// l'arrondi rend sa valeur telle quelle dès que l'échelle est ≤ 1, d'où le
/// `rescale`. N'est appelée qu'avec une base strictement positive.
fn share(amount: Decimal, base: Decimal) -> Decimal {
    let mut rounded = (amount * Decimal::ONE_HUNDRED / base)
        .round_dp_with_strategy(1, RoundingStrategy::MidpointAwayFromZero);
    rounded.rescale(1);
    rounded
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn bal(number: &str, balance: Decimal) -> ExpenseBalance {
        ExpenseBalance {
            account_id: number.parse().unwrap(),
            number: number.into(),
            name: format!("Compte {number}"),
            balance,
        }
    }

    fn shares(r: &RankedPosts) -> Vec<String> {
        r.posts.iter().map(|p| p.share.to_string()).collect()
    }

    #[test]
    fn civil_year_gives_twelve_months() {
        let m = calendar_months(d(2026, 1, 1), d(2026, 12, 31));
        assert_eq!(m.len(), 12);
        assert_eq!(m[0].month, "2026-01");
        assert_eq!((m[0].from, m[0].to), (d(2026, 1, 1), d(2026, 1, 31)));
        assert_eq!((m[11].from, m[11].to), (d(2026, 12, 1), d(2026, 12, 31)));
    }

    #[test]
    fn year_from_the_fifteenth_gives_thirteen_bounded_months() {
        let m = calendar_months(d(2026, 3, 15), d(2027, 3, 14));
        assert_eq!(m.len(), 13);
        assert_eq!((m[0].from, m[0].to), (d(2026, 3, 15), d(2026, 3, 31)));
        assert_eq!((m[12].from, m[12].to), (d(2027, 3, 1), d(2027, 3, 14)));
        assert_eq!(m[12].month, "2027-03");
    }

    #[test]
    fn eighteen_months_and_leap_february() {
        let m = calendar_months(d(2027, 7, 1), d(2028, 12, 31));
        assert_eq!(m.len(), 18);
        let feb = m.iter().find(|x| x.month == "2028-02").unwrap();
        assert_eq!(feb.to, d(2028, 2, 29), "février bissextile");
        let m = calendar_months(d(2026, 2, 1), d(2026, 2, 28));
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].to, d(2026, 2, 28));
    }

    #[test]
    fn single_day_year_gives_one_month() {
        let m = calendar_months(d(2026, 5, 10), d(2026, 5, 11));
        assert_eq!(m.len(), 1);
        assert_eq!((m[0].from, m[0].to), (d(2026, 5, 10), d(2026, 5, 11)));
    }

    #[test]
    fn seven_accounts_give_five_posts_and_others() {
        let r = rank_expense_posts(vec![
            bal("6300", dec!(30)),
            bal("6100", dec!(70)),
            bal("6600", dec!(10)),
            bal("6200", dec!(50)),
            bal("6400", dec!(40)),
            bal("6500", dec!(20)),
            bal("6050", dec!(60)),
        ]);
        let numbers: Vec<&str> = r.posts.iter().map(|p| p.number.as_str()).collect();
        assert_eq!(numbers, ["6100", "6050", "6200", "6400", "6300"]);
        let o = r.others.unwrap();
        assert_eq!(o.amount, dec!(30));
        assert_eq!(o.account_count, 2);
        assert_eq!(r.base, dec!(280));
    }

    #[test]
    fn five_accounts_leave_no_others_six_leave_one() {
        let five: Vec<_> = ["6500", "6400", "6300", "6200", "6100"]
            .into_iter()
            .map(|n| bal(n, dec!(10)))
            .collect();
        let r = rank_expense_posts(five.clone());
        assert_eq!(r.others, None);
        assert_eq!(r.posts[0].number, "6100", "à égalité, par numéro");
        let mut six = five;
        six.push(bal("6050", dec!(10)));
        let r = rank_expense_posts(six);
        assert_eq!(r.posts[0].number, "6050");
        assert_eq!(r.others.as_ref().unwrap().account_count, 1);
    }

    #[test]
    fn non_positive_balances_are_not_posts() {
        let r = rank_expense_posts(vec![bal("6000", dec!(100)), bal("6100", dec!(-300))]);
        assert_eq!(r.posts.len(), 1);
        assert_eq!(r.posts[0].share.to_string(), "100.0");
        assert_eq!(r.base, dec!(100));
        assert_eq!(r.credit_balance_accounts, 1);
    }

    #[test]
    fn no_positive_balance_divides_by_nothing() {
        let r = rank_expense_posts(vec![bal("6100", dec!(-5))]);
        assert!(r.posts.is_empty());
        assert_eq!(r.others, None);
        assert_eq!(r.base, Decimal::ZERO);
        assert_eq!(r.base.to_string(), "0");
        assert_eq!(r.credit_balance_accounts, 1);
    }

    #[test]
    fn shares_keep_one_written_decimal() {
        let r = rank_expense_posts(vec![bal("6000", dec!(200)), bal("6100", dec!(100))]);
        assert_eq!(shares(&r), ["66.7", "33.3"]);
        let r = rank_expense_posts(vec![bal("6000", dec!(100)), bal("6100", dec!(100))]);
        assert_eq!(shares(&r), ["50.0", "50.0"]);
        let r = rank_expense_posts(vec![bal("6000", dec!(42.5))]);
        assert_eq!(shares(&r), ["100.0"]);
    }

    #[test]
    fn share_rounds_midpoint_away_from_zero() {
        // 1 sur 16 : 6,25 %, jamais 6,2 %.
        let r = rank_expense_posts(vec![bal("6000", dec!(15)), bal("6100", dec!(1))]);
        assert_eq!(shares(&r), ["93.8", "6.3"]);
    }

    /// Le manuel dit « au plus trois dixièmes » : six parts, 0,05 chacune au plus. Ce
    /// cas atteint la borne (12,25 → 12,3 cinq fois, 38,75 → 38,8).
    #[test]
    fn shares_can_sum_to_three_tenths_above_one_hundred() {
        let r = rank_expense_posts(vec![
            bal("6000", dec!(155)),
            bal("6100", dec!(49)),
            bal("6200", dec!(49)),
            bal("6300", dec!(49)),
            bal("6400", dec!(49)),
            bal("6500", dec!(49)),
        ]);
        let others = r.others.as_ref().unwrap().share;
        assert_eq!(others.to_string(), "12.3");
        let sum: Decimal = r.posts.iter().map(|p| p.share).sum::<Decimal>() + others;
        assert_eq!(sum, dec!(100.3));
    }
}
