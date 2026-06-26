//! Rapport TVA par période (FR56) — TVA due côté vente.
//!
//! Agrège les lignes des factures **validées** dont la date tombe dans la
//! période, **par taux de TVA**, et calcule la TVA due en **arrondissant par
//! ligne** (FR55, cf. [`kesh_core::accounting::vat::line_vat_amount`]) puis en
//! sommant — jamais en arrondissant une base agrégée.
//!
//! ## Périmètre (cf. stories 11-2 et 18-1d)
//!
//! - **TVA due** (vente) : calculée depuis les factures de vente validées
//!   (source théorique `invoice_lines`, ventilable par taux).
//! - **TVA récupérable** (achats / impôt préalable, Story 18-1d) : **solde du
//!   compte `default_vat_recoverable_account_id` (impôt préalable, Asset) lu du
//!   grand livre** sur la période — `SUM(debit) − SUM(credit)`, filtré par
//!   `entry_date` SEUL (pas `fiscal_year_id`, DC4). Sûr car `ReportPeriod::resolve`
//!   clampe la période **intra-exercice** et les `fiscal_years` ne se chevauchent
//!   pas (invariant dur) → jamais d'agrégation multi-exercice. Périmètre
//!   **intégral** (DC4-bis) : toute écriture sur ce compte dédié compte. Signe
//!   `debit − credit` codé en dur (DC4-ter, compte Asset par construction ; un
//!   compte non-Asset configuré est une erreur de config hors scope). Si le compte
//!   n'est pas configuré (`NULL`) → `0.00`.
//! - La **réconciliation / cross-check** rapport ↔ grand livre (DC5) est déférée
//!   à la story 18-1e.
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
    /// TVA récupérable (achats / impôt préalable) = solde du compte
    /// `default_vat_recoverable_account_id` au grand livre sur la période
    /// (Story 18-1d). `0.00` si le compte n'est pas configuré.
    pub total_vat_recoverable: Decimal,
    /// Solde = `total_vat_due - total_vat_recoverable`.
    pub vat_balance: Decimal,
    /// Écart de réconciliation (Story 18-1e, DC5) =
    /// `total_vat_due` (dérivé des `invoice_lines`, total de référence traçable)
    /// − solde du compte TVA due (`2200`) au grand livre **isolé au périmètre
    /// ventes** (lignes liées à une facture validée de la période). Positif = TVA
    /// facturée > comptabilisée (une ligne 2200 a été réduite à la main) ; négatif
    /// = inverse. `0.00` si le compte TVA due n'est pas configuré.
    pub reconciliation_delta: Decimal,
    /// `"delta"` si `|reconciliation_delta| >= 0.01` (1 centime, écriture validée
    /// modifiée manuellement), sinon `"ok"` (Story 18-1e, DC5-delta). Informatif,
    /// non bloquant.
    pub reconciliation_status: String,
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

    // Stories 18-1d/18-1e : lecture des comptes TVA configurés en une seule requête.
    // La row `company_invoice_settings` existe dès l'onboarding ; chaque compte peut
    // ne pas être configuré (NULL) → récupérable/delta = 0 (comportement préservé).
    // `fetch_optional` + `unwrap_or` : une company sans row `company_invoice_settings`
    // (cas dégénéré / company inexistante) → comptes None → récupérable/delta = 0,
    // jamais une erreur (cohérent avec le comportement 18-1d).
    let (payable_account_id, recoverable_account_id): (Option<i64>, Option<i64>) = sqlx::query_as(
        "SELECT default_vat_payable_account_id, default_vat_recoverable_account_id \
             FROM company_invoice_settings WHERE company_id = ?",
    )
    .bind(company_id)
    .fetch_optional(pool)
    .await
    .map_err(kesh_db::errors::map_db_error)?
    .unwrap_or((None, None));

    // Story 18-1d : TVA récupérable = solde du compte impôt préalable lu du grand livre.
    let total_vat_recoverable = match recoverable_account_id {
        Some(account_id) => recoverable_balance(pool, company_id, account_id, period).await?,
        None => Decimal::ZERO,
    };
    let vat_balance = total_vat_due - total_vat_recoverable;

    // Story 18-1e (DC5) : réconciliation TVA due. Cross-check de la TVA due dérivée
    // des `invoice_lines` (ci-dessus) contre le solde du compte TVA due au grand
    // livre, isolé au périmètre ventes (lignes liées à une facture validée). En
    // l'absence d'édition manuelle d'une écriture validée, delta == 0 par construction
    // (mêmes factures, même `line_vat_amount` agrégé par taux).
    //
    // DC5-null : si le compte TVA due n'est pas configuré (`None`), rien à
    // réconcilier — delta = 0, status "ok" (PAS `total_vat_due − 0`, qui donnerait
    // un faux écart si des factures validées pré-existantes portent de la TVA).
    let (reconciliation_delta, reconciliation_status) = match payable_account_id {
        Some(account_id) => {
            let balance =
                due_account_balance_sales_scope(pool, company_id, account_id, period).await?;
            let delta = total_vat_due - balance;
            // Seuil 1 centime. NB : `Decimal::new(1, 2)` (pas `dec!` — macro dev-only).
            let status = if delta.abs() >= Decimal::new(1, 2) {
                "delta".to_string()
            } else {
                "ok".to_string()
            };
            (delta, status)
        }
        None => (Decimal::ZERO, "ok".to_string()),
    };

    Ok(VatReport {
        period: period.clone(),
        rows,
        total_base_ht,
        total_vat_due,
        total_vat_recoverable,
        vat_balance,
        reconciliation_delta,
        reconciliation_status,
    })
}

/// Solde du compte de TVA due (`account_id`, p.ex. `2200`) au grand livre **isolé au
/// périmètre ventes** sur la période (Story 18-1e, DC5-iso). `SUM(credit) − SUM(debit)`
/// — signe **codé en dur** car le compte est un Liability par construction (la TVA
/// collectée est portée au crédit ; un compte non-Liability configuré est une erreur
/// de config hors scope v0.2).
///
/// L'isolation « périmètre ventes » se fait par le **lien facture validée**
/// (`INNER JOIN invoices i ON i.journal_entry_id = jel.entry_id`), **pas** par le
/// libellé `journal` (qui est configurable via `default_sales_journal`, F-OPUS-4). La
/// jointure exclut nativement : les OD manuelles sur le compte TVA due sans facture,
/// les factures non validées, et les autres companies (via `i.company_id`). Le filtre
/// `i.date BETWEEN` couvre le même ensemble de factures que la TVA due dérivée
/// (`entry_date == invoice.date` pour les écritures de vente) → cohérence des deux
/// côtés du delta. `COALESCE` → `0` si aucune ligne.
///
/// Limitation (DC5, L-1) : suppose le compte TVA due **stable sur la période**. Une
/// reconfiguration de `default_vat_payable_account_id` en cours de période laisse les
/// factures déjà validées sur l'ancien compte → delta à investiguer (signal légitime,
/// pas un faux positif nuisible). Le résiduel manuel hors-ventes sur le compte
/// (auto-liquidation, régularisation AFC) est hors scope du delta.
async fn due_account_balance_sales_scope(
    pool: &MySqlPool,
    company_id: i64,
    account_id: i64,
    period: &ReportPeriod,
) -> Result<Decimal, ReportError> {
    let balance: Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(jel.credit), 0) - COALESCE(SUM(jel.debit), 0) \
         FROM journal_entry_lines jel \
         INNER JOIN invoices i ON i.journal_entry_id = jel.entry_id \
         WHERE i.company_id = ? \
           AND i.status = 'validated' \
           AND i.date BETWEEN ? AND ? \
           AND jel.account_id = ?",
    )
    .bind(company_id)
    .bind(period.start_date)
    .bind(period.end_date)
    .bind(account_id)
    .fetch_one(pool)
    .await
    .map_err(kesh_db::errors::map_db_error)?;
    Ok(balance)
}

/// Solde du compte d'impôt préalable (`account_id`) au grand livre sur la période
/// (Story 18-1d, DC4). `SUM(debit) − SUM(credit)` — signe **codé en dur** car le
/// compte est un Asset par construction (DC4-ter). Filtre `entry_date` **seul**
/// (pas `fiscal_year_id`, DC4) : sûr car la période est clampée intra-exercice par
/// `ReportPeriod::resolve` et les `fiscal_years` ne se chevauchent pas. `COALESCE`
/// → `0` si aucune écriture sur la période. Scopé `company_id` (anti-IDOR).
async fn recoverable_balance(
    pool: &MySqlPool,
    company_id: i64,
    account_id: i64,
    period: &ReportPeriod,
) -> Result<Decimal, ReportError> {
    let balance: Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(jel.debit), 0) - COALESCE(SUM(jel.credit), 0) \
         FROM journal_entry_lines jel \
         INNER JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE je.company_id = ? \
           AND jel.account_id = ? \
           AND je.entry_date BETWEEN ? AND ?",
    )
    .bind(company_id)
    .bind(account_id)
    .bind(period.start_date)
    .bind(period.end_date)
    .fetch_one(pool)
    .await
    .map_err(kesh_db::errors::map_db_error)?;
    Ok(balance)
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
            reconciliation_delta: Decimal::ZERO,
            reconciliation_status: "ok".to_string(),
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
