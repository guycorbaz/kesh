//! Routes CRUD pour le catalogue produits/services (Story 4.2).
//!
//! **Security Note (Story 6.2):** All handlers scope by `current_user.company_id` from JWT.
//! The company_id in JWT can become stale if a user is reassigned to a different company
//! during an active session. See `middleware/auth.rs` for staleness window (proportional to
//! `KESH_JWT_EXPIRY_MINUTES`, default 15 min).

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

use kesh_core::listing::SortDirection;
use kesh_db::entities::product::{NewProduct, Product, ProductUpdate};
use kesh_db::errors::DbError;
use kesh_db::repositories::products::{self, ProductListQuery, ProductSortBy};

use crate::AppState;
use crate::errors::AppError;
use crate::helpers::get_company_for;
use crate::middleware::auth::CurrentUser;
use crate::routes::ListResponse;
use crate::routes::limits::{MAX_DECIMAL_SCALE, MAX_UNIT_PRICE, scale_within};
use crate::routes::vat;

// ---------------------------------------------------------------------------
// Limites
// ---------------------------------------------------------------------------

const MAX_NAME_LEN: usize = 255;
const MAX_DESCRIPTION_LEN: usize = 1000;
const MAX_LIST_LIMIT: i64 = 100;
const DEFAULT_LIST_LIMIT: i64 = 20;
/// Plafond anti-DoS pour le paramètre `search` avant `escape_like` + LIKE SQL.
const MAX_SEARCH_LEN: usize = 100;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListProductsQuery {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub include_archived: bool,
    #[serde(default)]
    pub sort_by: Option<ProductSortBy>,
    #[serde(default)]
    pub sort_direction: Option<SortDirection>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProductRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
    pub version: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveProductRequest {
    pub version: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductResponse {
    pub id: i64,
    pub company_id: i64,
    pub name: String,
    pub description: Option<String>,
    /// Sérialisé en string via la feature `serde-str` de `rust_decimal`.
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
    pub active: bool,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<Product> for ProductResponse {
    fn from(p: Product) -> Self {
        Self {
            id: p.id,
            company_id: p.company_id,
            name: p.name,
            description: p.description,
            unit_price: p.unit_price,
            vat_rate: p.vat_rate,
            active: p.active,
            version: p.version,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn normalize_optional(s: Option<String>) -> Option<String> {
    s.and_then(|v| {
        let t = v.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    })
}

#[derive(Debug)]
struct ValidatedFields {
    name: String,
    description: Option<String>,
    unit_price: Decimal,
    vat_rate: Decimal,
}

fn validate_common(
    name: String,
    description: Option<String>,
    unit_price: Decimal,
    vat_rate: Decimal,
) -> Result<ValidatedFields, AppError> {
    // Normalisation NFC pour éviter les collisions `UNIQUE(company_id, name)` entre
    // formes composées (NFC) et décomposées (NFD) du même glyphe (ex: "Café" vs "Café").
    let trimmed_name: String = name.trim().nfc().collect();
    if trimmed_name.is_empty() {
        return Err(AppError::Validation("Le nom est obligatoire".into()));
    }
    if trimmed_name.chars().count() > MAX_NAME_LEN {
        return Err(AppError::Validation(format!(
            "Le nom doit faire au plus {MAX_NAME_LEN} caractères"
        )));
    }

    // Normalisation NFC aussi sur `description` pour rester cohérent avec `name`
    // (recherche LIKE comparant des représentations Unicode homogènes).
    let description = normalize_optional(description).map(|d| d.nfc().collect::<String>());
    if let Some(ref d) = description {
        if d.chars().count() > MAX_DESCRIPTION_LEN {
            return Err(AppError::Validation(format!(
                "La description doit faire au plus {MAX_DESCRIPTION_LEN} caractères"
            )));
        }
    }

    if unit_price < Decimal::ZERO {
        return Err(AppError::Validation(
            "Le prix unitaire doit être positif ou nul".into(),
        ));
    }
    // Scale ≤ 4 : empêche toute truncation silencieuse MariaDB sur DECIMAL(19,4).
    if !scale_within(&unit_price, MAX_DECIMAL_SCALE) {
        return Err(AppError::Validation(format!(
            "Le prix unitaire doit avoir au plus {MAX_DECIMAL_SCALE} décimales"
        )));
    }
    // Plafond 1 milliard CHF : anti-overflow Epic 5 (ligne facture = prix × qty).
    if unit_price > *MAX_UNIT_PRICE {
        return Err(AppError::Validation(
            "Le prix unitaire doit être inférieur ou égal à 1 000 000 000".into(),
        ));
    }

    Ok(ValidatedFields {
        name: trimmed_name,
        description,
        unit_price,
        vat_rate,
    })
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// Story 6.2: Scoped by current_user.company_id.
pub async fn list_products(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<ListProductsQuery>,
) -> Result<Json<ListResponse<ProductResponse>>, AppError> {
    // Validate company exists (defensive: company_id staleness window)
    let _ = get_company_for(&current_user, &state.pool).await?;

    let limit = params.limit.unwrap_or(DEFAULT_LIST_LIMIT);
    if !(1..=MAX_LIST_LIMIT).contains(&limit) {
        return Err(AppError::Validation(format!(
            "limit doit être compris entre 1 et {MAX_LIST_LIMIT}"
        )));
    }
    let offset = params.offset.unwrap_or(0);
    if offset < 0 {
        return Err(AppError::Validation(
            "offset doit être positif ou nul".into(),
        ));
    }

    // Anti-DoS : plafonner la longueur du filtre LIKE (après trim, cohérent
    // avec le comportement du repository qui trim avant d'exécuter le LIKE).
    if let Some(ref s) = params.search {
        if s.trim().chars().count() > MAX_SEARCH_LEN {
            return Err(AppError::Validation(format!(
                "search doit faire au plus {MAX_SEARCH_LEN} caractères"
            )));
        }
    }

    let query = ProductListQuery {
        search: params.search,
        include_archived: params.include_archived,
        sort_by: params.sort_by.unwrap_or_default(),
        sort_direction: params.sort_direction.unwrap_or(SortDirection::Asc),
        limit,
        offset,
    };

    let result =
        products::list_by_company_paginated(&state.pool, current_user.company_id, query).await?;

    Ok(Json(ListResponse {
        items: result
            .items
            .into_iter()
            .map(ProductResponse::from)
            .collect(),
        total: result.total,
        limit: result.limit,
        offset: result.offset,
    }))
}

/// Story 6.2: Scoped by current_user.company_id.
pub async fn get_product(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<ProductResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let product = products::find_by_id(&state.pool, company.id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;
    Ok(Json(ProductResponse::from(product)))
}

/// Story 6.2: Scoped by current_user.company_id.
pub async fn create_product(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateProductRequest>,
) -> Result<(StatusCode, Json<ProductResponse>), AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;

    let v = validate_common(req.name, req.description, req.unit_price, req.vat_rate)?;
    vat::verify_vat_rates_against_db(&state.pool, current_user.company_id, &[v.vat_rate]).await?;

    let new = NewProduct {
        company_id: company.id,
        name: v.name,
        description: v.description,
        unit_price: v.unit_price,
        vat_rate: v.vat_rate,
    };

    let product = products::create(&state.pool, current_user.user_id, new).await?;
    Ok((StatusCode::CREATED, Json(ProductResponse::from(product))))
}

pub async fn update_product(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateProductRequest>,
) -> Result<Json<ProductResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let v = validate_common(req.name, req.description, req.unit_price, req.vat_rate)?;
    vat::verify_vat_rates_against_db(&state.pool, current_user.company_id, &[v.vat_rate]).await?;

    let changes = ProductUpdate {
        name: v.name,
        description: v.description,
        unit_price: v.unit_price,
        vat_rate: v.vat_rate,
    };

    let product = products::update(
        &state.pool,
        company.id,
        id,
        req.version,
        current_user.user_id,
        changes,
    )
    .await?;
    Ok(Json(ProductResponse::from(product)))
}

pub async fn archive_product(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<ArchiveProductRequest>,
) -> Result<Json<ProductResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let product = products::archive(
        &state.pool,
        company.id,
        id,
        req.version,
        current_user.user_id,
    )
    .await?;
    Ok(Json(ProductResponse::from(product)))
}

// ---------------------------------------------------------------------------
// Tests unitaires
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    // Story 7.2 : `validate_common` ne valide plus le `vat_rate` (shape-check
    // sync vs VAT-check async DB-driven séparés). Le shape-check passe pour
    // n'importe quel `Decimal` ; le rejet d'un rate inconnu se fait dans le
    // handler via `vat::verify_vat_rates_against_db` (couvert par les E2E
    // products_e2e.rs et la suite vat_rates_e2e.rs).
    #[test]
    fn validate_accepts_any_vat_rate_shape() {
        for d in [dec!(0.00), dec!(2.60), dec!(3.80), dec!(8.10), dec!(7.70)] {
            let r = validate_common("Logo".into(), None, dec!(100), d);
            assert!(
                r.is_ok(),
                "shape-check ne doit pas filtrer le rate ({d}) — délégué à la couche DB"
            );
        }
    }

    #[test]
    fn validate_rejects_empty_name() {
        let err = validate_common("   ".into(), None, dec!(100), dec!(8.10)).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_trims_name() {
        let v = validate_common("  Logo  ".into(), None, dec!(100), dec!(8.10)).unwrap();
        assert_eq!(v.name, "Logo");
    }

    #[test]
    fn validate_rejects_name_too_long() {
        let long = "a".repeat(256);
        let err = validate_common(long, None, dec!(100), dec!(8.10)).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_rejects_description_too_long() {
        let long = "a".repeat(1001);
        let err = validate_common("Logo".into(), Some(long), dec!(100), dec!(8.10)).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_rejects_negative_price() {
        let err = validate_common("Logo".into(), None, dec!(-0.01), dec!(8.10)).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_accepts_zero_price() {
        // Produit offert : prix 0 OK.
        let v = validate_common("Logo".into(), None, dec!(0), dec!(8.10)).unwrap();
        assert_eq!(v.unit_price, dec!(0));
    }

    #[test]
    fn validate_rejects_unit_price_scale_above_4() {
        let err = validate_common("Logo".into(), None, dec!(100.12345), dec!(8.10)).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_accepts_unit_price_scale_up_to_4() {
        let v = validate_common("Logo".into(), None, dec!(100.1234), dec!(8.10)).unwrap();
        assert_eq!(v.unit_price, dec!(100.1234));
    }

    #[test]
    fn validate_rejects_unit_price_above_one_billion() {
        let err =
            validate_common("Logo".into(), None, dec!(1000000000.0001), dec!(8.10)).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_accepts_unit_price_at_cap() {
        let v = validate_common("Logo".into(), None, dec!(1000000000), dec!(8.10)).unwrap();
        assert_eq!(v.unit_price, dec!(1000000000));
    }

    #[test]
    fn validate_normalizes_name_and_description_to_nfc() {
        // "é" décomposé (NFD) : U+0065 U+0301 = "e\u{0301}"
        let nfd_name = "Caf\u{0065}\u{0301}";
        let nfd_desc = "logo Caf\u{0065}\u{0301}";
        let v = validate_common(
            nfd_name.into(),
            Some(nfd_desc.into()),
            dec!(100),
            dec!(8.10),
        )
        .unwrap();
        // "é" composé (NFC) : U+00E9
        let nfc = "Caf\u{00E9}";
        assert_eq!(v.name, nfc);
        // La description est également normalisée en NFC.
        let desc = v.description.unwrap();
        assert!(desc.contains(nfc), "description should be NFC-normalized");
        assert!(
            !desc.contains('\u{0301}'),
            "description should not contain NFD combining mark"
        );
    }

    #[test]
    fn normalize_optional_trims_and_collapses_empty_to_none() {
        assert_eq!(
            normalize_optional(Some("  desc  ".into())),
            Some("desc".into())
        );
        assert_eq!(normalize_optional(Some("   ".into())), None);
        assert_eq!(normalize_optional(None), None);
    }
}
