//! Entités `Invoice` et `InvoiceLine` (Story 5.1 — FR31, FR32).
//!
//! Les lignes snapshotent `description`, `unit_price`, `vat_rate` au moment
//! de la création : modifier un produit catalogue ne doit PAS altérer une
//! facture existante. Le catalogue n'est qu'un accélérateur de saisie.
//!
//! `total_amount` est recalculé et persisté par le repository à chaque
//! mutation (source de vérité = lignes).

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Facture persistée (entête).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Invoice {
    pub id: i64,
    pub company_id: i64,
    pub contact_id: i64,
    pub invoice_number: Option<String>,
    pub status: String,
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub payment_terms: Option<String>,
    pub total_amount: Decimal,
    /// Référence vers l'écriture comptable générée à la validation (Story 5.2).
    /// NULL tant que la facture est en brouillon.
    pub journal_entry_id: Option<i64>,
    /// Horodate de paiement manuel (Story 5.4). NULL = impayée.
    /// Ne peut être posée que sur `status = 'validated'` (CHECK DB).
    pub paid_at: Option<NaiveDateTime>,
    /// Horodate du dernier envoi par e-mail (Story 20-3b1). NULL = jamais
    /// envoyée. Le renvoi écrase la valeur (chaque envoi est audité).
    pub emailed_at: Option<NaiveDateTime>,
    /// Destinataire du dernier envoi (snapshot de `contacts.email` au moment
    /// de l'envoi, Story 20-3b1).
    pub emailed_to: Option<String>,
    /// Projet analytique document-level (Epic 19, Story 19-4). Propagé sur
    /// toutes les lignes de l'écriture de vente à la validation, hérité par
    /// la contre-passation d'avoir. `None` = pas de projet.
    pub project_id: Option<i64>,
    /// Suspension des rappels débiteurs (Story 21-5a). NULL = non suspendue.
    /// Une facture suspendue sort de la liste « à rappeler » mais reste dans la
    /// balance âgée et l'échéancier (invariant anti-dissimulation, item 10 epic-21).
    pub dunning_paused_at: Option<NaiveDateTime>,
    /// Note optionnelle accompagnant la suspension (posée à la pause, remise à
    /// NULL à la reprise).
    pub dunning_paused_note: Option<String>,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Ligne de facture persistée.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceLine {
    pub id: i64,
    pub invoice_id: i64,
    pub position: i32,
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
    pub line_total: Decimal,
    /// Compte de produit de la ligne (Story 16-1a, #152 / CR #265).
    ///
    /// `None` sur un **brouillon** signifie « utiliser le compte de produit
    /// par défaut de la société au moment de la validation » (liaison tardive,
    /// décision D2) — on ne fige pas le défaut à la création, sinon le
    /// brouillon suivrait une configuration périmée.
    ///
    /// À la **validation**, `validate_invoice` matérialise le compte effectif
    /// ici : une facture validée par ce binaire ne porte donc plus `None`. Les
    /// factures validées **avant** le déploiement de 16-1a le conservent — leur
    /// traitement relève de la Story 16-1a-bis.
    pub revenue_account_id: Option<i64>,
    pub created_at: NaiveDateTime,
}

/// Données de création d'une ligne (sans `position` ni `line_total` —
/// calculés par le repository).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewInvoiceLine {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
    /// Compte de produit choisi pour cette ligne. `None` = repli sur le compte
    /// de produit par défaut de la société (Story 16-1a, D1/D2).
    pub revenue_account_id: Option<i64>,
}

/// Données de création d'une facture. Le caller a déjà validé/normalisé.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewInvoice {
    pub company_id: i64,
    pub contact_id: i64,
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub payment_terms: Option<String>,
    /// Projet analytique document-level (Epic 19, Story 19-4). Validé à la
    /// création (projet de la company, non archivé).
    #[serde(default)]
    pub project_id: Option<i64>,
    pub lines: Vec<NewInvoiceLine>,
}

/// Données de modification d'une facture brouillon.
///
/// `version` est passée séparément au repository (pattern identique à
/// `products::update`). Les lignes remplacent entièrement les anciennes
/// (replace-all — voir Dev Notes).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceUpdate {
    pub contact_id: i64,
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub payment_terms: Option<String>,
    /// Projet analytique (Story 19-4). Validé seulement si la valeur change
    /// (grandfathering du tag inchangé — un projet archivé après la pose du
    /// tag ne bloque pas l'édition des autres champs du brouillon).
    #[serde(default)]
    pub project_id: Option<i64>,
    pub lines: Vec<NewInvoiceLine>,
}
