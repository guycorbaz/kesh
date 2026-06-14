//! Entité `VatRate` : taux TVA configurable par tenant (Story 7.2 — KF-003 closure).
//!
//! Remplace l'ancienne whitelist hardcodée (`ALLOWED_VAT_RATES` dans
//! `kesh-api/src/routes/vat.rs`). Story 11-1 a introduit le CRUD admin
//! (colonnes `version` + `category`).
//!
//! - `rate` en `DECIMAL(5,2)` (aligné `products.vat_rate`) → `rust_decimal::Decimal`.
//! - `label` = libellé d'affichage (clé i18n pour les taux seedés, ex.
//!   `product-vat-normal` ; texte libre optionnel pour un taux admin-créé),
//!   résolu côté frontend via `i18nMsg(label, fallback)`.
//! - `category` = discriminant métier **stable et extensible** (clé), distinct
//!   du `label` : permet de suivre « le taux normal » au fil des années. Clés
//!   réservées connues `normal`/`reduced`/`special`/`exempt` ; `custom` = libre ;
//!   toute autre valeur (catégorie officielle future) est acceptée — modèle
//!   extensible, pas de liste fermée. Typé `String` (et non un `enum` Rust) pour
//!   garantir qu'une catégorie inconnue ne casse jamais le décodage.
//! - `valid_from` inclusif, `valid_to` exclusif (NULL = pas d'expiration).
//! - `version` = verrou optimiste (incrémenté à chaque UPDATE).

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
    pub category: String,
    pub label: String,
    pub rate: Decimal,
    pub valid_from: NaiveDate,
    pub valid_to: Option<NaiveDate>,
    pub active: bool,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Clés de catégorie réservées (connues de l'application pour l'affichage i18n).
/// Le modèle reste **extensible** : `category` accepte toute valeur non vide,
/// ces constantes ne sont qu'un référentiel des clés standard suisses.
pub mod vat_category {
    pub const NORMAL: &str = "normal";
    pub const REDUCED: &str = "reduced";
    pub const SPECIAL: &str = "special";
    pub const EXEMPT: &str = "exempt";
    pub const CUSTOM: &str = "custom";
}

/// Données de création d'un taux TVA (CRUD admin Story 11-1 ; utilisé aussi
/// par les helpers seed via les valeurs littérales).
#[derive(Debug, Clone)]
pub struct NewVatRate {
    pub company_id: i64,
    pub category: String,
    pub label: String,
    pub rate: Decimal,
    pub valid_from: NaiveDate,
    pub valid_to: Option<NaiveDate>,
}

/// Champs modifiables d'un taux TVA existant (Story 11-1).
///
/// Le champ `rate` n'est **pas** modifiable (un taux a une valeur figée ;
/// changer de taux = créer un nouveau taux). La `category` n'est pas non plus
/// modifiable (un taux appartient à une série de catégorie).
#[derive(Debug, Clone)]
pub struct UpdateVatRate {
    pub label: String,
    pub valid_to: Option<NaiveDate>,
    pub active: bool,
}
