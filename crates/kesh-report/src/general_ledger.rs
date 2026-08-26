//! Grand livre — l'extrait d'un compte : ce qui a fait son solde, ligne à ligne.
//!
//! # Pourquoi ce rapport ne ressemble pas aux autres
//!
//! C'est le **premier rapport de Kesh qui franchit la borne d'exercice**, et c'est
//! délibéré : le bilan est cumulatif depuis l'origine, donc un grand livre borné
//! par exercice ne concorderait pas avec lui. Il n'utilise donc **pas**
//! [`crate::period::ReportPeriod`], qui exige un `fiscal_year_id` et refuse toute
//! borne hors exercice, mais [`LedgerPeriod`].
//!
//! # La règle du solde d'ouverture — le cœur du module
//!
//! Elle **diffère selon le type de compte**, et c'est ce qu'un lecteur pressé
//! prendra de travers :
//!
//! - `Asset` / `Liability` : cumul sur `entry_date < from`, **tous exercices
//!   confondus**, sans borne basse — même patron que
//!   [`crate::balance_sheet::generate`].
//! - `Revenue` / `Expense` : cumul depuis le **début de l'exercice contenant
//!   `from`** seulement. Si `from` est le premier jour d'un exercice, l'ouverture
//!   vaut 0.
//!
//! Un compte de bilan reporte son solde d'un exercice à l'autre ; un compte de
//! résultat est soldé au bouclement et repart de zéro. Cumuler un compte de
//! produits depuis l'origine donnerait un nombre qui ne correspond à **rien** —
//! ni au compte de résultat, ni à la balance, ni au bilan.
//!
//! ⚠️ **Kesh ne passe aucune écriture de clôture** : cette remise à zéro n'existe
//! que comme borne basse du `SUM`. Elle est **entièrement à la charge de ce
//! module**, et son échec serait **muet** — `closing = opening + mouvements`
//! resterait vrai, les totaux s'additionneraient, le rapport serait intérieurement
//! cohérent et extérieurement faux. C'est pourquoi les tests de concordance
//! appellent réellement `generate_balance_sheet`, `generate_trial_balance` et
//! `generate_income_statement`.
//!
//! # Deux pièges du schéma
//!
//! - `journal_entries.entry_number` est unique **par exercice** : il repart à 1.
//!   Le tri pose donc `fiscal_year_id` **avant** lui, sans quoi deux écritures de
//!   même date sur deux exercices s'entrelacent.
//! - `journal_entry_lines` **n'a aucun libellé** : il vient de l'écriture.
//!
//! # Limite connue
//!
//! Le signe est lu sur le `account_type` **courant** du compte. Retyper un compte
//! mouvementé re-signe donc silencieusement tout son historique — défaut
//! pré-existant, hors périmètre ici (issues #274 et #382).

use chrono::NaiveDate;
use kesh_db::entities::AccountType;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::MySqlPool;

use crate::errors::ReportError;

/// Plafond de lignes rendues en une fois, aligné sur le patron du dépôt.
///
/// ⚠️ Il borne **l'écran**, jamais l'export. Un grand livre tronqué n'est pas
/// un grand livre : l'Olico exige qu'il soit produisible **en entier**, et c'est
/// l'export qui porte cette obligation. D'où [`LedgerOptions::limit`] à `None`,
/// qui ne pose aucune borne.
pub const MAX_LEDGER_LIMIT: i64 = 500;

/// Fenêtre du grand livre — **sans exercice**, contrairement à `ReportPeriod`.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerPeriod {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

impl LedgerPeriod {
    /// Valide l'ordre des bornes. Aucune contrainte d'exercice : c'est le propre
    /// de ce rapport.
    pub fn new(from: NaiveDate, to: NaiveDate) -> Result<Self, ReportError> {
        if from > to {
            return Err(ReportError::PeriodInvalid {
                reason: format!("from ({from}) doit être ≤ to ({to})"),
            });
        }
        Ok(Self { from, to })
    }
}

/// Le grand livre : une section par compte.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralLedger {
    pub period: LedgerPeriod,
    pub sections: Vec<LedgerSection>,
}

/// L'extrait d'un compte sur la période.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerSection {
    pub account_id: i64,
    pub account_number: String,
    pub account_name: String,
    pub account_type: AccountType,
    /// `false` pour un compte archivé — il reste **inclus** s'il porte un solde.
    pub active: bool,
    /// Côté naturel du compte, pour l'affichage.
    pub balance_side: &'static str,
    pub opening: Decimal,
    pub lines: Vec<LedgerLine>,
    pub total_debit: Decimal,
    pub total_credit: Decimal,
    pub closing: Decimal,
    /// Vrai quand le solde de clôture est du côté contraire à la nature du
    /// compte — un compte de produits à solde débiteur, par exemple. C'est
    /// exactement l'anomalie que ce rapport doit rendre visible.
    pub unnatural_balance: bool,
    /// Ruptures d'exercice traversées, pour les comptes de résultat.
    pub fiscal_year_breaks: Vec<FiscalYearBreak>,
    /// Nombre total de lignes sur la période, **avant pagination**.
    pub line_count: i64,
}

/// Fin d'exercice traversée par la période, sur un compte de résultat : le solde
/// progressif y repart de zéro.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FiscalYearBreak {
    /// Premier jour du nouvel exercice.
    pub date: NaiveDate,
    pub closing_fiscal_year_id: i64,
    pub closing_balance: Decimal,
}

/// Une ligne d'écriture vue depuis le compte.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerLine {
    pub line_id: i64,
    pub entry_id: i64,
    pub entry_date: NaiveDate,
    pub fiscal_year_id: i64,
    /// Nom de l'exercice (`fiscal_years.name`). ⚠️ Il n'est pas décoratif : sans
    /// lui, un numéro de pièce est **ambigu** dès que la période traverse deux
    /// exercices, puisqu'il y repart à 1. C'est la pièce « n° 12 » de quel
    /// exercice ? La question doit se lire, pas se déduire.
    pub fiscal_year_name: String,
    /// ⚠️ Unique **par exercice** seulement.
    pub entry_number: i64,
    pub journal: String,
    /// Libellé de l'**écriture** : les lignes n'en portent pas.
    pub description: String,
    /// Comptes de l'écriture autres que celui-ci — ce qui rend l'extrait lisible.
    pub counterpart: Vec<String>,
    pub debit: Decimal,
    pub credit: Decimal,
    pub running_balance: Decimal,
}

/// Ligne brute, telle que la lit SQL.
#[derive(Debug, sqlx::FromRow)]
struct RawLine {
    line_id: i64,
    entry_id: i64,
    entry_date: NaiveDate,
    fiscal_year_id: i64,
    fiscal_year_name: String,
    entry_number: i64,
    journal: String,
    description: String,
    debit: Decimal,
    credit: Decimal,
}

#[derive(Debug, sqlx::FromRow)]
struct RawAccount {
    account_id: i64,
    number: String,
    name: String,
    account_type: AccountType,
    active: bool,
}

/// `true` si le compte a le débit pour côté naturel.
fn is_debit_natured(t: AccountType) -> bool {
    matches!(t, AccountType::Asset | AccountType::Expense)
}

/// Applique la convention de signe du dépôt — celle de la balance et du bilan.
fn signed(t: AccountType, debit: Decimal, credit: Decimal) -> Decimal {
    if is_debit_natured(t) {
        debit - credit
    } else {
        credit - debit
    }
}

/// Début de l'exercice contenant `date`, s'il en existe un.
///
/// Sert **uniquement** aux comptes de résultat : c'est la borne basse de leur
/// solde d'ouverture. Renvoie `None` si aucun exercice ne couvre la date —
/// l'appelant traite alors l'ouverture comme nulle.
async fn fiscal_year_start_containing(
    pool: &MySqlPool,
    company_id: i64,
    date: NaiveDate,
) -> Result<Option<NaiveDate>, ReportError> {
    let row: Option<(NaiveDate,)> = sqlx::query_as(
        "SELECT start_date FROM fiscal_years \
         WHERE company_id = ? AND start_date <= ? AND end_date >= ? \
         ORDER BY start_date DESC LIMIT 1",
    )
    .bind(company_id)
    .bind(date)
    .bind(date)
    .fetch_optional(pool)
    .await
    .map_err(kesh_db::errors::map_db_error)?;

    Ok(row.map(|r| r.0))
}

/// Solde d'ouverture d'un compte au `from` — **la règle centrale du module**.
///
/// Comptes de bilan : cumul depuis l'origine. Comptes de résultat : cumul depuis
/// le début de leur exercice. Voir la documentation du module pour le pourquoi.
async fn opening_balance(
    pool: &MySqlPool,
    company_id: i64,
    account_id: i64,
    account_type: AccountType,
    from: NaiveDate,
) -> Result<Decimal, ReportError> {
    // Borne basse : aucune pour un compte de bilan ; le début de l'exercice pour
    // un compte de résultat. `None` ⇒ ouverture nulle, la borne haute étant
    // exclusive et égale à la borne basse.
    let lower: Option<NaiveDate> = match account_type {
        AccountType::Asset | AccountType::Liability => None,
        AccountType::Revenue | AccountType::Expense => {
            match fiscal_year_start_containing(pool, company_id, from).await? {
                Some(start) => Some(start),
                // Aucun exercice ne couvre `from` : rien à reporter.
                None => return Ok(Decimal::ZERO),
            }
        }
    };

    let sql = match lower {
        Some(_) => {
            "SELECT COALESCE(SUM(jel.debit), 0) AS d, COALESCE(SUM(jel.credit), 0) AS c \
             FROM journal_entry_lines jel \
             INNER JOIN journal_entries je ON je.id = jel.entry_id \
             WHERE jel.account_id = ? AND je.company_id = ? \
               AND je.entry_date >= ? AND je.entry_date < ?"
        }
        None => {
            "SELECT COALESCE(SUM(jel.debit), 0) AS d, COALESCE(SUM(jel.credit), 0) AS c \
             FROM journal_entry_lines jel \
             INNER JOIN journal_entries je ON je.id = jel.entry_id \
             WHERE jel.account_id = ? AND je.company_id = ? \
               AND je.entry_date < ?"
        }
    };

    let mut q = sqlx::query_as::<_, (Decimal, Decimal)>(sql)
        .bind(account_id)
        .bind(company_id);
    if let Some(lo) = lower {
        q = q.bind(lo);
    }
    let (d, c) = q
        .bind(from)
        .fetch_one(pool)
        .await
        .map_err(kesh_db::errors::map_db_error)?;

    Ok(signed(account_type, d, c))
}

/// Comptes à rendre : ceux qui ont un mouvement sur la période **ou** un solde
/// d'ouverture non nul. Un compte explicitement demandé est toujours rendu.
async fn select_accounts(
    pool: &MySqlPool,
    company_id: i64,
    account_ids: Option<&[i64]>,
) -> Result<Vec<RawAccount>, ReportError> {
    let base = "SELECT a.id AS account_id, a.number, a.name, a.account_type, a.active \
                FROM accounts a WHERE a.company_id = ?";

    let rows = match account_ids {
        Some(ids) if !ids.is_empty() => {
            let placeholders = std::iter::repeat_n("?", ids.len())
                .collect::<Vec<_>>()
                .join(",");
            let sql = format!("{base} AND a.id IN ({placeholders}) ORDER BY a.number ASC");
            let mut q = sqlx::query_as::<_, RawAccount>(&sql).bind(company_id);
            for id in ids {
                q = q.bind(id);
            }
            q.fetch_all(pool).await
        }
        _ => {
            let sql = format!("{base} ORDER BY a.number ASC");
            sqlx::query_as::<_, RawAccount>(&sql)
                .bind(company_id)
                .fetch_all(pool)
                .await
        }
    }
    .map_err(kesh_db::errors::map_db_error)?;

    Ok(rows)
}

/// Lignes d'un compte sur la période, dans l'ordre de lecture d'un réviseur.
///
/// ⚠️ `fiscal_year_id` est trié **avant** `entry_number` : ce dernier repart à 1
/// à chaque exercice. `jel.id` en dernier ressort rend l'ordre **totalement
/// déterministe** — deux appels rendent la même chose, et c'est testé.
async fn fetch_lines(
    pool: &MySqlPool,
    company_id: i64,
    account_id: i64,
    period: &LedgerPeriod,
    offset: i64,
    // `None` ⇒ pas de borne (export).
    limit: Option<i64>,
) -> Result<Vec<RawLine>, ReportError> {
    let sql = "
        SELECT jel.id AS line_id, je.id AS entry_id, je.entry_date, je.fiscal_year_id,
               fy.name AS fiscal_year_name,
               je.entry_number, je.journal, je.description, jel.debit, jel.credit
        FROM journal_entry_lines jel
        INNER JOIN journal_entries je ON je.id = jel.entry_id
        INNER JOIN fiscal_years fy ON fy.id = je.fiscal_year_id
        WHERE jel.account_id = ? AND je.company_id = ?
          AND je.entry_date >= ? AND je.entry_date <= ?
        ORDER BY je.entry_date ASC, je.fiscal_year_id ASC, je.entry_number ASC,
                 jel.line_order ASC, jel.id ASC
        LIMIT ? OFFSET ?";

    sqlx::query_as::<_, RawLine>(sql)
        .bind(account_id)
        .bind(company_id)
        .bind(period.from)
        .bind(period.to)
        // MariaDB n'a pas de « LIMIT ALL » : la borne absente se dit i64::MAX.
        .bind(limit.unwrap_or(i64::MAX))
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| ReportError::Db(kesh_db::errors::map_db_error(e)))
}

/// Totaux et compte de lignes sur **toute la période**.
///
/// ⚠️ Requête distincte de [`fetch_lines`] **à dessein** : les totaux ne doivent
/// jamais être calculés sur la page rendue. `limit=5` et `limit=1000` doivent
/// donner les mêmes totaux — c'est testé.
async fn fetch_totals(
    pool: &MySqlPool,
    company_id: i64,
    account_id: i64,
    period: &LedgerPeriod,
) -> Result<(Decimal, Decimal, i64), ReportError> {
    let sql = "
        SELECT COALESCE(SUM(jel.debit), 0) AS d, COALESCE(SUM(jel.credit), 0) AS c,
               COUNT(*) AS n
        FROM journal_entry_lines jel
        INNER JOIN journal_entries je ON je.id = jel.entry_id
        WHERE jel.account_id = ? AND je.company_id = ?
          AND je.entry_date >= ? AND je.entry_date <= ?";

    sqlx::query_as::<_, (Decimal, Decimal, i64)>(sql)
        .bind(account_id)
        .bind(company_id)
        .bind(period.from)
        .bind(period.to)
        .fetch_one(pool)
        .await
        .map_err(|e| ReportError::Db(kesh_db::errors::map_db_error(e)))
}

/// Contreparties des écritures rendues, en **une seule requête** pour toutes.
///
/// ⚠️ Surtout pas une sous-requête corrélée par ligne.
async fn fetch_counterparts(
    pool: &MySqlPool,
    company_id: i64,
    account_id: i64,
    entry_ids: &[i64],
) -> Result<std::collections::HashMap<i64, Vec<String>>, ReportError> {
    use std::collections::HashMap;
    if entry_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let placeholders = std::iter::repeat_n("?", entry_ids.len())
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT jel.entry_id, a.number \
         FROM journal_entry_lines jel \
         INNER JOIN journal_entries je ON je.id = jel.entry_id \
         INNER JOIN accounts a ON a.id = jel.account_id \
         WHERE je.company_id = ? AND jel.account_id <> ? \
           AND jel.entry_id IN ({placeholders}) \
         ORDER BY jel.entry_id ASC, jel.line_order ASC"
    );

    let mut q = sqlx::query_as::<_, (i64, String)>(&sql)
        .bind(company_id)
        .bind(account_id);
    for id in entry_ids {
        q = q.bind(id);
    }
    let rows = q
        .fetch_all(pool)
        .await
        .map_err(kesh_db::errors::map_db_error)?;

    let mut map: HashMap<i64, Vec<String>> = HashMap::new();
    for (entry_id, number) in rows {
        let v = map.entry(entry_id).or_default();
        if !v.contains(&number) {
            v.push(number);
        }
    }
    Ok(map)
}

/// Options de rendu du grand livre.
#[derive(Debug, Clone, Default)]
pub struct LedgerOptions {
    /// Comptes demandés. `None` ⇒ ceux qui ont un mouvement ou un solde.
    pub account_ids: Option<Vec<i64>>,
    /// Rendre aussi les comptes sans mouvement **et** sans solde.
    pub include_zero: bool,
    pub offset: i64,
    /// Écrêté à [`MAX_LEDGER_LIMIT`]. **`None` ⇒ aucune borne** — c'est ce que
    /// prend l'export, qui doit rendre le livre entier. L'appelant qui veut le
    /// défaut d'écran passe `Some(MAX_LEDGER_LIMIT)` explicitement.
    pub limit: Option<i64>,
}

/// Génère le grand livre.
///
/// Une section par compte, triées par numéro. Chaque section porte son solde
/// d'ouverture, ses lignes avec solde progressif, ses totaux et son solde de
/// clôture — les trois lignes d'encadrement sans lesquelles ce n'est pas un
/// extrait de compte.
pub async fn generate(
    pool: &MySqlPool,
    company_id: i64,
    period: &LedgerPeriod,
    options: &LedgerOptions,
) -> Result<GeneralLedger, ReportError> {
    let explicit = options
        .account_ids
        .as_ref()
        .is_some_and(|ids| !ids.is_empty());
    let accounts = select_accounts(pool, company_id, options.account_ids.as_deref()).await?;

    let limit = options.limit.map(|l| l.clamp(1, MAX_LEDGER_LIMIT));
    let offset = options.offset.max(0);

    let mut sections = Vec::new();

    for acc in accounts {
        let opening = opening_balance(
            pool,
            company_id,
            acc.account_id,
            acc.account_type,
            period.from,
        )
        .await?;
        let (total_debit, total_credit, line_count) =
            fetch_totals(pool, company_id, acc.account_id, period).await?;

        // Un compte sans mouvement NI solde n'a rien à montrer — sauf s'il a été
        // explicitement demandé, ou si l'appelant veut tout voir.
        if !explicit && !options.include_zero && line_count == 0 && opening.is_zero() {
            continue;
        }

        let raw = fetch_lines(pool, company_id, acc.account_id, period, offset, limit).await?;
        let entry_ids: Vec<i64> = raw.iter().map(|r| r.entry_id).collect();
        let counterparts = fetch_counterparts(pool, company_id, acc.account_id, &entry_ids).await?;

        // Solde progressif + ruptures d'exercice.
        //
        // Sur un compte de RÉSULTAT, franchir une fin d'exercice remet le solde à
        // zéro : sinon le progressif affiché sur deux ans ne veut rien dire. Sur
        // un compte de BILAN, la chaîne est continue.
        let resets = !is_bilan(acc.account_type);
        let mut running = opening;
        let mut current_fy = raw.first().map(|r| r.fiscal_year_id);
        let mut breaks = Vec::new();
        let mut lines = Vec::with_capacity(raw.len());

        for r in raw {
            if resets
                && let Some(fy) = current_fy
                && r.fiscal_year_id != fy
            {
                breaks.push(FiscalYearBreak {
                    date: r.entry_date,
                    closing_fiscal_year_id: fy,
                    closing_balance: running,
                });
                running = Decimal::ZERO;
            }
            current_fy = Some(r.fiscal_year_id);

            running += signed(acc.account_type, r.debit, r.credit);

            lines.push(LedgerLine {
                line_id: r.line_id,
                entry_id: r.entry_id,
                entry_date: r.entry_date,
                fiscal_year_id: r.fiscal_year_id,
                fiscal_year_name: r.fiscal_year_name,
                entry_number: r.entry_number,
                journal: r.journal,
                description: r.description,
                counterpart: counterparts.get(&r.entry_id).cloned().unwrap_or_default(),
                debit: r.debit,
                credit: r.credit,
                running_balance: running,
            });
        }

        // ⚠️ La clôture se calcule sur les totaux de la PÉRIODE, jamais sur la
        // page rendue — c'est ce qui rend `limit` sans effet sur les chiffres.
        let closing = opening + signed(acc.account_type, total_debit, total_credit);

        sections.push(LedgerSection {
            account_id: acc.account_id,
            account_number: acc.number,
            account_name: acc.name,
            account_type: acc.account_type,
            active: acc.active,
            balance_side: if is_debit_natured(acc.account_type) {
                "debit"
            } else {
                "credit"
            },
            opening,
            lines,
            total_debit,
            total_credit,
            closing,
            unnatural_balance: closing.is_sign_negative() && !closing.is_zero(),
            fiscal_year_breaks: breaks,
            line_count,
        });
    }

    Ok(GeneralLedger {
        period: *period,
        sections,
    })
}

/// `true` pour un compte de bilan — dont le solde se reporte d'un exercice à
/// l'autre.
fn is_bilan(t: AccountType) -> bool {
    matches!(t, AccountType::Asset | AccountType::Liability)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(v: i64) -> Decimal {
        Decimal::from(v)
    }

    #[test]
    fn signe_suit_la_nature_du_compte() {
        // Convention du dépôt, reprise telle quelle de la balance et du bilan.
        assert_eq!(signed(AccountType::Asset, d(100), d(30)), d(70));
        assert_eq!(signed(AccountType::Expense, d(100), d(30)), d(70));
        assert_eq!(signed(AccountType::Liability, d(30), d(100)), d(70));
        assert_eq!(signed(AccountType::Revenue, d(30), d(100)), d(70));
    }

    #[test]
    fn seuls_les_comptes_de_bilan_reportent_leur_solde() {
        // C'est ce qui commande la borne basse du solde d'ouverture, et la
        // remise à zéro du solde progressif au passage d'exercice.
        assert!(is_bilan(AccountType::Asset));
        assert!(is_bilan(AccountType::Liability));
        assert!(!is_bilan(AccountType::Revenue));
        assert!(!is_bilan(AccountType::Expense));
    }

    #[test]
    fn periode_inversee_est_refusee() {
        let from = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        assert!(LedgerPeriod::new(from, to).is_err());
        assert!(LedgerPeriod::new(to, from).is_ok());
        // Une période d'un seul jour est valide.
        assert!(LedgerPeriod::new(from, from).is_ok());
    }

    #[test]
    fn periode_traverse_les_exercices_sans_broncher() {
        // Le propre de ce rapport : aucune contrainte d'exercice, contrairement
        // à `ReportPeriod`.
        let from = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let p = LedgerPeriod::new(from, to).expect("deux ans doivent passer");
        assert_eq!(p.from, from);
        assert_eq!(p.to, to);
    }
}
