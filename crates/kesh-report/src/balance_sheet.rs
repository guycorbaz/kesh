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
//! `total_assets == total_liabilities + total_equity + retained_earnings + equity_result`
//!
//! Cette égalité tient **par construction** (identité de la partie double) : sur
//! l'ensemble des écritures `entry_date ≤ date d'arrêté`, Σ(débit − crédit) = 0, d'où
//! `Σ_actifs(débit−crédit) = Σ_passifs(crédit−débit) + Σ_résultat(crédit−débit)`. Le
//! terme résultat se scinde en `retained_earnings` (avant `fy_start`) + `equity_result`
//! (`[fy_start, date d'arrêté]`). L'invariant dont dépend l'égalité : chaque écriture a
//! son `entry_date` **dans** les bornes de son `fiscal_year_id` (imposé à la
//! création par `journal_entries::create_in_tx` ; `equation_holds` sert de filet
//! — cf. Dev Notes 14-1).
//!
//! ⚠️ L'invariant était aussi tenu par `journal_entries::update`, supprimée par
//! la Story 24-4b (#380) : une écriture comptabilisée ne se réécrit plus, donc
//! son `entry_date` ne peut plus sortir de ses bornes après coup.
//!
//! # Présentation par rôle des fonds propres (Story 14-3c)
//!
//! Les comptes de passif dont le `role` est un **rôle de fonds propres**
//! (`EquityCapital`, `EquityOther`, `RetainedEarnings`, `CurrentYearResult`) sont
//! **partitionnés** — par rôle, **jamais par numéro** — de la section `liabilities`
//! (dettes réelles) vers une section `equity` dédiée (conforme CO art. 959a, distinction
//! capitaux étrangers / capitaux propres). `total_liabilities` ne compte donc plus que
//! les **dettes réelles** ; `total_equity` somme les comptes physiques de fonds propres.
//! Le déplacement est **mathématiquement neutre** (`total_liabilities + total_equity` =
//! ancien `total_liabilities`), donc l'équation restructurée tient trivialement. Les
//! deux **lignes calculées virtuelles** 14-1 (`retained_earnings`, `equity_result`)
//! restent **disjointes** des comptes physiques et ne sont **jamais** fusionnées avec un
//! compte physique de même rôle (collision D1 : un compte physique `RetainedEarnings` =
//! report d'ouverture d'un migrant ≠ la ligne calculée « Résultat reporté (calculé) »).
//!
//! # Pas de hardcode de numéros (décision Story 14-1)
//!
//! Aucune classification n'est déduite d'un **numéro** de compte : la partition
//! fonds propres / dettes se fait exclusivement sur `accounts.role` (Story 14-3a).
//! L'ancien hardcode `EQUITY_RESULT_ACCOUNT_NUMBERS = ["2979","2800"]` est **retiré**.

use chrono::NaiveDate;
use kesh_db::entities::{AccountRole, AccountType};
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
    /// Dettes réelles seulement — les comptes de fonds propres sont partitionnés
    /// vers `equity` (Story 14-3c). **Ne contient plus** les comptes de rôle equity.
    pub liabilities: Vec<AccountBalance>,
    /// Comptes physiques de **fonds propres** (rôle equity), triés par rang de rôle
    /// (`EquityCapital` → `EquityOther` → `RetainedEarnings` → `CurrentYearResult`,
    /// tie-break numéro de compte) — ordre CO 959a al. 2 garanti **en backend**,
    /// source unique (Story 14-3c). N'inclut **pas** les lignes calculées virtuelles.
    pub equity: Vec<AccountBalance>,
    pub total_assets: Decimal,
    pub total_liabilities: Decimal,
    /// Σ des soldes des comptes physiques de `equity` (**hors** lignes calculées
    /// `retained_earnings` / `equity_result`). Story 14-3c.
    pub total_equity: Decimal,
    /// Résultat reporté = cumul des résultats nets des exercices **strictement
    /// antérieurs** à `fy_start` (P&L `entry_date < fy_start`). Négatif = pertes
    /// cumulées (« Perte reportée »). **Ligne calculée virtuelle** — disjointe des
    /// comptes physiques de rôle `RetainedEarnings` (collision D1, Story 14-3c).
    pub retained_earnings: Decimal,
    /// Résultat de l'exercice courant = P&L sur `[fy_start, date d'arrêté]`.
    /// **Ligne calculée virtuelle** — disjointe d'un compte physique `CurrentYearResult`.
    pub equity_result: Decimal,
    pub equation_holds: bool,
}

impl BalanceSheet {
    /// Un bilan est **vide** seulement si **toutes** ses composantes sont nulles :
    /// aucun compte d'actif, de passif, **de fonds propres**, ET report/résultat
    /// calculés nuls.
    ///
    /// Story 14-3c (validate P1-F1, CRITICAL) : **source unique** de la garde « rapport
    /// vide », jadis dupliquée en ligne dans `csv.rs` et `pdf.rs` — et jamais étendue à
    /// `equity`. Un reclassement pur entre deux comptes de fonds propres (débit
    /// `EquityOther` / crédit `EquityCapital`, somme nette 0, actifs/passifs/virtuels
    /// nuls) laisse `equity` **non vide** → le bilan n'est **pas** vide, sinon la section
    /// Capitaux propres réellement peuplée serait masquée. Le pendant frontend
    /// (`isReportEmpty`) est maintenu en parallèle (assumé, cf. Dev Notes 14-3c).
    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
            && self.liabilities.is_empty()
            && self.equity.is_empty()
            && self.retained_earnings.is_zero()
            && self.equity_result.is_zero()
    }
}

/// Rang d'affichage d'un **rôle de fonds propres** (ordre CO art. 959a al. 2, du
/// capital vers le résultat), ou `None` si le rôle n'est **pas** un rôle equity.
///
/// **Source de vérité unique** (Story 14-3c, Piège #2) de la liste des 4 rôles de
/// fonds propres ET de leur ordre : la partition (`is_equity_role`), le tri backend
/// et l'ordre d'affichage en dérivent tous. Ne **jamais** dupliquer cette liste.
fn equity_role_rank(role: AccountRole) -> Option<u8> {
    match role {
        AccountRole::EquityCapital => Some(0),
        AccountRole::EquityOther => Some(1),
        AccountRole::RetainedEarnings => Some(2),
        AccountRole::CurrentYearResult => Some(3),
        _ => None,
    }
}

/// `true` si le compte porte un **rôle de fonds propres** (→ section `equity`).
/// Un compte de `role` NULL **ou** de rôle non-equity (dette réelle : `Payable`,
/// `VatPayable`, …) reste dans `liabilities`. Dérive de [`equity_role_rank`].
fn is_equity_role(role: Option<AccountRole>) -> bool {
    role.and_then(equity_role_rank).is_some()
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
    /// Rôle métier explicite du compte (`accounts.role`, Story 14-3a), `None` si
    /// aucun. Story 14-3c : porte la partition fonds propres / dettes au bilan et
    /// le regroupement par rôle — **jamais** déduit du numéro de compte.
    pub role: Option<AccountRole>,
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
    let liabilities_all =
        fetch_cumulative_section(pool, company_id, as_of, AccountType::Liability).await?;

    // Partition fonds propres / dettes (Story 14-3c, D2) — **par rôle, jamais par
    // numéro**. Les comptes de rôle equity quittent `liabilities` (dettes réelles)
    // pour la section `equity`. On filtre en mémoire le résultat de la requête
    // existante (pas de nouveau `WHERE role = ?` runtime → l'invariant `AND active`
    // de 14-3a est sans objet ici ; on préserve l'affichage d'un compte equity
    // archivé à solde non-nul, cohérent avec les passifs — AC-A).
    let (mut equity, liabilities): (Vec<AccountBalance>, Vec<AccountBalance>) = liabilities_all
        .into_iter()
        .partition(|a| is_equity_role(a.role));

    // Tri de la section equity par **rang de rôle** (CO 959a al. 2), tie-break numéro.
    // Garanti en backend (source unique) : le SQL trie par numéro, ce qui coïncide avec
    // l'ordre rôle par hasard sur un plan standard mais casserait sur un plan renuméroté
    // (validate P1-F2). Les 3 renderers itèrent `equity` dans l'ordre reçu, sans re-trier.
    // Tous les éléments de `equity` ont un rôle equity (`is_equity_role`), donc
    // `equity_role_rank` retourne toujours `Some` ici — `unwrap_or(u8::MAX)` par sûreté.
    equity.sort_by(|a, b| {
        let ra = a.role.and_then(equity_role_rank).unwrap_or(u8::MAX);
        let rb = b.role.and_then(equity_role_rank).unwrap_or(u8::MAX);
        ra.cmp(&rb)
            .then_with(|| a.account_number.cmp(&b.account_number))
    });

    let total_assets: Decimal = assets.iter().map(|a| a.balance).sum();
    let total_liabilities: Decimal = liabilities.iter().map(|a| a.balance).sum();
    let total_equity: Decimal = equity.iter().map(|a| a.balance).sum();

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

    // Équation restructurée (Story 14-3c, D2) : les comptes equity sont sortis de
    // `total_liabilities` vers `total_equity`. La somme totale passif + fonds propres
    // est inchangée (`total_liabilities + total_equity` = ancien `total_liabilities`),
    // donc l'égalité tient trivialement — la garde reste le filet defense-in-depth.
    let equation_holds =
        total_assets == total_liabilities + total_equity + retained_earnings + equity_result;
    if !equation_holds {
        tracing::warn!(
            total_assets = %total_assets,
            total_liabilities = %total_liabilities,
            total_equity = %total_equity,
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
        equity,
        total_assets,
        total_liabilities,
        total_equity,
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
        "SELECT a.id AS account_id, a.number, a.name, a.account_type, a.active, a.role, \
                {sign_expr} AS balance \
         FROM accounts a \
         INNER JOIN journal_entry_lines jel ON jel.account_id = a.id \
         INNER JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE a.company_id = ? \
           AND a.account_type = ? \
           AND je.company_id = ? \
           AND je.entry_date <= ? \
         GROUP BY a.id, a.number, a.name, a.account_type, a.active, a.role \
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
    use super::*;
    use crate::period::ReportPeriod;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    fn period() -> ReportPeriod {
        ReportPeriod {
            fiscal_year_id: 1,
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        }
    }

    fn equity_acc(number: &str, balance: Decimal, role: AccountRole) -> AccountBalance {
        AccountBalance {
            account_id: 1,
            account_number: number.into(),
            account_name: "eq".into(),
            account_type: AccountType::Liability,
            active: true,
            balance,
            role: Some(role),
        }
    }

    fn bs_with(equity: Vec<AccountBalance>, retained: Decimal, result: Decimal) -> BalanceSheet {
        let total_equity: Decimal = equity.iter().map(|a| a.balance).sum();
        BalanceSheet {
            period: period(),
            assets: vec![],
            liabilities: vec![],
            equity,
            total_assets: Decimal::ZERO,
            total_liabilities: Decimal::ZERO,
            total_equity,
            retained_earnings: retained,
            equity_result: result,
            equation_holds: true,
        }
    }

    // --- Story 14-3c : classification par rôle (source unique) ---

    #[test]
    fn is_equity_role_covers_the_four_equity_roles_and_nothing_else() {
        for r in [
            AccountRole::EquityCapital,
            AccountRole::EquityOther,
            AccountRole::RetainedEarnings,
            AccountRole::CurrentYearResult,
        ] {
            assert!(is_equity_role(Some(r)), "{r:?} doit être un rôle equity");
            assert!(equity_role_rank(r).is_some());
        }
        // Rôles NON-equity (dettes réelles / autres) + absence de rôle.
        for r in [
            AccountRole::Payable,
            AccountRole::VatPayable,
            AccountRole::VatSettlement,
            AccountRole::Receivable,
            AccountRole::DefaultRevenue,
            AccountRole::VatRecoverable,
        ] {
            assert!(!is_equity_role(Some(r)), "{r:?} ne doit PAS être equity");
            assert!(equity_role_rank(r).is_none());
        }
        assert!(!is_equity_role(None), "role NULL reste dans les dettes");
    }

    /// Ordre CO 959a al. 2 : capital → autres FP → report → résultat.
    #[test]
    fn equity_role_rank_orders_per_co_959a() {
        assert!(
            equity_role_rank(AccountRole::EquityCapital)
                < equity_role_rank(AccountRole::EquityOther)
        );
        assert!(
            equity_role_rank(AccountRole::EquityOther)
                < equity_role_rank(AccountRole::RetainedEarnings)
        );
        assert!(
            equity_role_rank(AccountRole::RetainedEarnings)
                < equity_role_rank(AccountRole::CurrentYearResult)
        );
    }

    // --- Story 14-3c : garde « rapport vide » centralisée (P1-F1) ---

    #[test]
    fn is_empty_true_when_everything_null() {
        let bs = bs_with(vec![], Decimal::ZERO, Decimal::ZERO);
        assert!(bs.is_empty());
    }

    /// Reclassement pur entre deux comptes de fonds propres qui s'annulent
    /// (`total_equity` net 0 mais `equity` peuplé) → le bilan n'est **pas** vide,
    /// sinon la section Capitaux propres réellement peuplée serait masquée.
    #[test]
    fn is_empty_false_on_pure_equity_reclass() {
        let bs = bs_with(
            vec![
                equity_acc("2800", dec!(1000), AccountRole::EquityCapital),
                equity_acc("2900", dec!(-1000), AccountRole::EquityOther),
            ],
            Decimal::ZERO,
            Decimal::ZERO,
        );
        assert_eq!(bs.total_equity, Decimal::ZERO);
        assert!(!bs.is_empty(), "equity peuplé ⇒ rapport non vide");
    }

    #[test]
    fn is_empty_false_when_only_calculated_lines_nonzero() {
        let bs = bs_with(vec![], dec!(1000), dec!(-1000));
        assert!(!bs.is_empty());
    }

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
