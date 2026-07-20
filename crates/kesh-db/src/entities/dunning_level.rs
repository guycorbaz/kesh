//! Niveau de rappel débiteur (dunning) — collection company-scoped.
//!
//! Calqué sur `vat_rate` : `FromRow` sans `Serialize` (anti-fuite `company_id` au
//! client ; l'exposition REST passe par `DunningLevelResponse`). Niveaux NUMÉROTÉS
//! CONTIGUS (`level_number` 1-based, unique par company) — hard-delete + renumérotation
//! côté repo (l'historique est protégé par les snapshots `invoice_reminders`, 21-5a).

use chrono::NaiveDateTime;
use rust_decimal::Decimal;

/// Niveau de rappel persisté (`dunning_levels`).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DunningLevel {
    pub id: i64,
    pub company_id: i64,
    /// Numéro de niveau 1-based, contigu et unique par company.
    pub level_number: i16,
    /// Délai en jours depuis l'étape précédente (échéance+grâce pour le niveau 1,
    /// rappel N-1 ensuite).
    pub delay_days: i32,
    /// Frais de rappel en CHF, borné 0..10'000 (CHECK DB).
    pub fee_amount: Decimal,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Payload de création (le `level_number` est posé par le repo = MAX+1 ;
/// `version` posé à 0).
#[derive(Debug, Clone)]
pub struct NewDunningLevel {
    pub company_id: i64,
    pub delay_days: i32,
    pub fee_amount: Decimal,
}

/// Payload de modification. `level_number` est **immutable** (pas de
/// réordonnancement en v1 ; create append / delete renumber seulement).
#[derive(Debug, Clone)]
pub struct UpdateDunningLevel {
    pub delay_days: i32,
    pub fee_amount: Decimal,
}
