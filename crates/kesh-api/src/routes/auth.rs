//! Handlers HTTP pour `/api/v1/auth/login` et `/api/v1/auth/logout`.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::Utc;
use kesh_db::entities::NewRefreshToken;
use kesh_db::repositories::{refresh_tokens, users};
use serde::{Deserialize, Serialize};

use crate::auth::{jwt, password};
use crate::errors::AppError;
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
/// Flow :
/// 0. Validation : rejet si username/password vide (400).
/// 1. Lookup de l'utilisateur via `users::find_by_username`.
/// 2. Timing-attack mitigation : si user inconnu OU inactif, passer par
///    `dummy_verify` puis retourner 401 générique.
/// 3. Verify Argon2id ; si mismatch, 401 générique.
/// 4. Encode JWT HS256 avec claims `sub`, `role`, `iat`, `exp`.
/// 5. Génère un refresh_token UUID v4 et le persiste.
/// 6. Retourne `{accessToken, refreshToken, expiresIn}`.
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
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
            return Err(AppError::InvalidCredentials);
        }
    };

    // Step 3 : verify Argon2id
    if !password::verify_password(&req.password, &user.password_hash)? {
        return Err(AppError::InvalidCredentials);
    }

    // Step 4 : JWT
    let access_token = jwt::encode(
        user.id,
        user.role,
        state.config.jwt_secret_bytes(),
        state.config.jwt_expiry,
    )?;

    // Step 5 : refresh token
    let refresh_token = uuid::Uuid::new_v4().to_string();
    let expires_at = (Utc::now() + state.config.refresh_token_max_lifetime).naive_utc();

    refresh_tokens::create(
        &state.pool,
        NewRefreshToken {
            user_id: user.id,
            token: refresh_token.clone(),
            expires_at,
        },
    )
    .await?;

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
    let revoked = refresh_tokens::revoke_by_token(&state.pool, &req.refresh_token).await?;
    tracing::info!(revoked = revoked, "logout");
    Ok(StatusCode::NO_CONTENT)
}
