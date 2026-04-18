//! Handlers HTTP pour `/api/v1/users/*` (story 1.7, refactored story 1.8).
//!
//! Tous les endpoints requièrent le rôle `Admin`. L'enforcement est fait
//! par le middleware RBAC (`require_admin_role`) appliqué via `route_layer`
//! sur le sous-routeur `/users/*` dans `build_router()`.

use axum::Extension;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chrono::NaiveDateTime;
use kesh_db::entities::{NewUser, Role, User, UserUpdate};
use kesh_db::repositories::{refresh_tokens, users};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::password;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;

// === DTOs ===

/// Corps de `POST /api/v1/users`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: Role,
}

impl std::fmt::Debug for CreateUserRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CreateUserRequest")
            .field("username", &self.username)
            .field("password", &"***")
            .field("role", &self.role)
            .finish()
    }
}

/// Corps de `PUT /api/v1/users/:id`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    pub role: Role,
    pub active: bool,
    pub version: i32,
}

/// Corps de `PUT /api/v1/users/:id/reset-password`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordRequest {
    pub new_password: String,
}

impl std::fmt::Debug for ResetPasswordRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResetPasswordRequest")
            .field("new_password", &"***")
            .finish()
    }
}

/// Réponse utilisateur (jamais de `password_hash`).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub role: Role,
    pub active: bool,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            role: u.role,
            active: u.active,
            version: u.version,
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}

/// Réponse paginée pour `GET /api/v1/users`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserListResponse {
    pub items: Vec<UserResponse>,
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
}

/// Paramètres de pagination pour `GET /api/v1/users`.
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// === Handlers ===
// Note: require_admin() supprimé en story 1.8 — le middleware RBAC
// (require_admin_role) appliqué via route_layer sur le sous-routeur
// /users/* s'en charge. Les handlers qui utilisent encore current_user
// le font pour des gardes métier (self-disable, last-admin).

/// `POST /api/v1/users` — Création d'utilisateur (Admin via middleware RBAC).
/// Story 6.2: User created in the current user's company.
pub async fn create_user(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    // Validation username : trim + longueur [1, 64]
    let username = req.username.trim().to_string();
    if username.is_empty() {
        return Err(AppError::Validation(state.i18n.format(
            &state.config.locale,
            "error-username-empty",
            None,
        )));
    }
    if username.chars().count() > 64 {
        let mut args = kesh_i18n::FluentArgs::new();
        args.set("max", 64_i64);
        return Err(AppError::Validation(state.i18n.format(
            &state.config.locale,
            "error-username-too-long",
            Some(&args),
        )));
    }

    // Validation mot de passe (politique configurable)
    password::validate_password(&req.password, state.config.password_min_length)?;

    // Hash Argon2id (async via spawn_blocking)
    let password_hash = password::hash_password_async(req.password).await?;

    let new_user = NewUser {
        username,
        password_hash,
        role: req.role,
        active: true,
        company_id: current_user.company_id,
    };

    let user = users::create(&state.pool, new_user).await?;
    tracing::info!(user_id = user.id, role = ?user.role, company_id = current_user.company_id, "user created");

    Ok((StatusCode::CREATED, Json(UserResponse::from(user))))
}

/// `PUT /api/v1/users/:id` — Modification d'utilisateur (Admin via middleware RBAC).
///
/// Garde `Extension<CurrentUser>` pour les gardes métier (self-disable, last-admin).
/// Story 6.2: Scoped by current_user.company_id (IDOR protection).
pub async fn update_user(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    // Vérifier que l'utilisateur existe ET appartient à la même company
    let user = users::find_by_id_in_company(&state.pool, id, current_user.company_id)
        .await?
        .ok_or(AppError::Database(kesh_db::errors::DbError::NotFound))?;

    // Gardes pour protéger le pool d'admins actifs (F1+F3+P1 code review).
    let is_deactivating = user.active && !req.active;
    let is_demoting_admin = user.role == Role::Admin && req.role != Role::Admin && user.active;
    let removes_active_admin = (is_deactivating && user.role == Role::Admin) || is_demoting_admin;

    if is_deactivating && id == current_user.user_id {
        return Err(AppError::CannotDisableSelf);
    }

    if removes_active_admin {
        let admin_count = users::count_active_by_role(&state.pool, Role::Admin).await?;
        if admin_count <= 1 {
            return Err(AppError::CannotDisableLastAdmin);
        }
    }

    let changes = UserUpdate {
        role: req.role,
        active: req.active,
    };

    let updated = users::update_role_and_active(&state.pool, id, req.version, changes).await?;

    // Révoquer les sessions si désactivation
    if is_deactivating {
        refresh_tokens::revoke_all_for_user(&state.pool, id, "admin_disable").await?;
    }

    tracing::info!(user_id = id, role = ?updated.role, active = updated.active, "user updated");

    Ok(Json(UserResponse::from(updated)))
}

/// `PUT /api/v1/users/:id/disable` — Désactivation de compte (Admin via middleware RBAC).
///
/// Garde `Extension<CurrentUser>` pour le self-disable check.
/// Story 6.2: Scoped by current_user.company_id (IDOR protection).
pub async fn disable_user(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<UserResponse>, AppError> {
    // Self-disable interdit
    if id == current_user.user_id {
        return Err(AppError::CannotDisableSelf);
    }

    let user = users::find_by_id_in_company(&state.pool, id, current_user.company_id)
        .await?
        .ok_or(AppError::Database(kesh_db::errors::DbError::NotFound))?;

    // Protection du dernier admin
    if user.role == Role::Admin {
        let admin_count = users::count_active_by_role(&state.pool, Role::Admin).await?;
        if admin_count <= 1 {
            return Err(AppError::CannotDisableLastAdmin);
        }
    }

    let changes = UserUpdate {
        role: user.role,
        active: false,
    };

    let updated = users::update_role_and_active(&state.pool, id, user.version, changes).await?;

    // Invalider toutes les sessions
    refresh_tokens::revoke_all_for_user(&state.pool, id, "admin_disable").await?;
    tracing::info!(user_id = id, "user disabled + sessions revoked");

    Ok(Json(UserResponse::from(updated)))
}

/// `PUT /api/v1/users/:id/reset-password` — Réinitialisation mot de passe (Admin via middleware RBAC).
/// Story 6.2: Scoped by current_user.company_id (IDOR protection).
pub async fn reset_password(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<Json<UserResponse>, AppError> {
    // Vérifier que l'utilisateur cible existe ET appartient à la même company
    users::find_by_id_in_company(&state.pool, id, current_user.company_id)
        .await?
        .ok_or(AppError::Database(kesh_db::errors::DbError::NotFound))?;

    // Validation mot de passe
    password::validate_password(&req.new_password, state.config.password_min_length)?;

    // Hash + update
    let new_hash = password::hash_password_async(req.new_password).await?;
    users::update_password(&state.pool, id, &new_hash).await?;

    // Invalider toutes les sessions de l'utilisateur cible
    refresh_tokens::revoke_all_for_user(&state.pool, id, "password_change").await?;

    // Re-fetch pour avoir la version mise à jour
    let user = users::find_by_id_in_company(&state.pool, id, current_user.company_id)
        .await?
        .ok_or(AppError::Database(kesh_db::errors::DbError::NotFound))?;

    tracing::info!(user_id = id, "password reset by admin + sessions revoked");

    Ok(Json(UserResponse::from(user)))
}

/// `GET /api/v1/users` — Liste paginée des utilisateurs (Admin via middleware RBAC).
/// Story 6.2: Scoped by current_user.company_id (IDOR protection).
pub async fn list_users(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<UserListResponse>, AppError> {
    let limit = params.limit.unwrap_or(50).clamp(1, 1000);
    let offset = params.offset.unwrap_or(0).max(0);

    let total = users::count_by_company(&state.pool, current_user.company_id).await?;
    let items: Vec<UserResponse> = users::list_by_company(&state.pool, current_user.company_id, limit, offset)
        .await?
        .into_iter()
        .map(UserResponse::from)
        .collect();

    Ok(Json(UserListResponse {
        items,
        total,
        offset,
        limit,
    }))
}

/// `GET /api/v1/users/:id` — Détail d'un utilisateur (Admin via middleware RBAC).
/// Story 6.2: Scoped by current_user.company_id (IDOR protection).
pub async fn get_user(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<UserResponse>, AppError> {
    let user = users::find_by_id_in_company(&state.pool, id, current_user.company_id)
        .await?
        .ok_or(AppError::Database(kesh_db::errors::DbError::NotFound))?;

    Ok(Json(UserResponse::from(user)))
}
