//! Handlers HTTP pour `/api/v1/auth/*`.

use std::net::SocketAddr;

use axum::extract::{ConnectInfo, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use kesh_db::entities::NewRefreshToken;
use kesh_db::repositories::{refresh_tokens, users};
use serde::{Deserialize, Serialize};

use crate::auth::{jwt, password};
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;
use crate::AppState;

// === DTOs ===

/// Corps de la requête `POST /api/v1/auth/login`.
///
/// `Debug` manuel : masque le `password` pour ne jamais le leaker
/// via `tracing::debug!("{:?}", req)`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

impl std::fmt::Debug for LoginRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoginRequest")
            .field("username", &self.username)
            .field("password", &"***")
            .finish()
    }
}

/// Réponse de `POST /api/v1/auth/login`.
///
/// **Attention** : contient `access_token` et `refresh_token` en clair.
/// NE JAMAIS logger cette structure en entier. Logger uniquement
/// `user_id` et `expires_in` si besoin.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

/// Corps de `POST /api/v1/auth/logout`.
///
/// `Debug` manuel : masque le `refresh_token`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogoutRequest {
    pub refresh_token: String,
}

impl std::fmt::Debug for LogoutRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LogoutRequest")
            .field("refresh_token", &"***")
            .finish()
    }
}

// === Handlers ===

/// `POST /api/v1/auth/login`
///
/// Flow (story 1.6 : rate limiter + sliding expiry + spawn_blocking Argon2) :
/// 0. Rate limit check : si IP bloquée → 429 immédiat.
/// 1. Validation : rejet si username/password vide (400).
/// 2. Lookup de l'utilisateur via `users::find_by_username`.
/// 3. Timing-attack mitigation : si user inconnu OU inactif, passer par
///    `dummy_verify` puis retourner 401 générique.
/// 4. Verify Argon2id (async via spawn_blocking) ; si mismatch, 401 générique.
/// 5. Encode JWT HS256 avec claims `sub`, `role`, `iat`, `exp`.
/// 6. Génère un refresh_token UUID v4 et le persiste (sliding expiry).
/// 7. Retourne `{accessToken, refreshToken, expiresIn}`.
pub async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    // Step 0 : rate limit check
    let ip = addr.ip();

    if let Err(reject) = state.rate_limiter.check_rate_limit(ip) {
        tracing::warn!(ip = %ip, "rate limit triggered");
        return Err(AppError::RateLimited {
            retry_after: reject.retry_after_secs,
        });
    }

    // Step 0 : validation
    // Username trimé pour la comparaison (patch #8) — évite qu'un user
    // qui tape accidentellement "alice " (trailing space) reçoive un
    // 401 sans comprendre pourquoi. La collation utf8mb4_unicode_ci
    // de la DB est déjà case-insensitive, mais le whitespace doit être
    // géré explicitement côté application.
    //
    // Password : on NE trim PAS (un password peut légitimement commencer
    // ou finir par un espace — byte-exact semantics). Mais on rejette un
    // password composé EXCLUSIVEMENT de whitespace, pour symétrie avec le
    // username et pour fermer un side-channel d'énumération (patch V2) :
    // sans cette vérification, un password `"   "` passe `is_empty()` et
    // atteint le verify Argon2, permettant de distinguer les codes de
    // retour entre `password=""` (400) et `password="   "` (401).
    let username = req.username.trim();
    let password_all_whitespace =
        !req.password.is_empty() && req.password.chars().all(char::is_whitespace);
    if username.is_empty() || req.password.is_empty() || password_all_whitespace {
        return Err(AppError::Validation(
            "username and password must be non-empty and non-whitespace".into(),
        ));
    }

    // Step 1 : lookup
    let user_opt = users::find_by_username(&state.pool, username).await?;

    // Step 2 : timing-attack mitigation — user inconnu ou inactif
    // convergent vers la même branche que bad password (via dummy_verify).
    let user = match user_opt {
        Some(u) if u.active => u,
        Some(_) | None => {
            password::dummy_verify();
            state.rate_limiter.record_failed_attempt(ip);
            tracing::info!(ip = %ip, "rate limit: failed attempt (unknown/inactive user)");
            return Err(AppError::InvalidCredentials);
        }
    };

    // Step 3 : verify Argon2id (async via spawn_blocking)
    let plain = req.password.clone();
    let phc = user.password_hash.clone();
    if !password::verify_password_async(plain, phc).await? {
        state.rate_limiter.record_failed_attempt(ip);
        tracing::info!(ip = %ip, "rate limit: failed attempt (bad password)");
        return Err(AppError::InvalidCredentials);
    }

    // Step 4 : JWT
    let access_token = jwt::encode(
        user.id,
        user.role,
        state.config.jwt_secret_bytes(),
        state.config.jwt_expiry,
    )?;

    // Step 5 : refresh token (sliding expiry — story 1.6)
    let refresh_token = uuid::Uuid::new_v4().to_string();
    let expires_at = (Utc::now() + state.config.refresh_inactivity).naive_utc();

    refresh_tokens::create(
        &state.pool,
        NewRefreshToken {
            user_id: user.id,
            token: refresh_token.clone(),
            expires_at,
        },
    )
    .await?;

    // Rate limiter : reset après succès
    state.rate_limiter.reset(ip);

    tracing::info!(user_id = user.id, "login success");

    Ok(Json(LoginResponse {
        access_token,
        refresh_token,
        expires_in: state.config.jwt_expiry.num_seconds(),
    }))
}

/// `POST /api/v1/auth/logout`
///
/// Invalide le refresh_token en base (`revoked_at = NOW()`). Idempotent :
/// un token déjà révoqué, expiré ou inexistant retourne aussi 204.
/// N'exige PAS de JWT valide — un client avec un access_token expiré doit
/// pouvoir invalider sa session.
pub async fn logout(
    State(state): State<AppState>,
    Json(req): Json<LogoutRequest>,
) -> Result<impl IntoResponse, AppError> {
    let revoked = refresh_tokens::revoke_by_token(&state.pool, &req.refresh_token, "logout").await?;
    tracing::info!(revoked = revoked, "logout");
    Ok(StatusCode::NO_CONTENT)
}

// === Story 1.6 : refresh & change_password ===

/// Corps de la requête `POST /api/v1/auth/refresh`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRequest {
    pub refresh_token: String,
}

impl std::fmt::Debug for RefreshRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RefreshRequest")
            .field("refresh_token", &"***")
            .finish()
    }
}

/// Réponse de `POST /api/v1/auth/refresh`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

/// `POST /api/v1/auth/refresh`
///
/// Token rotation avec détection de vol (OWASP best practice).
/// Séquence complète documentée dans story 1.6, Task 3.3.
pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<RefreshResponse>, AppError> {
    // Step 1 : chercher le token (actif OU révoqué)
    let token_opt = refresh_tokens::find_by_token_include_revoked(&state.pool, &req.refresh_token)
        .await?;

    let token = match token_opt {
        Some(t) => t,
        None => {
            tracing::warn!("refresh with unknown token");
            return Err(AppError::InvalidRefreshToken("unknown token".into()));
        }
    };

    // Step 2-4 : vérifier si révoqué
    if token.revoked_at.is_some() {
        if token.revoked_reason.as_deref() == Some("rotation") {
            // Détection de vol : mass revoke tous les tokens de l'utilisateur
            let revoked_count = refresh_tokens::revoke_all_for_user(
                &state.pool,
                token.user_id,
                "theft_detected",
            )
            .await?;
            tracing::warn!(
                user_id = token.user_id,
                revoked_count = revoked_count,
                "token replay detected, revoking all sessions"
            );
            return Err(AppError::InvalidRefreshToken("token replay detected".into()));
        }
        // Révoqué par logout, password_change, etc. → 401 simple
        return Err(AppError::InvalidRefreshToken("token revoked".into()));
    }

    // Step 5 : vérifier expiration
    let now = Utc::now().naive_utc();
    if token.expires_at < now {
        return Err(AppError::InvalidRefreshToken("token expired".into()));
    }

    // Step 6 : vérifier user actif et récupérer le rôle actuel
    let user = match users::find_by_id(&state.pool, token.user_id).await? {
        Some(u) => u,
        None => {
            tracing::warn!(user_id = token.user_id, "refresh for deleted user");
            return Err(AppError::InvalidRefreshToken("user deleted".into()));
        }
    };

    if !user.active {
        tracing::warn!(user_id = user.id, "refresh for inactive user");
        return Err(AppError::InvalidRefreshToken("user inactive".into()));
    }

    // Step 8 : révoquer l'ancien token (rotation)
    refresh_tokens::revoke_by_token(&state.pool, &req.refresh_token, "rotation").await?;

    // Step 9 : créer nouveau refresh_token + JWT
    let new_refresh_token = uuid::Uuid::new_v4().to_string();
    let new_expires_at = (Utc::now() + state.config.refresh_inactivity).naive_utc();

    refresh_tokens::create(
        &state.pool,
        NewRefreshToken {
            user_id: user.id,
            token: new_refresh_token.clone(),
            expires_at: new_expires_at,
        },
    )
    .await?;

    let access_token = jwt::encode(
        user.id,
        user.role,
        state.config.jwt_secret_bytes(),
        state.config.jwt_expiry,
    )?;

    tracing::info!(user_id = user.id, "refresh successful");

    Ok(Json(RefreshResponse {
        access_token,
        refresh_token: new_refresh_token,
        expires_in: state.config.jwt_expiry.num_seconds(),
    }))
}

// --- Change password ---

/// Corps de `PUT /api/v1/auth/password`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

impl std::fmt::Debug for ChangePasswordRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChangePasswordRequest")
            .field("current_password", &"***")
            .field("new_password", &"***")
            .finish()
    }
}

/// `PUT /api/v1/auth/password`
///
/// Changement de mot de passe par l'utilisateur authentifié.
/// Séquence : verify current → hash new → update DB → revoke all → new tokens.
pub async fn change_password(
    State(state): State<AppState>,
    axum::Extension(current_user): axum::Extension<CurrentUser>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<RefreshResponse>, AppError> {
    // Validation du nouveau mot de passe (politique configurable — story 1.7)
    password::validate_password(&req.new_password, state.config.password_min_length)?;

    // Charger le user pour vérifier le mot de passe courant
    let user = match users::find_by_id(&state.pool, current_user.user_id).await? {
        Some(u) => u,
        None => return Err(AppError::Internal("user not found after auth".into())),
    };

    // Step 1 : vérifier le mot de passe courant (timing-safe)
    // Pas de dummy_verify() ici : verify_password_async a déjà consommé
    // ~50ms d'Argon2 — un dummy_verify en plus doublerait le temps
    // d'erreur et créerait un timing side-channel inversé.
    let plain = req.current_password.clone();
    let phc = user.password_hash.clone();
    if !password::verify_password_async(plain, phc).await? {
        return Err(AppError::InvalidCredentials);
    }

    // Step 2 : hasher le nouveau mot de passe
    let new_hash = password::hash_password_async(req.new_password.clone()).await?;

    // Step 3 : update en base
    users::update_password(&state.pool, user.id, &new_hash).await?;

    // Step 4 : révoquer tous les refresh_tokens
    let revoked_count =
        refresh_tokens::revoke_all_for_user(&state.pool, user.id, "password_change").await?;
    tracing::info!(
        user_id = user.id,
        revoked_count = revoked_count,
        "password changed"
    );

    // Step 5 : créer nouveau refresh_token + JWT
    let new_refresh_token = uuid::Uuid::new_v4().to_string();
    let new_expires_at = (Utc::now() + state.config.refresh_inactivity).naive_utc();

    refresh_tokens::create(
        &state.pool,
        NewRefreshToken {
            user_id: user.id,
            token: new_refresh_token.clone(),
            expires_at: new_expires_at,
        },
    )
    .await?;

    let access_token = jwt::encode(
        user.id,
        user.role,
        state.config.jwt_secret_bytes(),
        state.config.jwt_expiry,
    )?;

    Ok(Json(RefreshResponse {
        access_token,
        refresh_token: new_refresh_token,
        expires_in: state.config.jwt_expiry.num_seconds(),
    }))
}
