//! Rapports analytiques par projet (Epic 19, Story 19-6a).
//!
//! Fondation partagée (scope racine + sous-projets, période 2 modes) réutilisée
//! par le rapport « Dépenses par projet » (ici) et « Rendement par projet »
//! (Story 19-6b). Lit la dimension `journal_entry_lines.project_id` posée par
//! 19-1 et alimentée par 19-2..19-5.
//!
//! Classification par `accounts.account_type` (DC1). Rollup 2 niveaux en Rust
//! (DC3). Deux modes de période : exercice unique OU cumulé multi-exercices
//! borné par les dates du projet (DC4). Aucun `Date::now()` en lib : le `today`
//! du mode cumulé est passé par le handler.

use chrono::NaiveDate;
use kesh_db::entities::project::Project;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::MySqlPool;

use crate::errors::ReportError;
use crate::period::ReportPeriod;

/// Identité minimale d'un projet exposée dans les rapports.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    pub id: i64,
    pub code: String,
    pub name: String,
}

impl From<&Project> for ProjectInfo {
    fn from(p: &Project) -> Self {
        Self {
            id: p.id,
            code: p.code.clone(),
            name: p.name.clone(),
        }
    }
}

/// Périmètre d'un rapport projet : le projet ciblé + (s'il est racine) ses
/// sous-projets. `project_ids` = tous les ids agrégés (au moins le ciblé).
#[derive(Debug, Clone)]
pub struct ProjectReportScope {
    pub root: Project,
    /// Sous-projets (vide si `root` est lui-même un sous-projet — DC3).
    pub subprojects: Vec<Project>,
    /// Dates du projet racine, pour borner le mode cumulé.
    pub project_ids: Vec<i64>,
}

impl ProjectReportScope {
    /// Ordre stable des sections : racine d'abord, puis sous-projets par code.
    pub fn ordered_projects(&self) -> Vec<&Project> {
        let mut v: Vec<&Project> = Vec::with_capacity(1 + self.subprojects.len());
        v.push(&self.root);
        let mut children: Vec<&Project> = self.subprojects.iter().collect();
        children.sort_by(|a, b| a.code.cmp(&b.code));
        v.extend(children);
        v
    }
}

/// Résout le périmètre d'un rapport pour un projet ciblé.
///
/// - Projet inconnu / cross-company → [`ReportError::ProjectNotFound`] (404).
/// - Projet **racine** (`parent_id IS NULL`) → scope = racine + sous-projets
///   (archivés inclus, l'historique reste lisible, DC3).
/// - **Sous-projet** → scope = lui seul (pas de remontée au parent, DC3).
pub async fn resolve_scope(
    pool: &MySqlPool,
    company_id: i64,
    project_id: i64,
) -> Result<ProjectReportScope, ReportError> {
    let root = kesh_db::repositories::projects::get_for_company(pool, company_id, project_id)
        .await?
        .ok_or(ReportError::ProjectNotFound { project_id })?;

    let subprojects: Vec<Project> = if root.parent_id.is_none() {
        // Racine : agréger ses enfants (archivés inclus).
        kesh_db::repositories::projects::list_by_company(pool, company_id, true)
            .await?
            .into_iter()
            .filter(|p| p.parent_id == Some(root.id))
            .collect()
    } else {
        Vec::new()
    };

    let mut project_ids = vec![root.id];
    project_ids.extend(subprojects.iter().map(|p| p.id));

    Ok(ProjectReportScope {
        root,
        subprojects,
        project_ids,
    })
}

/// Mode de période d'un rapport projet (DC4).
#[derive(Debug, Clone)]
pub enum ProjectPeriodMode {
    /// Exercice unique : filtre `je.fiscal_year_id = ? AND je.entry_date BETWEEN ? AND ?`.
    FiscalYear { period: ReportPeriod },
    /// Cumulé depuis l'origine : pas de filtre exercice, borné par les dates du
    /// projet. `start = None` → aucune borne basse ; `end` = `project.end_date`
    /// ou `today` (passé par le handler).
    Cumulative {
        start: Option<NaiveDate>,
        end: NaiveDate,
    },
}

impl ProjectPeriodMode {
    /// Libellé humain de la période (affiché dans le rapport + PDF/CSV).
    pub fn period_label(&self) -> String {
        match self {
            ProjectPeriodMode::FiscalYear { period } => {
                format!("Exercice {} — {}", period.start_date, period.end_date)
            }
            ProjectPeriodMode::Cumulative { start, end } => match start {
                Some(s) => format!("Cumulé du {s} au {end}"),
                None => format!("Cumulé jusqu'au {end}"),
            },
        }
    }

    /// Slug court du mode (`fiscal_year` / `cumulative`) exposé en JSON.
    pub fn as_str(&self) -> &'static str {
        match self {
            ProjectPeriodMode::FiscalYear { .. } => "fiscal_year",
            ProjectPeriodMode::Cumulative { .. } => "cumulative",
        }
    }

    /// Construit le fragment SQL de filtre `je` + les binds ordonnés associés.
    /// Retourne `(fragment, binds)` où chaque bind est un [`PeriodBind`] à
    /// appliquer dans l'ordre après les binds projet.
    ///
    /// - `FiscalYear` → `"AND je.fiscal_year_id = ? AND je.entry_date BETWEEN ? AND ?"`
    /// - `Cumulative` avec start → `"AND je.entry_date BETWEEN ? AND ?"`
    /// - `Cumulative` sans start → `"AND je.entry_date <= ?"`
    fn je_filter(&self) -> (String, Vec<PeriodBind>) {
        match self {
            ProjectPeriodMode::FiscalYear { period } => (
                "AND je.fiscal_year_id = ? AND je.entry_date BETWEEN ? AND ?".to_string(),
                vec![
                    PeriodBind::Id(period.fiscal_year_id),
                    PeriodBind::Date(period.start_date),
                    PeriodBind::Date(period.end_date),
                ],
            ),
            ProjectPeriodMode::Cumulative { start, end } => match start {
                Some(s) => (
                    "AND je.entry_date BETWEEN ? AND ?".to_string(),
                    vec![PeriodBind::Date(*s), PeriodBind::Date(*end)],
                ),
                None => (
                    "AND je.entry_date <= ?".to_string(),
                    vec![PeriodBind::Date(*end)],
                ),
            },
        }
    }
}

/// Bind hétérogène du filtre de période (id d'exercice ou date).
enum PeriodBind {
    Id(i64),
    Date(NaiveDate),
}

/// Placeholders `?,?,…` pour un `IN (...)` de `n` éléments (n ≥ 1 garanti par le scope).
fn in_placeholders(n: usize) -> String {
    std::iter::repeat_n("?", n).collect::<Vec<_>>().join(",")
}

// ===========================================================================
// Rapport « Dépenses par projet » (Story 19-6a)
// ===========================================================================

/// Référence à une écriture contributrice (drill-down jusqu'à l'écriture, DC).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectEntryRef {
    pub entry_id: i64,
    pub entry_number: i64,
    pub entry_date: NaiveDate,
    pub description: String,
    pub amount: Decimal,
}

/// Ligne « compte » d'une section : montant agrégé + écritures contributrices.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseAccountRow {
    pub account_id: i64,
    pub account_number: String,
    pub account_name: String,
    pub amount: Decimal,
    pub entries: Vec<ProjectEntryRef>,
}

/// Section = un projet du scope (racine ou sous-projet) et ses comptes de charge.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectExpenseSection {
    pub project: ProjectInfo,
    pub is_root: bool,
    pub rows: Vec<ExpenseAccountRow>,
    pub subtotal: Decimal,
}

/// Rapport « Dépenses par projet ».
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectExpensesReport {
    pub report_type: String,
    pub project: ProjectInfo,
    pub mode: String,
    pub period_label: String,
    pub sections: Vec<ProjectExpenseSection>,
    pub grand_total: Decimal,
}

/// Ligne brute d'agrégation (project_id + compte + montant).
#[derive(sqlx::FromRow)]
struct ExpenseAggRow {
    project_id: i64,
    account_id: i64,
    #[sqlx(rename = "number")]
    account_number: String,
    #[sqlx(rename = "name")]
    account_name: String,
    amount: Decimal,
}

/// Ligne brute de drill-down (project_id + compte + écriture).
#[derive(sqlx::FromRow)]
struct ExpenseEntryRow {
    project_id: i64,
    account_id: i64,
    entry_id: i64,
    entry_number: i64,
    entry_date: NaiveDate,
    description: String,
    amount: Decimal,
}

/// Génère le rapport « Dépenses par projet » : lignes de comptes `Expense`
/// taguées sur le scope, groupées par (sous-projet, compte), signe `debit − credit`.
pub async fn generate_project_expenses(
    pool: &MySqlPool,
    company_id: i64,
    scope: &ProjectReportScope,
    mode: &ProjectPeriodMode,
) -> Result<ProjectExpensesReport, ReportError> {
    let (je_filter, period_binds) = mode.je_filter();
    let ids_ph = in_placeholders(scope.project_ids.len());

    // Requête 1 — agrégat par (project_id, compte).
    let agg_sql = format!(
        "SELECT jel.project_id AS project_id, a.id AS account_id, a.number, a.name, \
                COALESCE(SUM(jel.debit), 0) - COALESCE(SUM(jel.credit), 0) AS amount \
         FROM accounts a \
         INNER JOIN journal_entry_lines jel ON jel.account_id = a.id \
         INNER JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE a.company_id = ? \
           AND a.account_type = 'Expense' \
           AND je.company_id = ? \
           AND jel.project_id IN ({ids_ph}) \
           {je_filter} \
         GROUP BY jel.project_id, a.id, a.number, a.name \
         HAVING amount != 0 \
         ORDER BY a.number ASC"
    );
    let mut agg_q = sqlx::query_as::<_, ExpenseAggRow>(&agg_sql)
        .bind(company_id)
        .bind(company_id);
    for pid in &scope.project_ids {
        agg_q = agg_q.bind(pid);
    }
    agg_q = bind_period(agg_q, &period_binds);
    let agg_rows = agg_q
        .fetch_all(pool)
        .await
        .map_err(kesh_db::errors::map_db_error)?;

    // Requête 2 — détail écritures (drill-down).
    let detail_sql = format!(
        "SELECT jel.project_id AS project_id, jel.account_id AS account_id, \
                je.id AS entry_id, je.entry_number, je.entry_date, je.description, \
                COALESCE(SUM(jel.debit), 0) - COALESCE(SUM(jel.credit), 0) AS amount \
         FROM journal_entry_lines jel \
         INNER JOIN accounts a ON a.id = jel.account_id \
         INNER JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE a.company_id = ? \
           AND a.account_type = 'Expense' \
           AND je.company_id = ? \
           AND jel.project_id IN ({ids_ph}) \
           {je_filter} \
         GROUP BY jel.project_id, jel.account_id, je.id, je.entry_number, je.entry_date, je.description \
         HAVING amount != 0 \
         ORDER BY je.entry_date ASC, je.entry_number ASC"
    );
    let mut det_q = sqlx::query_as::<_, ExpenseEntryRow>(&detail_sql)
        .bind(company_id)
        .bind(company_id);
    for pid in &scope.project_ids {
        det_q = det_q.bind(pid);
    }
    det_q = bind_period(det_q, &period_binds);
    let detail_rows = det_q
        .fetch_all(pool)
        .await
        .map_err(kesh_db::errors::map_db_error)?;

    // Assemblage en Rust : sections ordonnées (racine puis sous-projets par code).
    let mut sections: Vec<ProjectExpenseSection> = Vec::new();
    let mut grand_total = Decimal::ZERO;

    for proj in scope.ordered_projects() {
        let mut rows: Vec<ExpenseAccountRow> = Vec::new();
        let mut subtotal = Decimal::ZERO;
        for agg in agg_rows.iter().filter(|r| r.project_id == proj.id) {
            let entries: Vec<ProjectEntryRef> = detail_rows
                .iter()
                .filter(|d| d.project_id == proj.id && d.account_id == agg.account_id)
                .map(|d| ProjectEntryRef {
                    entry_id: d.entry_id,
                    entry_number: d.entry_number,
                    entry_date: d.entry_date,
                    description: d.description.clone(),
                    amount: d.amount,
                })
                .collect();
            subtotal += agg.amount;
            rows.push(ExpenseAccountRow {
                account_id: agg.account_id,
                account_number: agg.account_number.clone(),
                account_name: agg.account_name.clone(),
                amount: agg.amount,
                entries,
            });
        }
        if rows.is_empty() {
            continue; // section sans dépense → omise
        }
        grand_total += subtotal;
        sections.push(ProjectExpenseSection {
            project: ProjectInfo::from(proj),
            is_root: proj.id == scope.root.id,
            rows,
            subtotal,
        });
    }

    Ok(ProjectExpensesReport {
        report_type: "project-expenses".to_string(),
        project: ProjectInfo::from(&scope.root),
        mode: mode.as_str().to_string(),
        period_label: mode.period_label(),
        sections,
        grand_total,
    })
}

/// Applique les binds de période (id d'exercice ou dates) dans l'ordre.
fn bind_period<'q, O>(
    mut q: sqlx::query::QueryAs<'q, sqlx::MySql, O, sqlx::mysql::MySqlArguments>,
    binds: &[PeriodBind],
) -> sqlx::query::QueryAs<'q, sqlx::MySql, O, sqlx::mysql::MySqlArguments> {
    for b in binds {
        q = match b {
            PeriodBind::Id(id) => q.bind(*id),
            PeriodBind::Date(d) => q.bind(*d),
        };
    }
    q
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_placeholders_builds_correct_count() {
        assert_eq!(in_placeholders(1), "?");
        assert_eq!(in_placeholders(3), "?,?,?");
    }

    #[test]
    fn period_label_variants() {
        let cum = ProjectPeriodMode::Cumulative {
            start: None,
            end: NaiveDate::from_ymd_opt(2026, 7, 4).unwrap(),
        };
        assert!(cum.period_label().contains("Cumulé jusqu'au"));
        assert_eq!(cum.as_str(), "cumulative");

        let cum2 = ProjectPeriodMode::Cumulative {
            start: Some(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
            end: NaiveDate::from_ymd_opt(2026, 7, 4).unwrap(),
        };
        assert!(cum2.period_label().contains("Cumulé du"));
    }

    #[test]
    fn je_filter_shapes() {
        let cum_no_start = ProjectPeriodMode::Cumulative {
            start: None,
            end: NaiveDate::from_ymd_opt(2026, 7, 4).unwrap(),
        };
        let (frag, binds) = cum_no_start.je_filter();
        assert_eq!(frag, "AND je.entry_date <= ?");
        assert_eq!(binds.len(), 1);

        let cum_start = ProjectPeriodMode::Cumulative {
            start: Some(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
            end: NaiveDate::from_ymd_opt(2026, 7, 4).unwrap(),
        };
        let (frag, binds) = cum_start.je_filter();
        assert_eq!(frag, "AND je.entry_date BETWEEN ? AND ?");
        assert_eq!(binds.len(), 2);
    }
}
