//! Sérialiseur CSV des 4 rapports comptables — Story 9-2a.
//!
//! Format RFC 4180 + extensions Excel suisse :
//! - UTF-8 + BOM (`\u{FEFF}` en tête) — pour Excel CH/DE qui détecte l'encodage via BOM.
//! - Séparateur `;` (PAS la virgule — convention Excel CH/DE).
//! - Terminator `\r\n` (CRLF).
//! - Format décimal ISO point + **toujours 2 décimales** (`format!("{:.2}", ...)`).
//! - Dates ISO 8601 `YYYY-MM-DD` (machine-readable).
//! - Escaping automatique (RFC 4180) géré par `csv 1.3`.
//!
//! Cas rapport vide (T3.5 + Pass 1 ECH-M1) : **écrit uniquement la ligne d'en-tête**
//! sans data rows. Le frontend détecte le cas vide via le nombre de lignes du blob.

use std::io::Write;

use rust_decimal::Decimal;

use crate::aged_receivables::AgedReceivables;
use crate::balance_sheet::BalanceSheet;
use crate::errors::ReportError;
use crate::general_ledger::GeneralLedger;
use crate::income_statement::IncomeStatement;
use crate::journal_report::JournalReport;
use crate::trial_balance::TrialBalance;
use crate::vat_report::VatReport;

/// Octets UTF-8 du BOM (`\u{FEFF}`).
const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

/// Helper interne — construit un `csv::Writer` configuré (séparateur `;`, terminator CRLF).
fn make_writer<W: Write>(writer: W) -> csv::Writer<W> {
    csv::WriterBuilder::new()
        .delimiter(b';')
        .terminator(csv::Terminator::CRLF)
        .from_writer(writer)
}

/// Formate un Decimal au format ISO décimal point, **toujours 2 décimales** (T3.6 +
/// Pass 1 ECH-H5 — sinon Excel auto-type colonne mixte casse).
fn format_amount_iso(amount: Decimal) -> String {
    format!("{amount:.2}")
}

/// Helper : écrit le BOM en tête puis renvoie le writer prêt à recevoir des records.
///
/// Pass 1 code-review H1 : mappe vers `CsvGeneration` (pas `PdfGeneration` —
/// sinon un échec d'écriture du BOM lors d'un export CSV apparaissait à tort
/// comme un « Échec génération PDF » côté UI).
fn write_bom<W: Write>(writer: &mut W) -> Result<(), ReportError> {
    writer
        .write_all(UTF8_BOM)
        .map_err(|e| ReportError::CsvGeneration(format!("csv BOM write: {e}")))
}

/// Génère le CSV du bilan (AC #12).
///
/// Colonnes : `Section;NumeroCompte;NomCompte;Solde` où
/// `Section ∈ {Actifs, Passifs, CapitauxPropres}`. La section `CapitauxPropres`
/// (Story 14-3c) liste d'abord les **comptes physiques de fonds propres**, dans
/// l'ordre par rôle fixé en backend (**sans en-têtes de groupe** — le format linéaire
/// une-ligne-par-compte porte le regroupement par l'ordre), puis **deux lignes
/// calculées** distinctes (Story 14-1) : « Résultat reporté (calculé) »
/// (`retained_earnings`) + « Résultat de l'exercice » (`equity_result`), enfin
/// `Total capitaux propres`. Les comptes de rôle equity **ne figurent plus** dans
/// `Passifs` (partition D2) — `Total passifs` = dettes réelles seules. Libellés
/// **hardcodés FR-CH** (i18n sérialiseurs déférée, limitation L1/L4/L11 — D3).
pub fn render_balance_sheet_csv<W: Write>(
    bs: &BalanceSheet,
    mut writer: W,
) -> Result<(), ReportError> {
    write_bom(&mut writer)?;
    let mut wtr = make_writer(writer);

    wtr.write_record(["Section", "NumeroCompte", "NomCompte", "Solde"])
        .map_err(map_csv_err)?;

    // Cas rapport vide : header seul, pas de data rows (Pass 1 ECH-M1). Garde
    // centralisée `BalanceSheet::is_empty()` (Story 14-3c P1-F1) — inclut `equity` :
    // un reclassement pur entre comptes de fonds propres (section equity non vide,
    // actifs/passifs/virtuels nuls) n'est PAS vide, sinon on masquerait la section
    // Capitaux propres réellement peuplée.
    if bs.is_empty() {
        wtr.flush().map_err(map_io_err)?;
        return Ok(());
    }

    // Pass 1 code-review M6 (ECH1-M1) : guard chaque section indépendamment
    // — sinon un BS partiel (actifs vides mais passifs présents, ou inverse)
    // affiche un « Total actifs;;;0.00 » phantom alors qu'aucune ligne actif
    // n'a été émise. Idem section Passifs.
    if !bs.assets.is_empty() {
        for ab in &bs.assets {
            wtr.write_record([
                "Actifs",
                &ab.account_number,
                &ab.account_name,
                &format_amount_iso(ab.balance),
            ])
            .map_err(map_csv_err)?;
        }
        wtr.write_record(["Total actifs", "", "", &format_amount_iso(bs.total_assets)])
            .map_err(map_csv_err)?;
    }

    if !bs.liabilities.is_empty() {
        for ab in &bs.liabilities {
            wtr.write_record([
                "Passifs",
                &ab.account_number,
                &ab.account_name,
                &format_amount_iso(ab.balance),
            ])
            .map_err(map_csv_err)?;
        }
        wtr.write_record([
            "Total passifs",
            "",
            "",
            &format_amount_iso(bs.total_liabilities),
        ])
        .map_err(map_csv_err)?;
    }

    // Section Capitaux propres — présentation par rôle (Story 14-3c). D'abord les
    // comptes physiques de fonds propres, dans l'ordre par rôle fixé en backend
    // (une ligne par compte, l'ordre porte le regroupement — pas d'en-tête de groupe
    // dans le format linéaire CSV, AC-C). Puis les deux lignes **calculées** distinctes
    // (« Résultat reporté (calculé) » marquée calculée pour la distinguer d'un compte
    // physique de rôle RetainedEarnings — collision D1). Le report à-nouveau DOIT figurer
    // sinon le bilan exporté est déséquilibré.
    for ab in &bs.equity {
        wtr.write_record([
            "CapitauxPropres",
            &ab.account_number,
            &ab.account_name,
            &format_amount_iso(ab.balance),
        ])
        .map_err(map_csv_err)?;
    }
    wtr.write_record([
        "CapitauxPropres",
        "",
        "Résultat reporté (calculé)",
        &format_amount_iso(bs.retained_earnings),
    ])
    .map_err(map_csv_err)?;
    wtr.write_record([
        "CapitauxPropres",
        "",
        "Résultat de l'exercice",
        &format_amount_iso(bs.equity_result),
    ])
    .map_err(map_csv_err)?;

    // Total capitaux propres = comptes physiques + 2 lignes calculées.
    let total_equity_all = bs.total_equity + bs.retained_earnings + bs.equity_result;
    wtr.write_record([
        "Total capitaux propres",
        "",
        "",
        &format_amount_iso(total_equity_all),
    ])
    .map_err(map_csv_err)?;

    // Ligne finale : somme passifs (dettes) + capitaux propres — invariant restructuré
    // (14-3c) : `total_liabilities + total_equity + retained_earnings + equity_result`
    // = ancien total passifs + fonds propres.
    let total_liab_eq = bs.total_liabilities + total_equity_all;
    wtr.write_record([
        "Total passifs + capitaux propres",
        "",
        "",
        &format_amount_iso(total_liab_eq),
    ])
    .map_err(map_csv_err)?;

    wtr.flush().map_err(map_io_err)?;
    Ok(())
}

/// Génère le CSV du compte de résultat (AC #13).
///
/// Colonnes : `Section;NumeroCompte;NomCompte;Solde` où
/// `Section ∈ {Produits, Charges}`. Ligne finale `ResultatNet;;;<somme>`.
pub fn render_income_statement_csv<W: Write>(
    is_: &IncomeStatement,
    mut writer: W,
) -> Result<(), ReportError> {
    write_bom(&mut writer)?;
    let mut wtr = make_writer(writer);

    wtr.write_record(["Section", "NumeroCompte", "NomCompte", "Solde"])
        .map_err(map_csv_err)?;

    if is_.revenues.is_empty() && is_.expenses.is_empty() {
        wtr.flush().map_err(map_io_err)?;
        return Ok(());
    }

    // Pass 1 code-review M6 (ECH1-M1) : guard chaque section indépendamment
    // pour éviter un « Total produits;;;0.00 » / « Total charges;;;0.00 »
    // phantom quand une seule des deux sections est non-vide.
    if !is_.revenues.is_empty() {
        for ab in &is_.revenues {
            wtr.write_record([
                "Produits",
                &ab.account_number,
                &ab.account_name,
                &format_amount_iso(ab.balance),
            ])
            .map_err(map_csv_err)?;
        }
        wtr.write_record([
            "Total produits",
            "",
            "",
            &format_amount_iso(is_.total_revenues),
        ])
        .map_err(map_csv_err)?;
    }

    if !is_.expenses.is_empty() {
        for ab in &is_.expenses {
            wtr.write_record([
                "Charges",
                &ab.account_number,
                &ab.account_name,
                &format_amount_iso(ab.balance),
            ])
            .map_err(map_csv_err)?;
        }
        wtr.write_record([
            "Total charges",
            "",
            "",
            &format_amount_iso(is_.total_expenses),
        ])
        .map_err(map_csv_err)?;
    }

    // Résultat net : utile même si une section est vide (= total_revenues -
    // total_expenses, donc reflète le rapport réel).
    wtr.write_record(["ResultatNet", "", "", &format_amount_iso(is_.net_result)])
        .map_err(map_csv_err)?;

    wtr.flush().map_err(map_io_err)?;
    Ok(())
}

/// Génère le CSV de la balance des comptes (AC #14).
///
/// Colonnes : `NumeroCompte;NomCompte;TotalDebit;TotalCredit;Solde`. Ligne
/// finale totaux débit/crédit avec colonnes solde vide.
pub fn render_trial_balance_csv<W: Write>(
    tb: &TrialBalance,
    mut writer: W,
) -> Result<(), ReportError> {
    write_bom(&mut writer)?;
    let mut wtr = make_writer(writer);

    wtr.write_record([
        "NumeroCompte",
        "NomCompte",
        "TotalDebit",
        "TotalCredit",
        "Solde",
    ])
    .map_err(map_csv_err)?;

    if tb.rows.is_empty() {
        wtr.flush().map_err(map_io_err)?;
        return Ok(());
    }

    for row in &tb.rows {
        wtr.write_record([
            &row.account_number,
            &row.account_name,
            &format_amount_iso(row.total_debit),
            &format_amount_iso(row.total_credit),
            &format_amount_iso(row.balance),
        ])
        .map_err(map_csv_err)?;
    }

    wtr.write_record([
        "Total",
        "",
        &format_amount_iso(tb.total_debit),
        &format_amount_iso(tb.total_credit),
        "",
    ])
    .map_err(map_csv_err)?;

    wtr.flush().map_err(map_io_err)?;
    Ok(())
}

/// Génère le CSV des journaux (AC #15, #16).
///
/// Colonnes : `Journal;DateEcriture;NumeroEcriture;Description;NumeroCompte;NomCompte;Debit;Credit`.
/// Une ligne par `journal_entry_line`. Ordre déterminé par
/// `Journal ASC, entry_date ASC, journal_entry_id ASC, line_order ASC`
/// (déjà appliqué côté SQL Story 9-1).
pub fn render_journal_report_csv<W: Write>(
    jr: &JournalReport,
    mut writer: W,
) -> Result<(), ReportError> {
    write_bom(&mut writer)?;
    let mut wtr = make_writer(writer);

    wtr.write_record([
        "Journal",
        "DateEcriture",
        "NumeroEcriture",
        "Description",
        "NumeroCompte",
        "NomCompte",
        "Debit",
        "Credit",
    ])
    .map_err(map_csv_err)?;

    let is_empty = jr.journals.iter().all(|s| s.entries.is_empty());
    if is_empty {
        wtr.flush().map_err(map_io_err)?;
        return Ok(());
    }

    for section in &jr.journals {
        for entry in &section.entries {
            for line in &entry.lines {
                wtr.write_record([
                    section.journal.as_str(),
                    &entry.entry_date.format("%Y-%m-%d").to_string(),
                    &entry.entry_number.to_string(),
                    &entry.description,
                    &line.account_number,
                    &line.account_name,
                    &format_amount_iso(line.debit),
                    &format_amount_iso(line.credit),
                ])
                .map_err(map_csv_err)?;
            }
        }
    }

    wtr.flush().map_err(map_io_err)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Mappers d'erreur internes
// ---------------------------------------------------------------------------

/// Génère le CSV du rapport TVA (Story 11-2).
///
/// Colonnes : `Taux;ChiffreAffairesHT;TVADue` (une ligne par taux), puis lignes
/// récapitulatives (label en 1re colonne, montant dans la colonne naturelle) :
/// total CA HT, total TVA due, TVA récupérable (solde du compte impôt préalable,
/// Story 18-1d), solde. Rapport vide = en-tête seul, **sauf** si la TVA récupérable
/// est non nulle (cas « achats seuls » : on rend le récapitulatif).
pub fn render_vat_report_csv<W: Write>(
    report: &VatReport,
    mut writer: W,
) -> Result<(), ReportError> {
    write_bom(&mut writer)?;
    let mut wtr = make_writer(writer);

    wtr.write_record(["Taux", "ChiffreAffairesHT", "TVADue"])
        .map_err(map_csv_err)?;

    // Cas rapport vide : header seul (Pass 1 ECH-M1, pattern par renderer).
    // Story 18-1d : un rapport sans vente (rows vide) mais avec de la TVA
    // récupérable (achats seuls) n'est PAS vide — il faut écrire le récapitulatif.
    // On ne court-circuite que si AUSSI récupérable == 0.
    if report.rows.is_empty() && report.total_vat_recoverable == Decimal::ZERO {
        wtr.flush().map_err(map_io_err)?;
        return Ok(());
    }

    for row in &report.rows {
        wtr.write_record([
            &format_amount_iso(row.rate),
            &format_amount_iso(row.base_ht),
            &format_amount_iso(row.vat_due),
        ])
        .map_err(map_csv_err)?;
    }

    // Récapitulatif (montant dans sa colonne naturelle).
    wtr.write_record([
        "Total chiffre d'affaires HT",
        &format_amount_iso(report.total_base_ht),
        "",
    ])
    .map_err(map_csv_err)?;
    wtr.write_record([
        "Total TVA due",
        "",
        &format_amount_iso(report.total_vat_due),
    ])
    .map_err(map_csv_err)?;
    wtr.write_record([
        "TVA récupérable",
        "",
        &format_amount_iso(report.total_vat_recoverable),
    ])
    .map_err(map_csv_err)?;
    wtr.write_record(["Solde", "", &format_amount_iso(report.vat_balance)])
        .map_err(map_csv_err)?;
    // Story 18-1e : écart de réconciliation (TVA due dérivée − solde compte TVA due
    // au grand livre, périmètre ventes). Libellé en dur FR (i18n exports déférée v0.2).
    wtr.write_record([
        "Écart de réconciliation",
        "",
        &format_amount_iso(report.reconciliation_delta),
    ])
    .map_err(map_csv_err)?;

    wtr.flush().map_err(map_io_err)?;
    Ok(())
}

/// Génère le CSV de la balance âgée des créances clients (Story 21-7, AC 6).
///
/// Colonnes : `Contact;Non échu;1-30;31-60;61-90;90+;Total`. Une ligne par
/// contact, puis une ligne « Total général » (les `totals`). En-têtes **français
/// en dur** (patron `render_vat_report_csv` — les `render_*_csv` n'ont pas de
/// paramètre locale, i18n exports déférée v0.2). Rapport vide → en-tête seul.
pub fn render_aged_receivables_csv<W: Write>(
    report: &AgedReceivables,
    mut writer: W,
) -> Result<(), ReportError> {
    write_bom(&mut writer)?;
    let mut wtr = make_writer(writer);

    wtr.write_record([
        "Contact",
        "Non échu",
        "1-30",
        "31-60",
        "61-90",
        "90+",
        "Total",
    ])
    .map_err(map_csv_err)?;

    // Cas rapport vide : header seul, pas de data rows (pattern par renderer).
    if report.rows.is_empty() {
        wtr.flush().map_err(map_io_err)?;
        return Ok(());
    }

    for row in &report.rows {
        wtr.write_record([
            &row.contact_name,
            &format_amount_iso(row.buckets.not_due),
            &format_amount_iso(row.buckets.days_1_to_30),
            &format_amount_iso(row.buckets.days_31_to_60),
            &format_amount_iso(row.buckets.days_61_to_90),
            &format_amount_iso(row.buckets.days_over_90),
            &format_amount_iso(row.buckets.total),
        ])
        .map_err(map_csv_err)?;
    }

    // Ligne « Total général » (les totaux sommés en Rust).
    wtr.write_record([
        "Total général",
        &format_amount_iso(report.totals.not_due),
        &format_amount_iso(report.totals.days_1_to_30),
        &format_amount_iso(report.totals.days_31_to_60),
        &format_amount_iso(report.totals.days_61_to_90),
        &format_amount_iso(report.totals.days_over_90),
        &format_amount_iso(report.totals.total),
    ])
    .map_err(map_csv_err)?;

    wtr.flush().map_err(map_io_err)?;
    Ok(())
}

/// Génère le CSV du rapport « Dépenses par projet » (Story 19-6a).
///
/// Colonnes : `Projet;SousProjet;NumeroCompte;NomCompte;Montant`. Une ligne par
/// (section, compte), un sous-total par section, un total général. Le drill-down
/// (écritures) n'est PAS exporté en CSV (réservé à l'affichage/JSON).
pub fn render_project_expenses_csv<W: Write>(
    report: &crate::project_report::ProjectExpensesReport,
    mut writer: W,
) -> Result<(), ReportError> {
    write_bom(&mut writer)?;
    let mut wtr = make_writer(writer);

    wtr.write_record([
        "Projet",
        "SousProjet",
        "NumeroCompte",
        "NomCompte",
        "Montant",
    ])
    .map_err(map_csv_err)?;

    let root_label = format!("{} — {}", report.project.code, report.project.name);

    for section in &report.sections {
        let sub_label = if section.is_root {
            String::new()
        } else {
            format!("{} — {}", section.project.code, section.project.name)
        };
        for row in &section.rows {
            wtr.write_record([
                &root_label,
                &sub_label,
                &row.account_number,
                &row.account_name,
                &format_amount_iso(row.amount),
            ])
            .map_err(map_csv_err)?;
        }
        wtr.write_record([
            &root_label,
            &sub_label,
            "",
            "Sous-total",
            &format_amount_iso(section.subtotal),
        ])
        .map_err(map_csv_err)?;
    }

    wtr.write_record([
        &root_label,
        "",
        "",
        "Total dépenses",
        &format_amount_iso(report.grand_total),
    ])
    .map_err(map_csv_err)?;

    wtr.flush().map_err(map_io_err)?;
    Ok(())
}

/// Formate un rendement `Option<Decimal>` en `"xx.xx%"` ou `"—"` (Story 19-6b).
fn format_rendement(pct: Option<Decimal>) -> String {
    match pct {
        Some(p) => format!("{p:.2}%"),
        None => "—".to_string(),
    }
}

/// Génère le CSV du rapport « Rendement par projet » (Story 19-6b).
///
/// Colonnes : `Projet;SousProjet;CoutInvesti;Revenus;ResultatNet;RendementPct`.
/// Une ligne par section + une ligne total.
pub fn render_project_return_csv<W: Write>(
    report: &crate::project_report::ProjectReturnReport,
    mut writer: W,
) -> Result<(), ReportError> {
    write_bom(&mut writer)?;
    let mut wtr = make_writer(writer);

    wtr.write_record([
        "Projet",
        "SousProjet",
        "CoutInvesti",
        "Revenus",
        "ResultatNet",
        "RendementPct",
    ])
    .map_err(map_csv_err)?;

    let root_label = format!("{} — {}", report.project.code, report.project.name);
    for section in &report.sections {
        let sub_label = if section.is_root {
            String::new()
        } else {
            format!("{} — {}", section.project.code, section.project.name)
        };
        wtr.write_record([
            &root_label,
            &sub_label,
            &format_amount_iso(section.cout_investi),
            &format_amount_iso(section.revenus),
            &format_amount_iso(section.resultat_net),
            &format_rendement(section.rendement_pct),
        ])
        .map_err(map_csv_err)?;
    }
    wtr.write_record([
        &root_label,
        "Total",
        &format_amount_iso(report.totals.cout_investi),
        &format_amount_iso(report.totals.revenus),
        &format_amount_iso(report.totals.resultat_net),
        &format_rendement(report.totals.rendement_pct),
    ])
    .map_err(map_csv_err)?;

    wtr.flush().map_err(map_io_err)?;
    Ok(())
}

/// Pass 1 code-review H1 : mappent vers `CsvGeneration` (pas `PdfGeneration`).
/// Cf. commentaire `write_bom` ci-dessus pour le contexte.
fn map_csv_err(e: csv::Error) -> ReportError {
    ReportError::CsvGeneration(format!("csv write: {e}"))
}

fn map_io_err(e: std::io::Error) -> ReportError {
    ReportError::CsvGeneration(format!("csv io flush: {e}"))
}

/// Rend le grand livre en CSV.
///
/// ⚠️ **L'ouverture et la clôture sont des LIGNES, pas des métadonnées hors
/// tableau** : un CSV doit rester réconciliable dans un tableur, où l'on
/// additionne une colonne. Une ouverture reléguée en en-tête obligerait à la
/// ressaisir à la main.
///
/// Une section par compte, séparée par une ligne vide — le compte est répété sur
/// chaque ligne pour que le fichier reste filtrable et triable.
pub fn render_general_ledger_csv<W: Write>(
    ledger: &GeneralLedger,
    mut writer: W,
) -> Result<(), ReportError> {
    write_bom(&mut writer)?;
    let mut wtr = make_writer(writer);

    wtr.write_record([
        "NumeroCompte",
        "NomCompte",
        "Date",
        "Piece",
        "Exercice",
        "Journal",
        "Libelle",
        "Contrepartie",
        "Debit",
        "Credit",
        "SoldeProgressif",
    ])
    .map_err(map_csv_err)?;

    for section in &ledger.sections {
        // Ligne d'ouverture — le report d'entrée de période.
        wtr.write_record([
            &section.account_number,
            &section.account_name,
            &ledger.period.from.format("%Y-%m-%d").to_string(),
            "",
            "",
            "",
            "Solde d'ouverture",
            "",
            "",
            "",
            &format_amount_iso(section.opening),
        ])
        .map_err(map_csv_err)?;

        for line in &section.lines {
            wtr.write_record([
                &section.account_number,
                &section.account_name,
                &line.entry_date.format("%Y-%m-%d").to_string(),
                &line.entry_number.to_string(),
                &line.fiscal_year_name,
                &line.journal,
                &line.description,
                &line.counterpart.join(" "),
                &format_amount_iso(line.debit),
                &format_amount_iso(line.credit),
                &format_amount_iso(line.running_balance),
            ])
            .map_err(map_csv_err)?;
        }

        // Total des mouvements, puis clôture.
        wtr.write_record([
            &section.account_number,
            &section.account_name,
            "",
            "",
            "",
            "",
            "Total des mouvements",
            "",
            &format_amount_iso(section.total_debit),
            &format_amount_iso(section.total_credit),
            "",
        ])
        .map_err(map_csv_err)?;

        wtr.write_record([
            &section.account_number,
            &section.account_name,
            &ledger.period.to.format("%Y-%m-%d").to_string(),
            "",
            "",
            "",
            "Solde de clôture",
            "",
            "",
            "",
            &format_amount_iso(section.closing),
        ])
        .map_err(map_csv_err)?;
    }

    wtr.flush().map_err(map_io_err)?;
    Ok(())
}

// ============================================================================
// Tests unit (T10.1)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::balance_sheet::AccountBalance;
    use crate::journal_report::{JournalEntryLineRow, JournalEntryRow, JournalSection};
    use crate::period::ReportPeriod;
    use crate::trial_balance::TrialBalanceRow;
    use chrono::NaiveDate;
    use kesh_db::entities::AccountType;
    use kesh_db::entities::journal_entry::Journal;
    use rust_decimal_macros::dec;

    fn period() -> ReportPeriod {
        ReportPeriod {
            fiscal_year_id: 1,
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        }
    }

    // --- T10.1(b) : BOM + ; + CRLF pour les 4 CSV ---

    #[test]
    fn balance_sheet_csv_starts_with_bom() {
        let bs = BalanceSheet {
            period: period(),
            assets: vec![],
            liabilities: vec![],
            equity: vec![],
            total_assets: Decimal::ZERO,
            total_liabilities: Decimal::ZERO,
            total_equity: Decimal::ZERO,
            retained_earnings: Decimal::ZERO,
            equity_result: Decimal::ZERO,
            equation_holds: true,
        };
        let mut buf = Vec::new();
        render_balance_sheet_csv(&bs, &mut buf).unwrap();
        assert_eq!(&buf[..3], UTF8_BOM, "CSV must start with UTF-8 BOM");
    }

    #[test]
    fn balance_sheet_csv_uses_semicolon_delimiter() {
        let bs = BalanceSheet {
            period: period(),
            assets: vec![AccountBalance {
                account_id: 1,
                account_number: "1000".into(),
                account_name: "Caisse".into(),
                account_type: AccountType::Asset,
                active: true,
                balance: dec!(100),
                role: None,
            }],
            liabilities: vec![],
            equity: vec![],
            total_assets: dec!(100),
            total_liabilities: Decimal::ZERO,
            total_equity: Decimal::ZERO,
            retained_earnings: Decimal::ZERO,
            equity_result: dec!(100),
            equation_holds: true,
        };
        let mut buf = Vec::new();
        render_balance_sheet_csv(&bs, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        let header = body.lines().next().unwrap();
        assert!(
            header.contains(';'),
            "CSV header must use `;` delimiter, got: {header}"
        );
        assert!(
            !header.contains(','),
            "CSV header must NOT use `,` (Excel CH/DE convention), got: {header}"
        );
    }

    #[test]
    fn income_statement_csv_uses_crlf_terminator() {
        let is_ = IncomeStatement {
            period: period(),
            revenues: vec![],
            expenses: vec![],
            total_revenues: Decimal::ZERO,
            total_expenses: Decimal::ZERO,
            net_result: Decimal::ZERO,
        };
        let mut buf = Vec::new();
        render_income_statement_csv(&is_, &mut buf).unwrap();
        // Vérifier qu'il existe au moins une ligne se terminant par \r\n
        let s = String::from_utf8(buf[3..].to_vec()).unwrap();
        assert!(
            s.contains("\r\n"),
            "CSV must use CRLF terminator (RFC 4180), got first 50 bytes: {:?}",
            &s.chars().take(50).collect::<String>()
        );
    }

    #[test]
    fn trial_balance_csv_bom_and_delimiters() {
        let tb = TrialBalance {
            period: period(),
            rows: vec![],
            total_debit: Decimal::ZERO,
            total_credit: Decimal::ZERO,
            balanced: true,
        };
        let mut buf = Vec::new();
        render_trial_balance_csv(&tb, &mut buf).unwrap();
        assert_eq!(&buf[..3], UTF8_BOM);
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        assert!(body.contains(';'));
        assert!(body.contains("\r\n"));
    }

    // --- T10.1(c) : escaping RFC 4180 — séparateur, quote, retour-ligne ---

    /// Pass 1 ECH-L4 (a) : nom contenant `;` doit être quoté.
    /// Pass 1 ECH-L4 (b) : nom contenant `"` doit doubler la quote → `""`.
    /// Pass 1 ECH-L4 (c) : nom contenant `\n` doit être quoté.
    #[test]
    fn csv_escapes_special_characters_per_rfc_4180() {
        let bs = BalanceSheet {
            period: period(),
            assets: vec![
                AccountBalance {
                    account_id: 1,
                    account_number: "1000".into(),
                    account_name: "Caisse; intérimaire".into(),
                    account_type: AccountType::Asset,
                    active: true,
                    balance: dec!(100),
                    role: None,
                },
                AccountBalance {
                    account_id: 2,
                    account_number: "1001".into(),
                    account_name: r#"Nom avec "guillemets""#.into(),
                    account_type: AccountType::Asset,
                    active: true,
                    balance: dec!(50),
                    role: None,
                },
                AccountBalance {
                    account_id: 3,
                    account_number: "1002".into(),
                    account_name: "Ligne 1\nLigne 2".into(),
                    account_type: AccountType::Asset,
                    active: true,
                    balance: dec!(25),
                    role: None,
                },
            ],
            liabilities: vec![],
            equity: vec![],
            total_assets: dec!(175),
            total_liabilities: Decimal::ZERO,
            total_equity: Decimal::ZERO,
            retained_earnings: Decimal::ZERO,
            equity_result: dec!(175),
            equation_holds: true,
        };
        let mut buf = Vec::new();
        render_balance_sheet_csv(&bs, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        // (a) ; → quoté
        assert!(
            body.contains(r#""Caisse; intérimaire""#),
            "name containing `;` must be quoted, body:\n{body}"
        );
        // (b) " → doublé
        assert!(
            body.contains(r#""Nom avec ""guillemets""""#),
            "name containing `\"` must double-escape, body:\n{body}"
        );
        // (c) \n → quoté
        assert!(
            body.contains("\"Ligne 1\nLigne 2\""),
            "name containing newline must be quoted, body:\n{body}"
        );
    }

    // --- AC #17 + T3.5 : rapport vide = uniquement ligne d'en-tête ---

    #[test]
    fn empty_balance_sheet_csv_has_header_only() {
        let bs = BalanceSheet {
            period: period(),
            assets: vec![],
            liabilities: vec![],
            equity: vec![],
            total_assets: Decimal::ZERO,
            total_liabilities: Decimal::ZERO,
            total_equity: Decimal::ZERO,
            retained_earnings: Decimal::ZERO,
            equity_result: Decimal::ZERO,
            equation_holds: true,
        };
        let mut buf = Vec::new();
        render_balance_sheet_csv(&bs, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        // 1 ligne header + 0 data rows = exactement 1 CRLF terminé → 1 ligne effective
        let lines: Vec<&str> = body.split("\r\n").filter(|l| !l.is_empty()).collect();
        assert_eq!(
            lines.len(),
            1,
            "empty CSV must have only header line, got: {lines:?}"
        );
        assert_eq!(lines[0], "Section;NumeroCompte;NomCompte;Solde");
    }

    /// Review Pass 1 — actif/passif vides mais report/résultat non nuls et opposés
    /// (bénéfice N entièrement dépensé en N+1) : la section fonds propres NE DOIT PAS
    /// être masquée (sinon bilan exporté déséquilibré). Régression du finding HIGH.
    #[test]
    fn balance_sheet_csv_shows_equity_when_assets_liabilities_net_to_zero() {
        let bs = BalanceSheet {
            period: period(),
            assets: vec![],
            liabilities: vec![],
            equity: vec![],
            total_assets: Decimal::ZERO,
            total_liabilities: Decimal::ZERO,
            total_equity: Decimal::ZERO,
            retained_earnings: dec!(1000),
            equity_result: dec!(-1000),
            equation_holds: true,
        };
        let mut buf = Vec::new();
        render_balance_sheet_csv(&bs, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        assert!(
            body.contains("Résultat reporté"),
            "la ligne « Résultat reporté » doit figurer, body:\n{body}"
        );
        assert!(
            body.contains("Résultat de l'exercice"),
            "la ligne « Résultat de l'exercice » doit figurer, body:\n{body}"
        );
        // Total passifs + capitaux propres = 0 + 1000 + (-1000) = 0 (équilibré, présent)
        assert!(
            body.contains("Total passifs + capitaux propres"),
            "le total fonds propres doit figurer, body:\n{body}"
        );
    }

    /// Story 14-3c — la section CapitauxPropres itemise les comptes physiques de fonds
    /// propres (dans l'ordre reçu), la ligne calculée est marquée « (calculé) » (D1), et
    /// les comptes de fonds propres ne figurent PAS dans les lignes Passifs (partition D2).
    #[test]
    fn balance_sheet_csv_equity_section_by_role() {
        let equity = |number: &str, name: &str, bal, role| AccountBalance {
            account_id: 1,
            account_number: number.into(),
            account_name: name.into(),
            account_type: AccountType::Liability,
            active: true,
            balance: bal,
            role: Some(role),
        };
        let bs = BalanceSheet {
            period: period(),
            assets: vec![AccountBalance {
                account_id: 9,
                account_number: "1000".into(),
                account_name: "Banque".into(),
                account_type: AccountType::Asset,
                active: true,
                balance: dec!(3000),
                role: None,
            }],
            liabilities: vec![AccountBalance {
                account_id: 8,
                account_number: "2000".into(),
                account_name: "Fournisseurs".into(),
                account_type: AccountType::Liability,
                active: true,
                balance: dec!(500),
                role: Some(kesh_db::entities::AccountRole::Payable),
            }],
            equity: vec![
                equity(
                    "2800",
                    "Capital",
                    dec!(2000),
                    kesh_db::entities::AccountRole::EquityCapital,
                ),
                equity(
                    "2970",
                    "Bénéfice reporté",
                    dec!(500),
                    kesh_db::entities::AccountRole::RetainedEarnings,
                ),
            ],
            total_assets: dec!(3000),
            total_liabilities: dec!(500),
            total_equity: dec!(2500),
            retained_earnings: dec!(0),
            equity_result: dec!(0),
            equation_holds: true,
        };
        let mut buf = Vec::new();
        render_balance_sheet_csv(&bs, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        // Comptes physiques itemisés dans la section CapitauxPropres.
        assert!(
            body.contains("CapitauxPropres;2800;Capital;2000.00"),
            "{body}"
        );
        assert!(
            body.contains("CapitauxPropres;2970;Bénéfice reporté;500.00"),
            "{body}"
        );
        // Ligne calculée explicitement marquée « (calculé) » (D1).
        assert!(body.contains("Résultat reporté (calculé)"), "{body}");
        assert!(body.contains("Total capitaux propres"), "{body}");
        // Le capital NE figure PAS dans une ligne Passifs (partition D2).
        assert!(
            !body.contains("Passifs;2800"),
            "capital absent des passifs\n{body}"
        );
        // La dette réelle reste bien dans Passifs.
        assert!(body.contains("Passifs;2000;Fournisseurs;500.00"), "{body}");
        assert!(body.contains("Total passifs;;;500.00"), "{body}");
    }

    #[test]
    fn empty_journal_report_csv_has_header_only() {
        let jr = JournalReport {
            period: period(),
            journals: vec![JournalSection {
                journal: Journal::Achats,
                entries: vec![],
                section_total_debit: Decimal::ZERO,
                section_total_credit: Decimal::ZERO,
            }],
            grand_total_debit: Decimal::ZERO,
            grand_total_credit: Decimal::ZERO,
        };
        let mut buf = Vec::new();
        render_journal_report_csv(&jr, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        let lines: Vec<&str> = body.split("\r\n").filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 1, "empty journals CSV must have only header");
    }

    // --- T3.6 : format décimal toujours 2 décimales ---

    #[test]
    fn format_amount_iso_always_two_decimals() {
        assert_eq!(format_amount_iso(Decimal::ZERO), "0.00");
        assert_eq!(format_amount_iso(dec!(1234.5)), "1234.50");
        assert_eq!(format_amount_iso(dec!(-1234.56)), "-1234.56");
        assert_eq!(format_amount_iso(dec!(1000000)), "1000000.00");
    }

    // --- AC #15 : journals CSV contient toutes les colonnes attendues ---

    #[test]
    fn journal_report_csv_has_8_columns() {
        let jr = JournalReport {
            period: period(),
            journals: vec![JournalSection {
                journal: Journal::Ventes,
                entries: vec![JournalEntryRow {
                    entry_id: 1,
                    entry_number: 42,
                    entry_date: NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
                    description: "Test".into(),
                    lines: vec![JournalEntryLineRow {
                        account_id: 1,
                        account_number: "1000".into(),
                        account_name: "Caisse".into(),
                        debit: dec!(100),
                        credit: dec!(0),
                        line_order: 0,
                    }],
                }],
                section_total_debit: dec!(100),
                section_total_credit: Decimal::ZERO,
            }],
            grand_total_debit: dec!(100),
            grand_total_credit: Decimal::ZERO,
        };
        let mut buf = Vec::new();
        render_journal_report_csv(&jr, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        let lines: Vec<&str> = body.split("\r\n").filter(|l| !l.is_empty()).collect();
        assert!(lines.len() >= 2);
        // Header : 8 colonnes
        let header_cols: Vec<&str> = lines[0].split(';').collect();
        assert_eq!(
            header_cols.len(),
            8,
            "journals CSV header must have 8 columns"
        );
        // Data row : 8 colonnes, date ISO, journal name
        let data_cols: Vec<&str> = lines[1].split(';').collect();
        assert_eq!(data_cols.len(), 8);
        assert_eq!(data_cols[0], "Ventes");
        assert_eq!(data_cols[1], "2026-05-15");
        assert_eq!(data_cols[2], "42");
    }

    // --- AC #14 : trial balance CSV header + total row ---

    #[test]
    fn trial_balance_csv_header_and_total_row() {
        let tb = TrialBalance {
            period: period(),
            rows: vec![TrialBalanceRow {
                account_id: 1,
                account_number: "1000".into(),
                account_name: "Caisse".into(),
                account_type: AccountType::Asset,
                active: true,
                total_debit: dec!(200),
                total_credit: dec!(50),
                balance: dec!(150),
            }],
            total_debit: dec!(200),
            total_credit: dec!(50),
            balanced: true,
        };
        let mut buf = Vec::new();
        render_trial_balance_csv(&tb, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        let lines: Vec<&str> = body.split("\r\n").filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 3, "1 header + 1 row + 1 total = 3 lines");
        assert_eq!(
            lines[0],
            "NumeroCompte;NomCompte;TotalDebit;TotalCredit;Solde"
        );
        assert!(lines[2].starts_with("Total;"));
    }

    // Story 21-7 — CSV balance âgée.
    use crate::aged_receivables::{AgedBucket, AgedReceivables, AgedReceivablesRow};

    fn aged_bucket() -> AgedBucket {
        AgedBucket {
            not_due: dec!(110),
            days_1_to_30: dec!(205),
            days_31_to_60: dec!(307),
            days_61_to_90: dec!(409),
            days_over_90: dec!(1581),
            total: dec!(2612),
        }
    }

    #[test]
    fn aged_receivables_csv_bom_header_rows_total() {
        let report = AgedReceivables {
            as_of: chrono::NaiveDate::from_ymd_opt(2026, 7, 20).unwrap(),
            rows: vec![AgedReceivablesRow {
                contact_id: 1,
                contact_name: "Alpha SA".into(),
                buckets: aged_bucket(),
            }],
            totals: aged_bucket(),
        };
        let mut buf = Vec::new();
        render_aged_receivables_csv(&report, &mut buf).unwrap();

        // BOM UTF-8 en tête.
        assert_eq!(&buf[..3], &[0xEF, 0xBB, 0xBF]);
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        let lines: Vec<&str> = body.split("\r\n").filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 3, "1 header + 1 contact + 1 total");
        assert_eq!(lines[0], "Contact;Non échu;1-30;31-60;61-90;90+;Total");
        assert_eq!(
            lines[1],
            "Alpha SA;110.00;205.00;307.00;409.00;1581.00;2612.00"
        );
        assert!(lines[2].starts_with("Total général;"));
        assert!(lines[2].ends_with(";2612.00"));
    }

    #[test]
    fn aged_receivables_csv_empty_is_header_only() {
        let report = AgedReceivables {
            as_of: chrono::NaiveDate::from_ymd_opt(2026, 7, 20).unwrap(),
            rows: vec![],
            totals: AgedBucket {
                not_due: Decimal::ZERO,
                days_1_to_30: Decimal::ZERO,
                days_31_to_60: Decimal::ZERO,
                days_61_to_90: Decimal::ZERO,
                days_over_90: Decimal::ZERO,
                total: Decimal::ZERO,
            },
        };
        let mut buf = Vec::new();
        render_aged_receivables_csv(&report, &mut buf).unwrap();
        let body = String::from_utf8(buf[3..].to_vec()).unwrap();
        let lines: Vec<&str> = body.split("\r\n").filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 1, "rapport vide = en-tête seul");
        assert_eq!(lines[0], "Contact;Non échu;1-30;31-60;61-90;90+;Total");
    }

    // ------------------------------------------------------------------
    // Story 24-1 — grand livre
    // ------------------------------------------------------------------

    fn ledger_fixture() -> GeneralLedger {
        use crate::general_ledger::{GeneralLedger, LedgerLine, LedgerPeriod, LedgerSection};
        GeneralLedger {
            period: LedgerPeriod {
                from: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                to: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
            },
            sections: vec![LedgerSection {
                account_id: 7,
                account_number: "1020".into(),
                account_name: "Banque".into(),
                account_type: AccountType::Asset,
                active: true,
                balance_side: "debit",
                opening: dec!(500.00),
                lines: vec![LedgerLine {
                    line_id: 1,
                    entry_id: 1,
                    entry_date: NaiveDate::from_ymd_opt(2026, 3, 4).unwrap(),
                    fiscal_year_id: 3,
                    fiscal_year_name: "Exercice 2026".into(),
                    entry_number: 12,
                    journal: "Banque".into(),
                    description: "Loyer mars".into(),
                    counterpart: vec!["6000".into()],
                    debit: dec!(1200.00),
                    credit: dec!(0),
                    running_balance: dec!(1700.00),
                }],
                total_debit: dec!(1200.00),
                total_credit: dec!(0),
                closing: dec!(1700.00),
                unnatural_balance: false,
                fiscal_year_breaks: vec![],
                line_count: 1,
            }],
        }
    }

    /// ⚠️ Ouverture et clôture sont des **lignes du tableau**, pas des
    /// métadonnées d'en-tête : c'est ce qui rend le fichier réconciliable dans
    /// un tableur, où l'on additionne une colonne.
    #[test]
    fn general_ledger_csv_writes_opening_and_closing_as_rows() {
        let mut buf = Vec::new();
        render_general_ledger_csv(&ledger_fixture(), &mut buf).expect("csv");
        let out = String::from_utf8(buf).expect("utf-8");
        let lines: Vec<&str> = out.lines().collect();

        assert!(lines[0].starts_with('\u{feff}') || lines[0].contains("NumeroCompte"));
        let corps = out.replace('\u{feff}', "");
        let l: Vec<&str> = corps.lines().collect();
        assert!(l[1].contains("Solde d'ouverture"), "ligne 1 = {}", l[1]);
        assert!(l[1].ends_with("500.00"));
        assert!(l[2].contains("Loyer mars"));
        assert!(l[3].contains("Total des mouvements"));
        assert!(l[4].contains("Solde de clôture"));
        assert!(l[4].ends_with("1700.00"));
    }

    /// La colonne « Exercice » porte le **nom** de l'exercice, jamais son id —
    /// un identifiant de base de données ne dit rien à qui lit le fichier, et
    /// c'est pourtant ce qui désambiguïse le numéro de pièce.
    #[test]
    fn general_ledger_csv_names_the_fiscal_year_instead_of_its_id() {
        let mut buf = Vec::new();
        render_general_ledger_csv(&ledger_fixture(), &mut buf).expect("csv");
        let out = String::from_utf8(buf).expect("utf-8");
        assert!(out.contains("Exercice 2026"), "le nom manque : {out}");
        // L'id (3) ne doit pas s'être glissé dans la colonne Exercice.
        assert!(!out.contains(";12;3;"), "l'id d'exercice est rendu : {out}");
    }
}
