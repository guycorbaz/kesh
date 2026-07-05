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

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlPool;

use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::{NewVatRate, UpdateVatRate, VatRate};
use kesh_db::errors::{DbError, map_db_error};
use kesh_db::repositories::{audit_log, bank_accounts, onboarding, vat_rates};

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;
use crate::routes::limits::scale_within;

/// Query params de `GET /api/v1/vat-rates`. `?history=true` → liste complète
/// (actifs + inactifs/expirés) pour la vue admin ; défaut = actifs seulement.
#[derive(Debug, Deserialize)]
pub struct ListVatRatesQuery {
    pub history: Option<bool>,
}

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
    let placeholders = std::iter::repeat_n("?", unique.len())
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
    /// Catégorie métier (clé extensible : normal/reduced/special/exempt/custom
    /// ou catégorie officielle future) — Story 11-1.
    pub category: String,
    pub label: String,
    pub rate: Decimal,
    pub valid_from: NaiveDate,
    pub valid_to: Option<NaiveDate>,
    pub active: bool,
    /// Verrou optimiste — le client doit le renvoyer au `PUT`/`DELETE`.
    pub version: i32,
}

impl From<VatRate> for VatRateResponse {
    fn from(v: VatRate) -> Self {
        Self {
            id: v.id,
            category: v.category,
            label: v.label,
            rate: v.rate,
            valid_from: v.valid_from,
            valid_to: v.valid_to,
            active: v.active,
            version: v.version,
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
    Query(query): Query<ListVatRatesQuery>,
) -> Result<Json<Vec<VatRateResponse>>, AppError> {
    let rates = if query.history.unwrap_or(false) {
        // Vue historique admin : actifs + inactifs/expirés, ré-ordonnés
        // `valid_from DESC, rate ASC` pour l'affichage (la fn repo trie `id ASC`).
        let mut all = vat_rates::list_all_by_company(&state.pool, current_user.company_id).await?;
        all.sort_by(|a, b| {
            b.valid_from
                .cmp(&a.valid_from)
                .then_with(|| a.rate.cmp(&b.rate))
        });
        all
    } else {
        // Comportement v0.1 préservé (non-régression facturation/produits).
        vat_rates::list_active_for_company(&state.pool, current_user.company_id).await?
    };
    Ok(Json(rates.into_iter().map(VatRateResponse::from).collect()))
}

// ===========================================================================
// CRUD admin (Story 11-1) — POST / PUT / DELETE
//
// RBAC : ces handlers sont montés dans `admin_routes` (rôle Administrateur,
// FR54) — le guard est appliqué au niveau du routeur, pas dans le handler.
// Audit émis dans la même transaction que la mutation. Sentinel lock company
// acquis en début de tx (invariant de non-chevauchement par catégorie =
// contrainte cross-row).
// ===========================================================================

/// Longueurs max alignées sur le schéma (`category VARCHAR(32)`, `label VARCHAR(64)`).
const CATEGORY_MAX_LEN: usize = 32;
const LABEL_MAX_LEN: usize = 64;

/// Garde : les mutations TVA exigent l'onboarding complété (step ≥ 7) ou le
/// mode demo. Dupliqué de `bank_accounts` (helper trivial — éviter un couplage
/// cross-domaine vat→bank_accounts pour 5 lignes).
async fn assert_onboarding_complete(pool: &MySqlPool) -> Result<(), AppError> {
    match onboarding::get_state(pool).await? {
        Some(s) if s.step_completed >= 7 || s.is_demo => Ok(()),
        Some(_) | None => Err(AppError::OnboardingNotComplete),
    }
}

/// Corps de `POST /api/v1/vat-rates`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVatRateBody {
    /// Clé de catégorie (extensible — non vide). Réservées : normal/reduced/
    /// special/exempt/custom ; toute autre valeur acceptée.
    pub category: String,
    /// Libellé d'affichage optionnel (défaut vide → l'UI affiche la catégorie).
    #[serde(default)]
    pub label: Option<String>,
    pub rate: Decimal,
    pub valid_from: NaiveDate,
    #[serde(default)]
    pub valid_to: Option<NaiveDate>,
}

/// Corps de `PUT /api/v1/vat-rates/{id}`. `rate` et `category` non modifiables.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVatRateBody {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub valid_to: Option<NaiveDate>,
    pub active: bool,
    pub version: i32,
}

/// Corps de `DELETE /api/v1/vat-rates/{id}` (désactivation soft — porte le
/// `version` pour le verrou optimiste).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeactivateVatRateBody {
    pub version: i32,
}

/// Valide et normalise la catégorie + le taux + les dates (forme).
/// `valid_from` est fourni à part pour la vérification `valid_to > valid_from`.
fn validate_category(category: &str) -> Result<String, AppError> {
    let trimmed = category.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("La catégorie est requise.".into()));
    }
    if trimmed.chars().count() > CATEGORY_MAX_LEN {
        return Err(AppError::Validation(format!(
            "La catégorie ne peut dépasser {CATEGORY_MAX_LEN} caractères."
        )));
    }
    Ok(trimmed.to_string())
}

fn validate_label(label: Option<&str>) -> Result<String, AppError> {
    let trimmed = label.unwrap_or("").trim();
    if trimmed.chars().count() > LABEL_MAX_LEN {
        return Err(AppError::Validation(format!(
            "Le libellé ne peut dépasser {LABEL_MAX_LEN} caractères."
        )));
    }
    Ok(trimmed.to_string())
}

fn validate_rate(rate: &Decimal) -> Result<(), AppError> {
    if *rate < Decimal::ZERO || *rate > Decimal::from(100) {
        return Err(AppError::Validation(
            "Le taux doit être compris entre 0 et 100.".into(),
        ));
    }
    if !scale_within(rate, 2) {
        return Err(AppError::Validation(
            "Le taux ne peut avoir plus de 2 décimales.".into(),
        ));
    }
    Ok(())
}

fn validate_dates(valid_from: NaiveDate, valid_to: Option<NaiveDate>) -> Result<(), AppError> {
    if let Some(to) = valid_to
        && to <= valid_from
    {
        return Err(AppError::Validation(
            "La date de fin de validité doit être postérieure à la date de début.".into(),
        ));
    }
    Ok(())
}

/// `POST /api/v1/vat-rates` — crée un taux TVA (Administrateur).
pub async fn create_vat_rate(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(body): Json<CreateVatRateBody>,
) -> Result<(StatusCode, Json<VatRateResponse>), AppError> {
    assert_onboarding_complete(&state.pool).await?;

    let category = validate_category(&body.category)?;
    // Le `label` d'affichage est optionnel ; s'il est vide, défaut = la
    // catégorie (la contrainte DB `chk_vat_rates_label_not_empty` exige un
    // label non vide ; l'UI affiche de toute façon la catégorie en priorité).
    let label = match validate_label(body.label.as_deref())? {
        s if s.is_empty() => category.clone(),
        s => s,
    };
    validate_rate(&body.rate)?;
    validate_dates(body.valid_from, body.valid_to)?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;

    bank_accounts::acquire_company_sentinel_lock(&mut tx, current_user.company_id).await?;

    // Invariant de non-chevauchement par catégorie (exclude_id=0 : pas d'id réel à 0).
    if vat_rates::has_overlap_in_category(
        &mut tx,
        current_user.company_id,
        &category,
        body.valid_from,
        body.valid_to,
        0,
    )
    .await?
    {
        return Err(AppError::Validation(
            "Un taux actif de cette catégorie chevauche déjà cette période de validité.".into(),
        ));
    }

    let new = NewVatRate {
        company_id: current_user.company_id,
        category: category.clone(),
        label,
        rate: body.rate,
        valid_from: body.valid_from,
        valid_to: body.valid_to,
    };
    let created = vat_rates::create_for_company(&mut tx, &new).await?;

    let details = serde_json::json!({
        "vat_rate_id": created.id,
        "category": created.category,
        "rate": created.rate.to_string(),
        "valid_from": created.valid_from,
        "valid_to": created.valid_to,
    });
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::for_actor(
            current_user.user_id,
            current_user.api_key_id,
            "vat_rate.created",
            "vat_rate",
            created.id,
            Some(details),
        ),
    )
    .await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit tx: {e}")))?;

    Ok((StatusCode::CREATED, Json(VatRateResponse::from(created))))
}

/// `PUT /api/v1/vat-rates/{id}` — modifie label / valid_to / active (Administrateur).
pub async fn update_vat_rate(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateVatRateBody>,
) -> Result<Json<VatRateResponse>, AppError> {
    assert_onboarding_complete(&state.pool).await?;

    let validated_label = validate_label(body.label.as_deref())?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;

    bank_accounts::acquire_company_sentinel_lock(&mut tx, current_user.company_id).await?;

    let existing = vat_rates::find_by_id_for_company(&mut tx, current_user.company_id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;

    // Label vide → défaut = catégorie (contrainte DB non-vide ; cf. create).
    let label = if validated_label.is_empty() {
        existing.category.clone()
    } else {
        validated_label
    };

    // `valid_to` doit rester > `valid_from` (figé) du taux.
    validate_dates(existing.valid_from, body.valid_to)?;

    // Si le taux reste/devient actif, re-vérifier le non-chevauchement (en
    // s'excluant soi-même). Un taux désactivé ne contraint pas la grille.
    if body.active
        && vat_rates::has_overlap_in_category(
            &mut tx,
            current_user.company_id,
            &existing.category,
            existing.valid_from,
            body.valid_to,
            id,
        )
        .await?
    {
        return Err(AppError::Validation(
            "Un autre taux actif de cette catégorie chevauche cette période de validité.".into(),
        ));
    }

    let fields = UpdateVatRate {
        label,
        valid_to: body.valid_to,
        active: body.active,
    };
    let updated =
        vat_rates::update_for_company(&mut tx, current_user.company_id, id, &fields, body.version)
            .await?;

    let details = serde_json::json!({
        "vat_rate_id": updated.id,
        "category": updated.category,
        "before": { "valid_to": existing.valid_to, "active": existing.active, "version": existing.version },
        "after": { "valid_to": updated.valid_to, "active": updated.active, "version": updated.version },
    });
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::for_actor(
            current_user.user_id,
            current_user.api_key_id,
            "vat_rate.updated",
            "vat_rate",
            updated.id,
            Some(details),
        ),
    )
    .await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit tx: {e}")))?;

    Ok(Json(VatRateResponse::from(updated)))
}

/// `DELETE /api/v1/vat-rates/{id}` — désactive (soft-delete) un taux (Administrateur).
pub async fn deactivate_vat_rate(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(body): Json<DeactivateVatRateBody>,
) -> Result<Json<VatRateResponse>, AppError> {
    assert_onboarding_complete(&state.pool).await?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;

    bank_accounts::acquire_company_sentinel_lock(&mut tx, current_user.company_id).await?;

    let deactivated =
        vat_rates::deactivate_for_company(&mut tx, current_user.company_id, id, body.version)
            .await?;

    let details = serde_json::json!({
        "vat_rate_id": deactivated.id,
        "category": deactivated.category,
        "rate": deactivated.rate.to_string(),
    });
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::for_actor(
            current_user.user_id,
            current_user.api_key_id,
            "vat_rate.deactivated",
            "vat_rate",
            deactivated.id,
            Some(details),
        ),
    )
    .await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit tx: {e}")))?;

    Ok(Json(VatRateResponse::from(deactivated)))
}
