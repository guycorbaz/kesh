//! Validation des taux TVA — DB-driven (Story 7.2 — KF-003 closure).
//!
//! Remplace l'ancienne whitelist hardcodée (`ALLOWED_VAT_RATES: LazyLock<[Decimal; 4]>`)
//! par une consultation directe de la table `vat_rates` scopée par tenant.
//!
//! **Pattern shape-check (sync) / VAT-check (async)** :
//! - `validate_line` (`invoices.rs`) et `validate_common` (`products.rs`) restent
//!   **sync** et ne valident plus le `vat_rate` — uniquement la forme (non-empty,
//!   ranges, scale).
//! - Les handlers appellent ensuite `verify_vat_rates_against_db(...)` qui
//!   déduplique les rates via `BTreeSet<&Decimal>` et fait 1 SELECT par rate
//!   distinct (typiquement 1-2 sur factures réelles, max 4-10).
//!
//! **Multi-tenant** : `find_active_by_rate` exige `company_id` ; aucune fuite
//! cross-tenant possible.

use std::collections::BTreeSet;

use axum::extract::State;
use axum::{Extension, Json};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::mysql::MySqlPool;

use kesh_db::entities::VatRate;
use kesh_db::errors::{DbError, map_db_error};
use kesh_db::repositories::vat_rates;

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;

/// Message d'erreur unique partagé entre `products` et `invoices` quand un
/// taux TVA n'est pas autorisé pour la company de l'utilisateur courant.
///
/// Pas de liste hardcodée dans le message — la liste à jour est
/// consultable via `GET /api/v1/vat-rates`.
pub const VAT_REJECTED_MSG: &str = "Taux TVA non autorisé pour cette entreprise.";

/// Vérifie qu'un taux TVA donné est actif pour la company courante.
///
/// Délègue au repo `vat_rates::find_active_by_rate`. Scale-invariant
/// (`Decimal::eq` ignore le scale, `DECIMAL(5,2)` aussi côté MariaDB).
///
/// Erreurs DB transitives bubble up (le caller mappe via `?` ou
/// `AppError::Database`).
pub async fn validate_vat_rate(
    pool: &MySqlPool,
    company_id: i64,
    rate: &Decimal,
) -> Result<bool, DbError> {
    vat_rates::find_active_by_rate(pool, company_id, rate)
        .await
        .map(|opt| opt.is_some())
}

/// Vérifie qu'un ensemble de taux TVA sont tous autorisés pour la company.
///
/// Déduplique via `BTreeSet<&Decimal>` (`Decimal::cmp` ignore le scale en
/// `rust_decimal` ≥ 1.30 — projet en 1.41, OK) puis émet **une seule**
/// requête `SELECT ... WHERE rate IN (?, ?, ...)`. Évite l'amplification
/// O(N) qui transformerait MAX_LINES (200) lignes distinctes en 200
/// round-trips DB.
///
/// Si tous les rates sont valides → `Ok(())`. Sinon → `AppError::Validation`
/// avec `VAT_REJECTED_MSG`.
pub async fn verify_vat_rates_against_db(
    pool: &MySqlPool,
    company_id: i64,
    rates: &[Decimal],
) -> Result<(), AppError> {
    let unique: BTreeSet<&Decimal> = rates.iter().collect();
    if unique.is_empty() {
        return Ok(());
    }

    // SELECT COUNT(DISTINCT rate) WHERE rate IN (?, ?, ...) — un seul
    // round-trip DB quel que soit le nombre de rates distincts. Les
    // placeholders `?` sont liés via `.bind()` (pas d'interpolation).
    let placeholders = std::iter::repeat("?")
        .take(unique.len())
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT COUNT(DISTINCT rate) FROM vat_rates \
         WHERE company_id = ? AND active = TRUE AND rate IN ({placeholders})",
    );

    let mut query = sqlx::query_scalar::<_, i64>(&sql).bind(company_id);
    for rate in &unique {
        query = query.bind(*rate);
    }
    let matched = query.fetch_one(pool).await.map_err(map_db_error)?;

    if matched as usize == unique.len() {
        Ok(())
    } else {
        Err(AppError::Validation(VAT_REJECTED_MSG.into()))
    }
}

// ---------------------------------------------------------------------------
// DTO + handler GET /api/v1/vat-rates (Story 7.2 T4)
// ---------------------------------------------------------------------------

/// Réponse REST pour `GET /api/v1/vat-rates`.
///
/// `rate` est sérialisé en string décimale via la feature `serde-str` de
/// `rust_decimal` (activée par défaut dans `kesh-db/Cargo.toml`).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VatRateResponse {
    pub id: i64,
    pub label: String,
    pub rate: Decimal,
    pub valid_from: NaiveDate,
    pub valid_to: Option<NaiveDate>,
    pub active: bool,
}

impl From<VatRate> for VatRateResponse {
    fn from(v: VatRate) -> Self {
        Self {
            id: v.id,
            label: v.label,
            rate: v.rate,
            valid_from: v.valid_from,
            valid_to: v.valid_to,
            active: v.active,
        }
    }
}

/// `GET /api/v1/vat-rates` — liste les taux TVA actifs pour la company
/// de l'utilisateur courant.
///
/// - Auth : tout rôle authentifié (Consultation incluse — lecture pure).
/// - Multi-tenant : `current_user.company_id` est l'unique source de scope ;
///   aucun query param `companyId` (Anti-Pattern 4 Story 7-1).
/// - Réponse : array direct triée `rate DESC` (pas de wrapper `ListResponse` :
///   liste minuscule, pas de pagination).
pub async fn list_vat_rates(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<VatRateResponse>>, AppError> {
    let rates = vat_rates::list_active_for_company(&state.pool, current_user.company_id).await?;
    Ok(Json(rates.into_iter().map(VatRateResponse::from).collect()))
}
