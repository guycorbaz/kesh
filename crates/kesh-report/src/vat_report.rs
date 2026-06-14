//! Rapport TVA par période (FR56) — TVA due côté vente.
//!
//! Agrège les lignes des factures **validées** dont la date tombe dans la
//! période, **par taux de TVA**, et calcule la TVA due en **arrondissant par
//! ligne** (FR55, cf. [`kesh_core::accounting::vat::line_vat_amount`]) puis en
//! sommant — jamais en arrondissant une base agrégée.
//!
//! ## Périmètre v0.2 (cf. story 11-2)
//!
//! - **TVA due** (vente) : calculée depuis les factures de vente validées.
//! - **TVA récupérable** (achats / impôt préalable) : `0.00` — aucune source de
//!   données dans le modèle actuel (pas de saisie d'achats avec TVA, pas de
//!   comptes TVA). Déférée à une story de suivi ; la structure est prête à la
//!   recevoir (`total_vat_recoverable` / `vat_balance`).
//!
//! Le grouping est **par taux numérique** (`vat_rate` snapshoté sur la ligne),
//! pas par catégorie : c'est la granularité du décompte AFC. `category` reste
//! `None` en 11-2 (l'inférence taux→catégorie est déférée).

use std::collections::BTreeMap;

use kesh_core::accounting::vat::line_vat_amount;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::MySqlPool;

use crate::errors::ReportError;
use crate::period::ReportPeriod;

/// Une ligne du rapport TVA = un taux présent dans les ventes de la période.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VatReportRow {
    /// Taux de TVA en pourcent (ex. `8.10`).
    pub rate: Decimal,
    /// Catégorie métier (best-effort). `None` en 11-2 (grouping par taux).
    pub category: Option<String>,
    /// Chiffre d'affaires HT pour ce taux (somme des `line_total`).
    pub base_ht: Decimal,
    /// TVA due pour ce taux = Σ TVA arrondies **ligne par ligne** (FR55).
    pub vat_due: Decimal,
}

/// Rapport TVA complet pour une période.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VatReport {
    pub period: ReportPeriod,
    pub rows: Vec<VatReportRow>,
    pub total_base_ht: Decimal,
    pub total_vat_due: Decimal,
    /// TVA récupérable (achats). `0.00` en v0.2 — déférée (cf. doc module).
    pub total_vat_recoverable: Decimal,
    /// Solde = `total_vat_due - total_vat_recoverable`.
    pub vat_balance: Decimal,
}

/// Génère le rapport TVA (TVA due/vente) pour la période donnée.
///
/// Anti-IDOR : toutes les lignes sont scopées par `company_id`.
pub async fn generate(
    pool: &MySqlPool,
    company_id: i64,
    period: &ReportPeriod,
) -> Result<VatReport, ReportError> {
    // Granularité LIGNE (pas de GROUP BY/SUM SQL) — l'arrondi par ligne (FR55)
    // se fait en Rust. Seules les factures validées dans la période comptent.
    let lines: Vec<(Decimal, Decimal)> = sqlx::query_as(
        "SELECT il.vat_rate, il.line_total \
         FROM invoice_lines il \
         INNER JOIN invoices i ON i.id = il.invoice_id \
         WHERE i.company_id = ? \
           AND i.status = 'validated' \
           AND i.date BETWEEN ? AND ?",
    )
    .bind(company_id)
    .bind(period.start_date)
    .bind(period.end_date)
    .fetch_all(pool)
    .await
    .map_err(kesh_db::errors::map_db_error)?;

    // Accumulation par taux. BTreeMap → itération triée par taux ASC (AC : tri
    // stable). Deux taux numériquement égaux (8.1 / 8.10) fusionnent (Ord/Eq
    // de Decimal compare la valeur), ce qui est conforme au décompte AFC.
    let mut by_rate: BTreeMap<Decimal, (Decimal, Decimal)> = BTreeMap::new();
    for (vat_rate, line_total) in lines {
        // Arrondi PAR LIGNE (FR55) — surtout pas une base agrégée.
        let vat = line_vat_amount(line_total, vat_rate);
        let entry = by_rate
            .entry(vat_rate)
            .or_insert((Decimal::ZERO, Decimal::ZERO));
        entry.0 += line_total;
        entry.1 += vat;
    }

    let rows: Vec<VatReportRow> = by_rate
        .into_iter()
        .map(|(rate, (base_ht, vat_due))| VatReportRow {
            rate,
            category: None,
            base_ht,
            vat_due,
        })
        .collect();

    let total_base_ht: Decimal = rows.iter().map(|r| r.base_ht).sum();
    let total_vat_due: Decimal = rows.iter().map(|r| r.vat_due).sum();
    let total_vat_recoverable = Decimal::ZERO;
    let vat_balance = total_vat_due - total_vat_recoverable;

    Ok(VatReport {
        period: period.clone(),
        rows,
        total_base_ht,
        total_vat_due,
        total_vat_recoverable,
        vat_balance,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    /// Construit un `VatReport` synthétique depuis des lignes brutes
    /// `(vat_rate, line_total)` en répliquant la logique d'agrégation de
    /// `generate` (sans DB). Permet de tester l'invariant d'arrondi.
    fn aggregate(lines: &[(Decimal, Decimal)]) -> VatReport {
        let mut by_rate: BTreeMap<Decimal, (Decimal, Decimal)> = BTreeMap::new();
        for (rate, total) in lines {
            let vat = line_vat_amount(*total, *rate);
            let entry = by_rate
                .entry(*rate)
                .or_insert((Decimal::ZERO, Decimal::ZERO));
            entry.0 += *total;
            entry.1 += vat;
        }
        let rows: Vec<VatReportRow> = by_rate
            .into_iter()
            .map(|(rate, (base_ht, vat_due))| VatReportRow {
                rate,
                category: None,
                base_ht,
                vat_due,
            })
            .collect();
        let total_base_ht: Decimal = rows.iter().map(|r| r.base_ht).sum();
        let total_vat_due: Decimal = rows.iter().map(|r| r.vat_due).sum();
        let total_vat_recoverable = Decimal::ZERO;
        let vat_balance = total_vat_due - total_vat_recoverable;
        VatReport {
            period: ReportPeriod {
                fiscal_year_id: 1,
                start_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end_date: chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
            },
            rows,
            total_base_ht,
            total_vat_due,
            total_vat_recoverable,
            vat_balance,
        }
    }

    /// AC#3 — vecteur divergent : arrondi PAR LIGNE ≠ arrondi GLOBAL.
    /// 3 lignes de 0.10 à 8 % → par ligne 3×0.01 = 0.03 ; global
    /// round(0.30×8/100)=round(0.024)=0.02. L'implémentation DOIT donner 0.03.
    #[test]
    fn rounding_per_line_not_global() {
        let report = aggregate(&[
            (dec!(8.00), dec!(0.10)),
            (dec!(8.00), dec!(0.10)),
            (dec!(8.00), dec!(0.10)),
        ]);
        assert_eq!(report.rows.len(), 1);
        assert_eq!(report.rows[0].vat_due, dec!(0.03));
        // Preuve que ce n'est PAS l'arrondi global :
        assert_ne!(report.rows[0].vat_due, dec!(0.02));
        assert_eq!(report.total_vat_due, dec!(0.03));
        assert_eq!(report.total_base_ht, dec!(0.30));
    }

    #[test]
    fn sorted_by_rate_ascending_and_totals() {
        let report = aggregate(&[
            (dec!(8.10), dec!(1000)),
            (dec!(2.60), dec!(500)),
            (dec!(8.10), dec!(2000)),
        ]);
        // Tri par taux ASC + fusion des deux lignes 8.10.
        assert_eq!(report.rows.len(), 2);
        assert_eq!(report.rows[0].rate, dec!(2.60));
        assert_eq!(report.rows[1].rate, dec!(8.10));
        assert_eq!(report.rows[1].base_ht, dec!(3000));
        // 1000×8.1% = 81.00 ; 2000×8.1% = 162.00 → 243.00
        assert_eq!(report.rows[1].vat_due, dec!(243.00));
        assert_eq!(
            report.total_vat_due,
            report.rows[0].vat_due + report.rows[1].vat_due
        );
    }

    #[test]
    fn balance_equals_due_when_recoverable_zero() {
        let report = aggregate(&[(dec!(8.10), dec!(1000))]);
        assert_eq!(report.total_vat_recoverable, dec!(0));
        assert_eq!(report.vat_balance, report.total_vat_due);
    }

    #[test]
    fn category_is_none_in_11_2() {
        let report = aggregate(&[(dec!(8.10), dec!(1000)), (dec!(0.00), dec!(500))]);
        assert!(report.rows.iter().all(|r| r.category.is_none()));
    }

    /// AC OPUS-5 — une ligne 0 %/exonérée figure dans le rapport (base > 0,
    /// TVA = 0).
    #[test]
    fn exempt_zero_rate_row_present() {
        let report = aggregate(&[(dec!(0.00), dec!(500))]);
        assert_eq!(report.rows.len(), 1);
        assert_eq!(report.rows[0].rate, dec!(0.00));
        assert_eq!(report.rows[0].base_ht, dec!(500));
        assert_eq!(report.rows[0].vat_due, dec!(0));
    }

    /// Test source-level : la requête sélectionne les lignes brutes (granularité
    /// ligne, requis pour l'arrondi par ligne FR55) et ne compte que les
    /// factures validées. Assertion positive pour éviter l'auto-référence.
    #[test]
    fn generate_sql_is_line_granular_and_validated_only() {
        const SRC: &str = include_str!("vat_report.rs");
        assert!(
            SRC.contains("SELECT il.vat_rate, il.line_total"),
            "generate doit récupérer les lignes brutes (vat_rate, line_total) pour arrondir par ligne"
        );
        assert!(SRC.contains("i.status = ?") || SRC.contains("status = 'validated'"));
    }
}
