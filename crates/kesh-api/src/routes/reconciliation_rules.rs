//! Routes Story 8-5b — CRUD `reconciliation_rules` (FR47).
//!
//! - `POST   /api/v1/reconciliation/rules`       (Comptable+) — création.
//! - `GET    /api/v1/reconciliation/rules`       (tous rôles) — liste paginée.
//! - `GET    /api/v1/reconciliation/rules/{id}`  (tous rôles) — détail.
//! - `PATCH  /api/v1/reconciliation/rules/{id}`  (Comptable+) — patch partial + optimistic lock.
//! - `DELETE /api/v1/reconciliation/rules/{id}`  (Comptable+) — soft-delete idempotent (204).
//!
//! Validation handler-side AVANT INSERT/UPDATE (Pass 1 P-M ECH-07) :
//! label/match_value empty + length + priority range + IBAN canonical
//! normalize. Cross-tenant scoping via `current_user.company_id`
//! (KF-002, 404 systématique).
//!
//! Pre-flight `accounts::find_by_id_in_company` valide que
//! `counterpartyAccountId` existe ET `active=true` ET appartient à la
//! company courante. Distinct du FK MariaDB qui ne peut pas garantir
//! cross-tenant.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

use crate::audit::AuditActor;
use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::{
    NewReconciliationRule, ReconciliationMatchType, ReconciliationRule, UpdateReconciliationRule,
};
use kesh_db::errors::DbError;
use kesh_db::repositories::{accounts, audit_log, reconciliation_rules};

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;
use crate::routes::ListResponse;

// ---------------------------------------------------------------------------
// Constantes (Pass 1 P-M ECH-07 validation bornes)
// ---------------------------------------------------------------------------

const LABEL_MAX_CHARS: usize = 120;
const MATCH_VALUE_MAX_CHARS: usize = 255;
const PRIORITY_MIN: i32 = 1;
const PRIORITY_MAX: i32 = 1000;
const DEFAULT_PRIORITY: i32 = 100;
const PER_PAGE_DEFAULT: i64 = 50;
const PER_PAGE_MAX: i64 = 200;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationRuleResponse {
    pub id: i64,
    pub label: String,
    pub match_type: ReconciliationMatchType,
    pub match_value: String,
    pub counterparty_account_id: i64,
    pub priority: i32,
    pub active: bool,
    /// Projet analytique par défaut (Story 19-5) — `null` si aucun.
    pub default_project_id: Option<i64>,
    pub applied_count: i64,
    pub last_applied_at: Option<chrono::NaiveDateTime>,
    pub version: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl From<ReconciliationRule> for ReconciliationRuleResponse {
    fn from(r: ReconciliationRule) -> Self {
        Self {
            id: r.id,
            label: r.label,
            match_type: r.match_type,
            match_value: r.match_value,
            counterparty_account_id: r.counterparty_account_id,
            priority: r.priority,
            active: r.active,
            default_project_id: r.default_project_id,
            applied_count: r.applied_count,
            last_applied_at: r.last_applied_at,
            version: r.version,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateRuleRequest {
    pub label: String,
    pub match_type: ReconciliationMatchType,
    pub match_value: String,
    pub counterparty_account_id: i64,
    #[serde(default = "default_priority")]
    pub priority: i32,
    /// Projet analytique par défaut (Story 19-5) — optionnel.
    #[serde(default)]
    pub default_project_id: Option<i64>,
}

fn default_priority() -> i32 {
    DEFAULT_PRIORITY
}

/// Deserializer « double option » (Story 19-5) : distingue un champ **absent**
/// (→ `None`, inchangé) d'un champ **présent à `null`** (→ `Some(None)`,
/// effacement). Sans ce helper, serde replie `null` sur le `None` externe et
/// l'effacement du projet par défaut via PATCH serait un no-op silencieux
/// (bug HIGH Pass 1 Blind Hunter). À combiner avec `#[serde(default)]` pour
/// gérer le cas absent.
fn double_option<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: serde::Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    serde::Deserialize::deserialize(deserializer).map(Some)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateRuleRequest {
    /// Optimistic lock — obligatoire.
    pub expected_version: i32,
    pub label: Option<String>,
    /// `match_type` PAS patchable v0.1 (cf. §rules-types-rust).
    pub match_value: Option<String>,
    pub counterparty_account_id: Option<i64>,
    pub priority: Option<i32>,
    pub active: Option<bool>,
    /// Projet analytique par défaut (Story 19-5) — sémantique deux niveaux :
    /// absent = inchangé ; `null` = effacer ; `<id>` = affecter. Mappé sur
    /// `UpdateReconciliationRule.default_project_id: Option<Option<i64>>`.
    /// `deserialize_with = "double_option"` est **obligatoire** pour que
    /// `null` produise `Some(None)` (effacement) plutôt que `None` (inchangé).
    #[serde(default, deserialize_with = "double_option")]
    pub default_project_id: Option<Option<i64>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
    /// Filtre `active` : `?active=true` / `?active=false`. Absent → tous.
    pub active: Option<bool>,
}

fn default_page() -> i64 {
    1
}
fn default_per_page() -> i64 {
    PER_PAGE_DEFAULT
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Normalisation IBAN canonique (Pass 1 P-H4 ECH-03 + BH-F29) :
/// `uppercase + strip whitespace`. Identique à
/// `kesh_qrbill::validation::normalize_iban` (réimplémenté inline pour
/// éviter coupling `kesh-qrbill` — le matching IBAN n'a aucune relation
/// avec la génération QR-Bill).
fn normalize_iban_canonical(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_uppercase()
}

fn validate_label(label: &str) -> Result<(), AppError> {
    if label.trim().is_empty() {
        return Err(AppError::Validation("label vide".into()));
    }
    if label.chars().count() > LABEL_MAX_CHARS {
        return Err(AppError::Validation(format!(
            "label trop long (max {LABEL_MAX_CHARS} chars)"
        )));
    }
    Ok(())
}

fn validate_match_value(match_value: &str) -> Result<(), AppError> {
    if match_value.trim().is_empty() {
        return Err(AppError::Validation("matchValue vide".into()));
    }
    if match_value.chars().count() > MATCH_VALUE_MAX_CHARS {
        return Err(AppError::Validation(format!(
            "matchValue trop long (max {MATCH_VALUE_MAX_CHARS} chars)"
        )));
    }
    Ok(())
}

fn validate_priority(priority: i32) -> Result<(), AppError> {
    if !(PRIORITY_MIN..=PRIORITY_MAX).contains(&priority) {
        return Err(AppError::Validation(format!(
            "priority doit être dans [{PRIORITY_MIN}, {PRIORITY_MAX}]"
        )));
    }
    Ok(())
}

/// Pré-flight : valide que `counterparty_account_id` existe, est actif
/// et appartient à `company_id`. Sinon → 404 `ACCOUNT_NOT_FOUND` pour
/// anti-énumération KF-002 (compte archivé ne leak pas son existence).
async fn validate_counterparty_account(
    state: &AppState,
    company_id: i64,
    account_id: i64,
) -> Result<(), AppError> {
    if account_id <= 0 {
        return Err(AppError::Validation(
            "counterpartyAccountId doit être strictement positif".into(),
        ));
    }
    let account = accounts::find_by_id_in_company(&state.pool, account_id, company_id)
        .await?
        .ok_or(AppError::AccountNotFound {
            account_id,
            missing_account_ids: None,
        })?;
    if !account.active {
        return Err(AppError::AccountNotFound {
            account_id,
            missing_account_ids: None,
        });
    }
    Ok(())
}

/// Mappe `DbError::UniqueConstraintViolation` sur
/// `uq_reconciliation_rules_match_active` → 409
/// `RECONCILIATION_RULE_DUPLICATE`. Autres erreurs DB pass-through.
fn map_create_or_update_db_err(
    err: DbError,
    match_type: ReconciliationMatchType,
    match_value: &str,
) -> AppError {
    if reconciliation_rules::is_duplicate_rule_constraint(&err) {
        AppError::ReconciliationRuleDuplicate {
            match_type: match_type.as_str().to_string(),
            match_value: match_value.to_string(),
        }
    } else {
        AppError::Database(err)
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/v1/reconciliation/rules — Comptable+, AC #101.
pub async fn post_create(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateRuleRequest>,
) -> Result<(StatusCode, Json<ReconciliationRuleResponse>), AppError> {
    // Validation handler-side avant DB INSERT (Pass 1 P-M ECH-07).
    validate_label(&req.label)?;
    validate_match_value(&req.match_value)?;
    validate_priority(req.priority)?;

    // Normalisation IBAN canonique pour `IbanExact` (Pass 1 P-H4).
    let normalized_match_value = if req.match_type == ReconciliationMatchType::IbanExact {
        normalize_iban_canonical(&req.match_value)
    } else {
        req.match_value.clone()
    };

    // Pre-flight counterparty account (AC #104).
    validate_counterparty_account(&state, current_user.company_id, req.counterparty_account_id)
        .await?;

    let new_rule = NewReconciliationRule {
        label: req.label.trim().to_string(),
        match_type: req.match_type,
        match_value: normalized_match_value,
        counterparty_account_id: req.counterparty_account_id,
        priority: req.priority,
        default_project_id: req.default_project_id,
    };
    let match_type_for_err = req.match_type;
    let match_value_for_err = new_rule.match_value.clone();

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;

    let rule = reconciliation_rules::create_in_tx(
        &mut tx,
        current_user.company_id,
        current_user.user_id,
        &new_rule,
    )
    .await
    .map_err(|e| map_create_or_update_db_err(e, match_type_for_err, &match_value_for_err))?;

    tx.commit()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;

    Ok((StatusCode::CREATED, Json(rule.into())))
}

/// GET /api/v1/reconciliation/rules — tous rôles, AC #105 + #106 + #107.
pub async fn list(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ListResponse<ReconciliationRuleResponse>>, AppError> {
    let page = query.page.max(1);
    let per_page = query.per_page.clamp(1, PER_PAGE_MAX);

    let (rules, total) = reconciliation_rules::list_by_company_paginated(
        &state.pool,
        current_user.company_id,
        query.active,
        page,
        per_page,
    )
    .await?;

    let items = rules.into_iter().map(Into::into).collect();
    let offset = (page - 1) * per_page;
    Ok(Json(ListResponse {
        items,
        total,
        limit: per_page,
        offset,
    }))
}

/// GET /api/v1/reconciliation/rules/{id} — tous rôles. AC #107 (multi-tenant).
pub async fn detail(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<ReconciliationRuleResponse>, AppError> {
    let rule =
        reconciliation_rules::find_by_id_for_company(&state.pool, current_user.company_id, id)
            .await?
            .ok_or(AppError::ReconciliationRuleNotFound { rule_id: id })?;
    Ok(Json(rule.into()))
}

/// PATCH /api/v1/reconciliation/rules/{id} — Comptable+. AC #108 (optimistic lock),
/// AC #109 (reactivate), AC #109b (reactivation conflict).
pub async fn patch(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateRuleRequest>,
) -> Result<Json<ReconciliationRuleResponse>, AppError> {
    // Validation handler-side avant UPDATE.
    if req.expected_version < 1 {
        return Err(AppError::Validation(
            "expectedVersion doit être >= 1".into(),
        ));
    }
    if let Some(ref label) = req.label {
        validate_label(label)?;
    }
    if let Some(ref mv) = req.match_value {
        validate_match_value(mv)?;
    }
    if let Some(priority) = req.priority {
        validate_priority(priority)?;
    }
    if let Some(account_id) = req.counterparty_account_id {
        validate_counterparty_account(&state, current_user.company_id, account_id).await?;
    }

    // Pré-load rule actuelle pour normalisation IBAN PATCH (Pass 2 Q4 ECH2-1).
    // Si match_value est patché ET la rule a match_type=IbanExact, normaliser.
    let current =
        reconciliation_rules::find_by_id_for_company(&state.pool, current_user.company_id, id)
            .await?
            .ok_or(AppError::ReconciliationRuleNotFound { rule_id: id })?;

    let normalized_match_value = match (&req.match_value, current.match_type) {
        (Some(mv), ReconciliationMatchType::IbanExact) => Some(normalize_iban_canonical(mv)),
        (Some(mv), _) => Some(mv.clone()),
        (None, _) => None,
    };

    let patch_data = UpdateReconciliationRule {
        label: req.label.map(|l| l.trim().to_string()),
        match_value: normalized_match_value.clone(),
        counterparty_account_id: req.counterparty_account_id,
        priority: req.priority,
        active: req.active,
        default_project_id: req.default_project_id,
    };

    let match_type_for_err = current.match_type;
    let match_value_for_err = normalized_match_value.unwrap_or(current.match_value.clone());

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;

    let updated = reconciliation_rules::update_in_tx(
        &mut tx,
        current_user.company_id,
        current_user.user_id,
        id,
        req.expected_version,
        &patch_data,
    )
    .await
    .map_err(|e| map_create_or_update_db_err(e, match_type_for_err, &match_value_for_err))?;

    tx.commit()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;

    Ok(Json(updated.into()))
}

/// DELETE /api/v1/reconciliation/rules/{id} — Comptable+. AC #110 + AC #111
/// (idempotent — 204 dans les 2 cas pour Pass 2 Q7 simplification).
///
/// Si la rule existe et est active → soft-delete + audit `reconciliation_rule.deleted`.
/// Si la rule existe mais déjà inactive → no-op (idempotent, 204).
/// Si la rule n'existe pas (ou cross-tenant) → 404 (anti-énumération KF-002).
pub async fn delete(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    // Pass 1 code review HIGH EC2 fix : pre-flight existence check
    // déplacé INSIDE la transaction pour éliminer la race window où
    // un concurrent PATCH (reactivate) entre l'existence check et le
    // soft-delete causerait un double-delete avec audit trail corrompu
    // (violation CO Art. 958f). Pattern aligné sur le PATCH handler.
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;

    // Existence check transaction-bound : si la rule n'existe pas
    // (ou cross-tenant), 404 sans leak d'existence (KF-002).
    reconciliation_rules::find_by_id_for_company(&mut *tx, current_user.company_id, id)
        .await?
        .ok_or(AppError::ReconciliationRuleNotFound { rule_id: id })?;

    let transitioned = reconciliation_rules::soft_delete_by_id_for_company(
        &mut tx,
        current_user.company_id,
        current_user.user_id,
        id,
    )
    .await?;

    // Idempotence : si rule déjà inactive (transitioned=false), on log un
    // audit informatif léger pour traçabilité (action
    // `reconciliation_rule.deleted` avec `idempotent_noop: true`).
    if !transitioned {
        audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::from_current_user(
                &current_user,
                "reconciliation_rule.deleted",
                "reconciliation_rules",
                id,
                Some(serde_json::json!({
                    "rule_id": id,
                    "soft_delete": true,
                    "idempotent_noop": true,
                })),
            ),
        )
        .await?;
    }

    tx.commit()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;

    Ok(StatusCode::NO_CONTENT)
}
