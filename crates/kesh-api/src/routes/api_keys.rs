//! Routes de gestion des clés API (PAT) — `/api/v1/settings/api-keys`.
//!
//! Story 17-2a (#100, AC7/AC8). Guard `require_comptable_role` (DC4 — un
//! Comptable gère ses intégrations, aligné sur le CRUD `bank_accounts`).
//!
//! **DC6 — gestion interdite via PAT** : ces 3 handlers refusent
//! `403 API_KEY_MANAGEMENT_FORBIDDEN` si la requête est elle-même authentifiée
//! par PAT (`current_user.api_key_id.is_some()`), même `read-write` — sinon une
//! clé fuitée pourrait se cloner/escalader (auto-propagation). La gestion est
//! réservée à la session JWT cookie (UI web).

use std::str::FromStr;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use kesh_db::entities::api_key::{ApiKey, ApiKeyScope, NewApiKey};
use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::errors::DbError;
use kesh_db::repositories::{api_keys, audit_log};

use crate::AppState;
use crate::auth::api_key::generate_pat;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;

/// Réponse de liste/détail d'une clé (jamais le hash ni le secret).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyResponse {
    pub id: i64,
    pub name: String,
    pub scope: String,
    pub created_at: NaiveDateTime,
    pub last_used_at: Option<NaiveDateTime>,
    pub revoked_at: Option<NaiveDateTime>,
    pub expires_at: Option<NaiveDateTime>,
    pub version: i32,
}

impl From<ApiKey> for ApiKeyResponse {
    fn from(k: ApiKey) -> Self {
        Self {
            id: k.id,
            name: k.name,
            scope: k.scope.as_str().to_string(),
            created_at: k.created_at,
            last_used_at: k.last_used_at,
            revoked_at: k.revoked_at,
            expires_at: k.expires_at,
            version: k.version,
        }
    }
}

/// Corps de création d'une clé.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub scope: String,
    /// Expiration optionnelle (RFC 3339, ex. `2027-01-01T00:00:00Z`).
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
}

/// Réponse de création : contient le secret en clair **une seule fois**.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApiKeyResponse {
    pub id: i64,
    pub name: String,
    pub scope: String,
    pub created_at: NaiveDateTime,
    /// Secret en clair `kesh_pat_…` — affiché une seule fois, jamais re-récupérable.
    pub key: String,
}

/// Corps de révocation (optimistic lock).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevokeApiKeyRequest {
    pub version: i32,
}

/// DC6 — refuse l'accès aux routes de gestion si la requête est authentifiée
/// par PAT (même `read-write`).
///
/// Story 17-3a — promu `pub(crate)` : réutilisé par les endpoints d'infra
/// destructeurs (`admin/full-export`, et 17-3c `full-import`) qui doivent eux
/// aussi être interdits aux clés PAT (AC2).
pub(crate) fn ensure_not_pat(current_user: &CurrentUser) -> Result<(), AppError> {
    if current_user.api_key_id.is_some() {
        return Err(AppError::ApiKeyManagementForbidden);
    }
    Ok(())
}

/// `GET /api/v1/settings/api-keys` — liste **toutes** les clés de la company
/// (actives ET révoquées, `include_revoked=true`), triées `created_at DESC`.
pub async fn list(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<ApiKeyResponse>>, AppError> {
    ensure_not_pat(&current_user)?;

    let keys = api_keys::list_by_company(&state.pool, current_user.company_id, true).await?;
    Ok(Json(keys.into_iter().map(ApiKeyResponse::from).collect()))
}

/// `POST /api/v1/settings/api-keys` — crée une clé, retourne le secret en clair
/// une seule fois. Audit `api_key.created` (`actor_type='user'`).
pub async fn create(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<CreateApiKeyResponse>), AppError> {
    ensure_not_pat(&current_user)?;

    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Validation("le nom de la clé est requis".into()));
    }
    // Borne la longueur côté handler : `name` alimente une colonne VARCHAR(255).
    // Sans ce garde, MySQL renvoie 1406 « Data too long » → `DbError::Sqlx` → HTTP 500
    // au lieu d'un 400 actionnable côté client (code-review 17-2a Pass 1).
    if name.chars().count() > 255 {
        return Err(AppError::Validation(
            "le nom de la clé est trop long (255 caractères maximum)".into(),
        ));
    }
    let scope = ApiKeyScope::from_str(&req.scope)
        .map_err(|_| AppError::Validation("scope invalide (read | read-write)".into()))?;
    // Refuse une expiration déjà passée : sinon la clé est créée « mort-née »
    // (201 + secret retourné, mais `find_active_by_key_hash` l'exclut aussitôt via
    // `expires_at > NOW(3)`), sans feedback au client (code-review 17-2a Pass 1).
    if let Some(exp) = req.expires_at
        && exp <= Utc::now()
    {
        return Err(AppError::Validation(
            "la date d'expiration doit être dans le futur".into(),
        ));
    }
    let expires_at: Option<NaiveDateTime> = req.expires_at.map(|dt| dt.naive_utc());

    // Génère le secret : (token_clair, key_hash). Seul le hash est persisté.
    let (token, key_hash) = generate_pat();

    let new = NewApiKey {
        company_id: current_user.company_id,
        created_by_user_id: current_user.user_id,
        name: name.clone(),
        key_hash,
        scope,
        expires_at,
    };

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;

    let created = api_keys::create_in_tx(&mut tx, new).await?;

    // Audit `api_key.created` — actor_type='user' (c'est un user UI qui gère
    // ses clés ; DC6 garantit qu'aucun PAT n'atteint ce handler). Jamais le secret.
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            current_user.user_id,
            "api_key.created",
            "api_key",
            created.id,
            Some(serde_json::json!({
                "name": created.name,
                "scope": created.scope.as_str(),
            })),
        ),
    )
    .await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit tx: {e}")))?;

    Ok((
        StatusCode::CREATED,
        Json(CreateApiKeyResponse {
            id: created.id,
            name: created.name,
            scope: created.scope.as_str().to_string(),
            created_at: created.created_at,
            key: token,
        }),
    ))
}

/// `DELETE /api/v1/settings/api-keys/{id}` — révocation soft-delete avec
/// optimistic lock. `404` si la clé est absente / d'une autre company.
/// Audit `api_key.revoked` (`actor_type='user'`).
pub async fn revoke(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<RevokeApiKeyRequest>,
) -> Result<StatusCode, AppError> {
    ensure_not_pat(&current_user)?;

    if req.version < 1 {
        return Err(AppError::Validation("version doit être >= 1".into()));
    }

    // Pré-flight 404 (anti-énumération KF-002 — None si autre company).
    let existing = api_keys::find_by_id_for_company(&state.pool, current_user.company_id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;

    api_keys::revoke_in_tx(&mut tx, current_user.company_id, id, req.version).await?;

    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            current_user.user_id,
            "api_key.revoked",
            "api_key",
            id,
            Some(serde_json::json!({
                "name": existing.name,
                "scope": existing.scope.as_str(),
            })),
        ),
    )
    .await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit tx: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}
