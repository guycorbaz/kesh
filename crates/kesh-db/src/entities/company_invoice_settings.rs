//! Entité `CompanyInvoiceSettings` (Story 5.2 — FR35).
//!
//! Relation 1-1 avec `companies` (PK = `company_id`). Row créée à la volée
//! (lazy) via `INSERT IGNORE` au premier accès, avec les DEFAULT définis
//! dans le CREATE TABLE.
//!
//! `default_receivable_account_id` et `default_revenue_account_id` NULL à
//! l'install : forcent l'Admin à les configurer avant la première
//! validation de facture. Le handler `validate` refuse (400
//! `CONFIGURATION_REQUIRED`) si l'un des deux est NULL.

use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::Journal;

/// Config facturation d'une company (Story 5.2).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CompanyInvoiceSettings {
    pub company_id: i64,
    pub invoice_number_format: String,
    pub default_receivable_account_id: Option<i64>,
    pub default_revenue_account_id: Option<i64>,
    /// Compte TVA due (collectée sur ventes), type Liability. Story 18-1a.
    pub default_vat_payable_account_id: Option<i64>,
    /// Compte impôt préalable (TVA récupérable sur achats), type Asset. Story 18-1a.
    pub default_vat_recoverable_account_id: Option<i64>,
    /// Compte de décompte TVA (solde net dû à l'AFC), type Liability. Story 18-1a.
    pub default_vat_decompte_account_id: Option<i64>,
    pub default_sales_journal: Journal,
    pub journal_entry_description_template: String,
    /// Format du numéro d'avoir (note de crédit). Story 12.1. Défaut `AV-{YEAR}-{SEQ:04}`.
    pub credit_note_number_format: String,
    /// Compte créanciers (2000) par défaut, contrepartie de l'écriture d'achat
    /// des factures fournisseurs, type Liability. Story 12.2.
    pub default_payable_account_id: Option<i64>,
    /// Compte de **différences d'arrondi** — charge ou produit, actif et
    /// imputable — qui reçoit l'écart d'un demi-centime au plus qu'un paiement
    /// arrondi au centime laisse sur une facture (Story 25-4-c3-a1, #476).
    /// Lu seulement quand un écart se présente — mais l'arrondi à 5 centimes des
    /// pièces émises (Story 25-4-c4-a), actif par défaut, en produit un sur la
    /// plupart des factures : sans ce compte, leur validation est refusée.
    /// Un réglage et non un rôle de compte (arbitrage du 2026-09-30).
    pub default_rounding_account_id: Option<i64>,
    /// Arrondir à 5 centimes le total des pièces émises (Story 25-4-c4-a, #494).
    /// Actif par défaut ; lu à la validation, qui fige l'écart sur la pièce. Son
    /// API et son écran viennent avec la Story 25-4-c4-b.
    pub round_to_5_centimes: bool,
    /// Montant minimum d'une facture émise (Story 25-4-e, #495) : une facture dont
    /// le total TTC arrondi lui est inférieur ne se valide pas. `None` = aucun seuil.
    pub minimum_invoice_amount: Option<Decimal>,
    /// Comptes des **natures d'écart soldé** (Story 25-4-d1, #384) : ce qui reste
    /// dû sur une facture et que le comptable abandonne s'impute au compte de sa
    /// nature — escompte accordé, frais bancaires retenus par la banque du
    /// client, perte sur débiteur. Charge ou produit, imputable. Désignés
    /// d'office depuis le marqueur `writeOffNature` du plan à la création de la
    /// société ; `None` si le plan n'en marque pas (l'escompte des associations).
    pub default_discount_account_id: Option<i64>,
    pub default_bank_fees_account_id: Option<i64>,
    pub default_bad_debt_account_id: Option<i64>,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Données de mise à jour (PUT /company/invoice-settings). Tous les
/// champs sont requis (remplacement intégral). `version` est géré
/// séparément par le repository (verrou optimiste, pattern contacts/products).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyInvoiceSettingsUpdate {
    pub invoice_number_format: String,
    pub default_receivable_account_id: Option<i64>,
    pub default_revenue_account_id: Option<i64>,
    pub default_vat_payable_account_id: Option<i64>,
    pub default_vat_recoverable_account_id: Option<i64>,
    pub default_vat_decompte_account_id: Option<i64>,
    pub default_sales_journal: Journal,
    pub journal_entry_description_template: String,
    pub credit_note_number_format: String,
    pub default_payable_account_id: Option<i64>,
    /// Story 25-4-c3-a1 — absent du corps, il vaut `None` (comme les comptes
    /// TVA) : le remplacement est intégral.
    pub default_rounding_account_id: Option<i64>,
    /// Story 25-4-c4-b (#494) — la valeur **résolue** (la route préserve un champ
    /// absent du corps de la requête).
    pub round_to_5_centimes: bool,
    /// Story 25-4-e (#495) — la valeur résolue (la route préserve l'absent).
    pub minimum_invoice_amount: Option<Decimal>,
    /// Story 25-4-d1 (#384) — les valeurs résolues (la route préserve l'absent).
    pub default_discount_account_id: Option<i64>,
    pub default_bank_fees_account_id: Option<i64>,
    pub default_bad_debt_account_id: Option<i64>,
}
