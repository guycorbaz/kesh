//! Balance âgée des créances clients (Story 21-7, #231, §E items 23/25).
//!
//! Répartit l'encours débiteur — le **reste dû** de chaque facture validée non
//! soldée, après règlements partiels et avoirs (Story 25-4-b1, #416) — par contact et par tranche d'ancienneté relative à une date de
//! référence `as_of` :
//!
//! `Non échu | 1-30 | 31-60 | 61-90 | 90+` jours de retard (`days = as_of − due_date`).
//!
//! Invariants :
//! - **Reste dû dérivé** (`INVOICE_AMOUNT_DUE_DERIVED_SQL` sur
//!   `amount_due_derived_joins`, Story 25-4-b1) : TTC − avoir émis − Σ
//!   règlements, jamais le TTC seul ni `total_amount` (le HT). Le TTC y est
//!   arrondi PAR LIGNE (DC7), asservi au helper Rust `invoice_total_ttc`
//!   (`tests/aged_receivables.rs`) ; le reste dû joint, à `amount_due`
//!   (`kesh-db/tests/invoice_amount_due_parity.rs`).
//! - ⛔ **Concordance avec le grand livre** — le total égale le solde du compte
//!   débiteurs **si et seulement si** ce compte n'est mouvementé que par les
//!   écritures de vente, de règlement client et d'avoir de factures validées
//!   (contre-passations comprises), datées au plus tard à `as_of`, sans que le
//!   compte débiteurs ait changé dans les réglages. Tout autre mouvement — solde
//!   d'ouverture, écriture au journal, rapprochement hors facture, compensation
//!   fournisseur, données antérieures à la 0.12.1 — l'en écarte, et la balance
//!   âgée ne le montre pas (règle, non liste : `tests/aged_receivables.rs`).
//! - **Factures suspendues INCLUSES** (D10) : le prédicat postes ouverts
//!   n'exclut PAS `dunning_paused_at` — une facture suspendue reste dans la
//!   balance âgée (elle ne sort que de la liste « à rappeler »).
//! - **`as_of` bindé** (pas `UTC_DATE()` en dur) pour la testabilité.
//! - Totaux généraux **sommés en Rust** (patron `balance_sheet`), pas en SQL.

use chrono::NaiveDate;
use kesh_db::repositories::invoice_settlements::{
    INVOICE_AMOUNT_DUE_DERIVED_SQL, amount_due_derived_joins,
};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::MySqlPool;

use crate::errors::ReportError;

/// Montants d'une tranche d'ancienneté (une ligne de contact ou le total général).
///
/// Renames serde **explicites** (piège n°2 21-7) : `rename_all = "camelCase"`
/// sur `days_1_to_30` produirait certes `days1To30`, mais on fige le contrat
/// vis-à-vis du miroir TypeScript par des renames nominatifs.
#[derive(Debug, Clone, Serialize)]
pub struct AgedBucket {
    #[serde(rename = "notDue")]
    pub not_due: Decimal,
    #[serde(rename = "days1To30")]
    pub days_1_to_30: Decimal,
    #[serde(rename = "days31To60")]
    pub days_31_to_60: Decimal,
    #[serde(rename = "days61To90")]
    pub days_61_to_90: Decimal,
    #[serde(rename = "daysOver90")]
    pub days_over_90: Decimal,
    #[serde(rename = "total")]
    pub total: Decimal,
}

impl AgedBucket {
    fn zero() -> Self {
        Self {
            not_due: Decimal::ZERO,
            days_1_to_30: Decimal::ZERO,
            days_31_to_60: Decimal::ZERO,
            days_61_to_90: Decimal::ZERO,
            days_over_90: Decimal::ZERO,
            total: Decimal::ZERO,
        }
    }
}

/// Une ligne de la balance âgée : les créances ouvertes d'un contact, ventilées.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgedReceivablesRow {
    pub contact_id: i64,
    pub contact_name: String,
    #[serde(flatten)]
    pub buckets: AgedBucket,
}

/// Balance âgée complète, arrêtée à `as_of`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgedReceivables {
    pub as_of: NaiveDate,
    pub rows: Vec<AgedReceivablesRow>,
    pub totals: AgedBucket,
}

/// Ligne SQL brute (une par contact) — projetée par la requête agrégée.
#[derive(Debug, sqlx::FromRow)]
struct AgedRowSql {
    contact_id: i64,
    contact_name: String,
    not_due: Decimal,
    d1_30: Decimal,
    d31_60: Decimal,
    d61_90: Decimal,
    d90p: Decimal,
    total: Decimal,
}

/// Génère la balance âgée des créances clients arrêtée à `as_of`.
///
/// Une seule requête agrégée groupée par contact. Buckets calculés via
/// `DATEDIFF(as_of, due_date)` ; `due_date IS NULL` → « Non échu ». Montants =
/// **reste dû** dérivé (Story 25-4-b1) ; ⚠️ il ne filtre pas les règlements
/// par date : `as_of` borne les tranches, pas les pièces — la route le fixe à
/// aujourd'hui. Scoping multi-tenant obligatoire (`i.company_id = ?`).
pub async fn generate(
    pool: &MySqlPool,
    company_id: i64,
    as_of: NaiveDate,
) -> Result<AgedReceivables, ReportError> {
    // Buckets (jours de retard = as_of − due_date) :
    //   Non échu : due_date NULL OU DATEDIFF <= 0 (échéance >= as_of)
    //   1-30 / 31-60 / 61-90 : DATEDIFF dans [lo, hi]
    //   90+ : DATEDIFF >= 91 (strictement plus de 90 jours — pas de recouvrement)
    // HAVING total <> 0 écarte une facture legacy sans lignes (reste dû 0)
    // qui polluerait la liste. ⚠️ Un reste dû NÉGATIF (trop-perçu hérité)
    // n'est pas écrêté : il apparaît.
    // ⛔ Story 25-4-b1 (#416) : chaque tranche somme le RESTE DÛ, jamais le
    // TTC — une facture de 1 000.— réglée à 900.— pèse 100.—. Les tranches
    // restent assises sur `due_date` : un règlement partiel ne rajeunit pas la
    // créance restante. Forme JOINTE (`amount_due_derived_joins`) : la forme
    // scalaire serait réévaluée par ligne et par CASE.
    let due = INVOICE_AMOUNT_DUE_DERIVED_SQL;
    let sql = format!(
        "SELECT c.id AS contact_id, c.name AS contact_name, \
            COALESCE(SUM(CASE WHEN i.due_date IS NULL OR DATEDIFF(?, i.due_date) <= 0 \
                THEN {due} ELSE 0 END), CAST(0 AS DECIMAL(19,4))) AS not_due, \
            COALESCE(SUM(CASE WHEN DATEDIFF(?, i.due_date) BETWEEN 1 AND 30 \
                THEN {due} ELSE 0 END), CAST(0 AS DECIMAL(19,4))) AS d1_30, \
            COALESCE(SUM(CASE WHEN DATEDIFF(?, i.due_date) BETWEEN 31 AND 60 \
                THEN {due} ELSE 0 END), CAST(0 AS DECIMAL(19,4))) AS d31_60, \
            COALESCE(SUM(CASE WHEN DATEDIFF(?, i.due_date) BETWEEN 61 AND 90 \
                THEN {due} ELSE 0 END), CAST(0 AS DECIMAL(19,4))) AS d61_90, \
            COALESCE(SUM(CASE WHEN DATEDIFF(?, i.due_date) >= 91 \
                THEN {due} ELSE 0 END), CAST(0 AS DECIMAL(19,4))) AS d90p, \
            COALESCE(SUM({due}), CAST(0 AS DECIMAL(19,4))) AS total \
         FROM invoices i \
         INNER JOIN contacts c ON c.id = i.contact_id \
         {joins} \
         WHERE i.company_id = ? AND i.status = 'validated' AND i.paid_at IS NULL \
         GROUP BY c.id, c.name \
         HAVING total <> 0 \
         ORDER BY c.name, c.id",
        joins = amount_due_derived_joins()
    );

    let sql_rows = sqlx::query_as::<_, AgedRowSql>(&sql)
        .bind(as_of)
        .bind(as_of)
        .bind(as_of)
        .bind(as_of)
        .bind(as_of)
        .bind(company_id)
        .fetch_all(pool)
        .await
        .map_err(kesh_db::errors::map_db_error)?;

    let rows: Vec<AgedReceivablesRow> = sql_rows
        .into_iter()
        .map(|r| AgedReceivablesRow {
            contact_id: r.contact_id,
            contact_name: r.contact_name,
            buckets: AgedBucket {
                not_due: r.not_due,
                days_1_to_30: r.d1_30,
                days_31_to_60: r.d31_60,
                days_61_to_90: r.d61_90,
                days_over_90: r.d90p,
                total: r.total,
            },
        })
        .collect();

    // Totaux généraux sommés en Rust (patron balance_sheet — pas de SUM SQL global).
    let mut totals = AgedBucket::zero();
    for row in &rows {
        totals.not_due += row.buckets.not_due;
        totals.days_1_to_30 += row.buckets.days_1_to_30;
        totals.days_31_to_60 += row.buckets.days_31_to_60;
        totals.days_61_to_90 += row.buckets.days_61_to_90;
        totals.days_over_90 += row.buckets.days_over_90;
        totals.total += row.buckets.total;
    }

    Ok(AgedReceivables {
        as_of,
        rows,
        totals,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    /// Le total d'un bucket doit égaler la somme de ses tranches (réconciliation
    /// par ligne) — vérifié ici sur des données construites à la main.
    #[test]
    fn row_reconciliation_holds() {
        let b = AgedBucket {
            not_due: dec!(100),
            days_1_to_30: dec!(50),
            days_31_to_60: dec!(25),
            days_61_to_90: dec!(10),
            days_over_90: dec!(5),
            total: dec!(190),
        };
        let sum = b.not_due + b.days_1_to_30 + b.days_31_to_60 + b.days_61_to_90 + b.days_over_90;
        assert_eq!(sum, b.total);
    }

    #[test]
    fn zero_bucket_is_all_zero() {
        let z = AgedBucket::zero();
        assert_eq!(z.not_due, Decimal::ZERO);
        assert_eq!(z.total, Decimal::ZERO);
    }
}
