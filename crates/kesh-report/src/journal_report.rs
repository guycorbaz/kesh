//! Journaux : écritures groupées par journal (Achats, Ventes, Banque, Caisse, OD).
//!
//! Pass 1 BH-10 : utilise `kesh_db::entities::journal_entry::Journal` (avec traits sqlx).
//! Pass 1 ECH-05 : si `journal_filter = None`, **TOUJOURS 5 sections** dans l'ordre fixe
//! (Achats, Ventes, Banque, Caisse, OD), même si certaines sont vides.

use std::collections::HashMap;

use chrono::NaiveDate;
use kesh_db::entities::journal_entry::Journal;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::MySqlPool;

use crate::errors::ReportError;
use crate::period::ReportPeriod;

/// Rapport journaux complet.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalReport {
    pub period: ReportPeriod,
    pub journals: Vec<JournalSection>,
    pub grand_total_debit: Decimal,
    pub grand_total_credit: Decimal,
}

/// Section d'un journal (Achats, Ventes, Banque, Caisse, OD).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalSection {
    pub journal: Journal,
    pub entries: Vec<JournalEntryRow>,
    pub section_total_debit: Decimal,
    pub section_total_credit: Decimal,
}

/// Une écriture comptable affichée dans le rapport journaux.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntryRow {
    pub entry_id: i64,
    pub entry_number: i64,
    pub entry_date: NaiveDate,
    pub description: String,
    pub lines: Vec<JournalEntryLineRow>,
}

/// Une ligne d'écriture affichée dans le rapport journaux.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntryLineRow {
    pub account_id: i64,
    pub account_number: String,
    pub account_name: String,
    pub debit: Decimal,
    pub credit: Decimal,
    pub line_order: i32,
}

#[derive(sqlx::FromRow)]
struct RawJoinedRow {
    entry_id: i64,
    entry_number: i64,
    entry_date: NaiveDate,
    journal: Journal,
    description: String,
    line_account_id: i64,
    line_account_number: String,
    line_account_name: String,
    line_debit: Decimal,
    line_credit: Decimal,
    line_order: i32,
}

/// Ordre fixe des journaux (Pass 1 ECH-05 — convention métier suisse).
fn fixed_journal_order() -> [Journal; 5] {
    [
        Journal::Achats,
        Journal::Ventes,
        Journal::Banque,
        Journal::Caisse,
        Journal::OD,
    ]
}

/// Génère le rapport journaux pour la période donnée.
///
/// Si `journal_filter = None` → 5 sections fixes (Achats, Ventes, Banque, Caisse, OD)
/// dans cet ordre, même si certaines sont vides.
/// Si `journal_filter = Some(j)` → 1 seule section.
pub async fn generate(
    pool: &MySqlPool,
    company_id: i64,
    period: &ReportPeriod,
    journal_filter: Option<Journal>,
) -> Result<JournalReport, ReportError> {
    let (sql, has_filter) = if journal_filter.is_some() {
        (
            "SELECT
                je.id AS entry_id,
                je.entry_number,
                je.entry_date,
                je.journal,
                je.description,
                jel.account_id AS line_account_id,
                a.number AS line_account_number,
                a.name AS line_account_name,
                jel.debit AS line_debit,
                jel.credit AS line_credit,
                jel.line_order
            FROM journal_entries je
            INNER JOIN journal_entry_lines jel ON jel.entry_id = je.id
            INNER JOIN accounts a ON a.id = jel.account_id
            WHERE je.company_id = ?
              AND je.fiscal_year_id = ?
              AND je.entry_date BETWEEN ? AND ?
              AND je.journal = ?
            ORDER BY je.journal, je.entry_date ASC, je.entry_number ASC, jel.line_order ASC",
            true,
        )
    } else {
        (
            "SELECT
                je.id AS entry_id,
                je.entry_number,
                je.entry_date,
                je.journal,
                je.description,
                jel.account_id AS line_account_id,
                a.number AS line_account_number,
                a.name AS line_account_name,
                jel.debit AS line_debit,
                jel.credit AS line_credit,
                jel.line_order
            FROM journal_entries je
            INNER JOIN journal_entry_lines jel ON jel.entry_id = je.id
            INNER JOIN accounts a ON a.id = jel.account_id
            WHERE je.company_id = ?
              AND je.fiscal_year_id = ?
              AND je.entry_date BETWEEN ? AND ?
            ORDER BY je.journal, je.entry_date ASC, je.entry_number ASC, jel.line_order ASC",
            false,
        )
    };

    let mut query = sqlx::query_as::<_, RawJoinedRow>(sql)
        .bind(company_id)
        .bind(period.fiscal_year_id)
        .bind(period.start_date)
        .bind(period.end_date);

    if has_filter {
        if let Some(j) = &journal_filter {
            query = query.bind(j.as_str());
        }
    }

    let rows = query
        .fetch_all(pool)
        .await
        .map_err(kesh_db::errors::map_db_error)?;

    // Initialiser les sections fixes
    let mut sections: HashMap<Journal, Vec<JournalEntryRow>> = HashMap::new();
    if let Some(j) = &journal_filter {
        sections.insert(*j, Vec::new());
    } else {
        for j in fixed_journal_order().iter() {
            sections.insert(*j, Vec::new());
        }
    }

    // Grouper les rows par entry_id (préservant l'ordre SQL)
    let mut current_entry: Option<(i64, Journal, JournalEntryRow)> = None;
    let mut grouped: Vec<(Journal, JournalEntryRow)> = Vec::new();

    for row in rows {
        let line = JournalEntryLineRow {
            account_id: row.line_account_id,
            account_number: row.line_account_number,
            account_name: row.line_account_name,
            debit: row.line_debit,
            credit: row.line_credit,
            line_order: row.line_order,
        };

        match &mut current_entry {
            Some((id, _, entry)) if *id == row.entry_id => {
                entry.lines.push(line);
            }
            _ => {
                if let Some((_, j, entry)) = current_entry.take() {
                    grouped.push((j, entry));
                }
                current_entry = Some((
                    row.entry_id,
                    row.journal,
                    JournalEntryRow {
                        entry_id: row.entry_id,
                        entry_number: row.entry_number,
                        entry_date: row.entry_date,
                        description: row.description,
                        lines: vec![line],
                    },
                ));
            }
        }
    }
    if let Some((_, j, entry)) = current_entry.take() {
        grouped.push((j, entry));
    }

    // Distribuer dans les sections
    for (j, entry) in grouped {
        if let Some(entries) = sections.get_mut(&j) {
            entries.push(entry);
        }
    }

    // Construire les sections dans l'ordre fixe (Pass 1 ECH-05)
    let ordered_journals: Vec<Journal> = if let Some(j) = journal_filter {
        vec![j]
    } else {
        fixed_journal_order().to_vec()
    };

    let mut journals_out: Vec<JournalSection> = Vec::with_capacity(ordered_journals.len());
    let mut grand_total_debit = Decimal::ZERO;
    let mut grand_total_credit = Decimal::ZERO;

    for j in ordered_journals {
        let entries = sections.remove(&j).unwrap_or_default();
        let section_total_debit: Decimal = entries
            .iter()
            .flat_map(|e| e.lines.iter().map(|l| l.debit))
            .sum();
        let section_total_credit: Decimal = entries
            .iter()
            .flat_map(|e| e.lines.iter().map(|l| l.credit))
            .sum();
        grand_total_debit += section_total_debit;
        grand_total_credit += section_total_credit;
        journals_out.push(JournalSection {
            journal: j,
            entries,
            section_total_debit,
            section_total_credit,
        });
    }

    Ok(JournalReport {
        period: period.clone(),
        journals: journals_out,
        grand_total_debit,
        grand_total_credit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_order_has_five_journals() {
        let order = fixed_journal_order();
        assert_eq!(order.len(), 5);
        assert_eq!(order[0], Journal::Achats);
        assert_eq!(order[1], Journal::Ventes);
        assert_eq!(order[2], Journal::Banque);
        assert_eq!(order[3], Journal::Caisse);
        assert_eq!(order[4], Journal::OD);
    }
}
