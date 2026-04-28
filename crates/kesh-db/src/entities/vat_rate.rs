//! Entité `VatRate` : taux TVA configurable par tenant (Story 7.2 — KF-003 closure).
//!
//! Remplace l'ancienne whitelist hardcodée (`ALLOWED_VAT_RATES` dans
//! `kesh-api/src/routes/vat.rs`). La table est **read-only v0.1** : seul le
//! seed (migration backfill + onboarding + `seed_demo`) écrit. Epic 11-1
//! introduira le CRUD admin et la colonne `version`.
//!
//! - `rate` en `DECIMAL(5,2)` (aligné `products.vat_rate`) → `rust_decimal::Decimal`.
//! - `label` = clé i18n (ex. `product-vat-normal`), résolue côté frontend.
//! - `valid_from` inclusif, `valid_to` exclusif (NULL = pas d'expiration).

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;

/// Taux TVA persisté en base, scopé `company_id`.
///
/// **Pas de dérivation `Serialize`** (Pass 1 remediation #17) : si un futur
/// handler retourne `Json(rates)` au lieu de la projection
/// `routes/vat::VatRateResponse`, `companyId` fuiterait au client. Toute
/// exposition REST passe obligatoirement par `VatRateResponse`.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct VatRate {
    pub id: i64,
    pub company_id: i64,
    pub label: String,
    pub rate: Decimal,
    pub valid_from: NaiveDate,
    pub valid_to: Option<NaiveDate>,
    pub active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Données de création d'un taux TVA. Réservé Epic 11-1 (CRUD admin) ;
/// v0.1 utilisé uniquement par les helpers seed.
#[derive(Debug, Clone)]
pub struct NewVatRate {
    pub company_id: i64,
    pub label: String,
    pub rate: Decimal,
    pub valid_from: NaiveDate,
    pub valid_to: Option<NaiveDate>,
}
