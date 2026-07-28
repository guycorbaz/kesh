//! Entités `CreditNote` et `CreditNoteLine` (Story 12.1 — FR36, FR37).
//!
//! Un avoir (note de crédit) annule une facture validée par contre-passation.
//! Modèle v0.2 : avoir TOTAL en single-step (DC3/DC5) — créer un avoir le crée
//! et l'émet immédiatement (`status = 'issued'`), génère l'écriture de
//! contre-passation et bascule la facture d'origine en `cancelled`.
//!
//! Les lignes snapshotent `description`, `quantity`, `unit_price`, `vat_rate`
//! depuis la facture d'origine au moment de la création — figées comme les
//! lignes de facture (le `vat_rate` n'est jamais relu depuis `vat_rates`).
//!
//! `total_amount` est le HT (Σ `line_total`), miroir strict de
//! `invoices.total_amount`.

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Avoir persisté (entête).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CreditNote {
    pub id: i64,
    pub company_id: i64,
    pub contact_id: i64,
    /// Facture d'origine annulée par cet avoir (FK RESTRICT, UNIQUE).
    pub invoice_id: i64,
    /// Numéro séquentiel (ex. `AV-2026-0001`). NULL seulement en `draft`
    /// (inutilisé en v0.2 — un avoir est créé directement `issued`).
    pub credit_note_number: Option<String>,
    pub status: String,
    pub date: NaiveDate,
    /// HT (Σ line_total), miroir de `invoices.total_amount`.
    pub total_amount: Decimal,
    /// Référence vers l'écriture de contre-passation. NULL seulement en `draft`.
    pub journal_entry_id: Option<i64>,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Ligne d'avoir persistée (snapshot d'une ligne de facture).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CreditNoteLine {
    pub id: i64,
    pub credit_note_id: i64,
    pub position: i32,
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
    pub line_total: Decimal,
    /// Compte de produit **recopié depuis la ligne de facture** à la création
    /// de l'avoir (Story 16-1a, décision D5).
    ///
    /// Porté par le snapshot plutôt que relu depuis la facture d'origine pour
    /// que l'avoir reste auto-descriptif, comme toutes les autres colonnes de
    /// `credit_note_lines`. C'est ce qui garantit que la contre-passation
    /// débite **le compte effectivement crédité par la facture**, et non le
    /// compte de produit par défaut tel qu'il se trouve être configuré le jour
    /// de l'avoir — sans quoi les deux écritures ne s'annulent plus et
    /// laissent un résidu permanent, invisible au bilan mais faux au compte de
    /// résultat.
    ///
    /// `None` seulement pour un avoir émis sur une facture validée **avant**
    /// 16-1a et non traitée par 16-1a-bis : le repli sur le défaut société
    /// s'applique alors, exactement comme avant la story.
    pub revenue_account_id: Option<i64>,
    pub created_at: NaiveDateTime,
}

/// Données de création d'un avoir. Les lignes sont snapshotées côté repository
/// depuis la facture `invoice_id` (DC9) — le caller ne fournit pas les lignes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewCreditNote {
    pub company_id: i64,
    pub invoice_id: i64,
    /// Date de l'avoir (détermine l'exercice de la contre-passation, DC10).
    pub date: NaiveDate,
}
