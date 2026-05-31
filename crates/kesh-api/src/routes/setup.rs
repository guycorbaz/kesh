//! Handler `POST /api/v1/setup/admin` — création self-service du 1er admin.
//!
//! **Story v011-5** : route publique sans `require_auth`, gated par
//! `user_count == 0` (auto-disable au 1er succès → 410 Gone). Au boot DB
//! vide + pas d'env, le bootstrap crée uniquement la company stub ; cette
//! route prend le relais pour créer l'admin via un formulaire web (cohérent
//! Jellyfin/Bitwarden/Sonarr first-run experience).
//!
//! Flow :
//! 1. Rate-limit IP (5/15min, cohérent `/auth/login`, quota partagé).
//! 2. Validation : username trim non-vide ; password ≥ `KESH_PASSWORD_MIN_LENGTH` (12).
//! 3. Gate `user_count > 0` → 410 SETUP_ALREADY_COMPLETE.
//! 4. Hash Argon2id.
//! 5. INSERT user `Admin` attaché à la company stub existante.
//! 6. Set cookies HttpOnly (réutilise `build_auth_cookies` pub(crate)).
//! 7. Set `state.users_exist = true` (Release) → désactive le gate 423.
//! 8. Retourne `LoginResponse` (cohérent /login pour fluidité du frontend).
//!
//! **Sécurité** :
//! - Race TOCTOU 2 usernames distincts : limitation v0.1 L1 (rate-limit IP +
//!   auto-disable réduisent la fenêtre d'exploitation). Issue v0.2.
//! - CSRF : pas de token (endpoint accepte uniquement `Content-Type: application/json`,
//!   pas de cookie de session présent au moment de l'appel).
//! - MITM : v0.1 ne force pas HTTPS (reverse proxy externe en charge).

use std::net::SocketAddr;
use std::sync::atomic::Ordering;

use axum::Json;
use axum::extract::{ConnectInfo, State};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use kesh_db::entities::{NewRefreshToken, NewUser, Role};
use kesh_db::errors::DbError;
use kesh_db::repositories::{refresh_tokens, users};
use serde::Deserialize;

use crate::AppState;
use crate::auth::{jwt, password};
use crate::errors::AppError;
use crate::routes::auth::{LoginResponse, build_auth_cookies};

/// Corps de `POST /api/v1/setup/admin`.
///
/// `Debug` manuel : masque le `password`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupAdminRequest {
    pub username: String,
    pub password: String,
}

impl std::fmt::Debug for SetupAdminRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SetupAdminRequest")
            .field("username", &self.username)
            .field("password", &"***")
            .finish()
    }
}

/// `POST /api/v1/setup/admin`
pub async fn create_admin(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    jar: CookieJar,
    Json(req): Json<SetupAdminRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), AppError> {
    // Step 0 — rate-limit IP (cohérent /login, quota partagé via state.rate_limiter).
    let ip = addr.ip();
    if let Err(reject) = state.rate_limiter.check_rate_limit(ip) {
        tracing::warn!(ip = %ip, "setup: rate limit triggered");
        return Err(AppError::RateLimited {
            retry_after: reject.retry_after_secs,
        });
    }

    // Step 1 — validation (avant lecture DB pour minimiser le coût).
    let username = req.username.trim().to_string();
    if username.is_empty() {
        state.rate_limiter.record_failed_attempt(ip);
        return Err(AppError::Validation("username must be non-empty".into()));
    }
    // CR Pass 1 ECH1-2 — réutiliser `validate_password` (story 1.7) qui couvre :
    // empty, whitespace-only, length < min. Évite que `"            "` (12 espaces)
    // passe `chars().count() >= 12` et se retrouve hashé. Cohérent
    // `auth.rs::change_password` qui appelle déjà `validate_password`.
    if let Err(e) = password::validate_password(&req.password, state.config.password_min_length) {
        state.rate_limiter.record_failed_attempt(ip);
        return Err(e);
    }
    if req.password.eq_ignore_ascii_case("changeme") {
        state.rate_limiter.record_failed_attempt(ip);
        return Err(AppError::Validation(
            "password 'changeme' is forbidden (placeholder)".into(),
        ));
    }

    // Step 2 — gate auto-disable. Lecture user_count fraîche (pas le cache
    // `users_exist` mémoire — cette route doit observer la DB réelle pour
    // éviter une race au boot tandis que le cache n'est pas encore set).
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("setup user count: {e}")))?;
    if user_count > 0 {
        // CR Pass 1 BH1-2 — `record_failed_attempt` AVANT 410 (cohérent AC #8
        // qui demande explicitement le décompte sur user_count > 0). Un attacker
        // qui spam ce path après setup sera rate-limited. Synchronise aussi le
        // cache mémoire (defense-in-depth si le bootstrap cas 1 a tourné sur une
        // DB non-vide — improbable, mais cohérent invariant).
        state.rate_limiter.record_failed_attempt(ip);
        state.users_exist.store(true, Ordering::Release);
        return Err(AppError::SetupAlreadyComplete);
    }

    // Step 3 — récupérer la company stub (créée par bootstrap cas 1).
    let company_id: i64 = match sqlx::query_scalar("SELECT id FROM companies ORDER BY id LIMIT 1")
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("setup get stub company: {e}")))?
    {
        Some(id) => id,
        None => {
            tracing::error!(
                "setup: aucune company stub trouvée — bootstrap a échoué silencieusement"
            );
            return Err(AppError::Internal(
                "company stub introuvable au setup".into(),
            ));
        }
    };

    // Step 4 — hash Argon2id.
    let hash = password::hash_password_async(req.password.clone()).await?;

    // Step 5 — INSERT user Admin.
    //
    // CR Pass 1 BH1-1 — gérer `UniqueConstraintViolation` explicitement : si une
    // requête concurrente avec le même username a réussi entre notre SELECT
    // user_count (line 100) et notre INSERT, le 2e INSERT échoue. Le user existe
    // → l'état logique est « setup déjà complété », pas un conflit de ressource.
    // On flip aussi `users_exist=true` car la contrainte UNIQUE garantit qu'au
    // moins un user existe en DB. Retourner 410 cohérent avec la 1ère gate AC #8.
    let user = match users::create(
        &state.pool,
        NewUser {
            username: username.clone(),
            password_hash: hash,
            role: Role::Admin,
            active: true,
            company_id,
        },
    )
    .await
    {
        Ok(u) => u,
        Err(DbError::UniqueConstraintViolation(_)) => {
            state.users_exist.store(true, Ordering::Release);
            state.rate_limiter.record_failed_attempt(ip);
            tracing::info!(
                "setup: UniqueConstraintViolation sur INSERT user — race concurrente, setup déjà complété"
            );
            return Err(AppError::SetupAlreadyComplete);
        }
        Err(e) => return Err(AppError::Database(e)),
    };

    tracing::info!(
        user_id = user.id,
        username = %user.username,
        "setup: 1er admin créé via /setup"
    );

    // CR Pass 1 ECH1-1 — flip `users_exist` IMMÉDIATEMENT après l'INSERT user
    // réussi, AVANT `refresh_tokens::create`. Si le 2e INSERT (refresh token)
    // échoue (transient DB error, pool exhaustion), le user est déjà commité
    // mais sans cache mémoire à jour → toutes les routes protégées retournent
    // 423 jusqu'au prochain restart et `/setup/admin` retourne 410 → app
    // verrouillée. Le store immédiat heal le state en mémoire avec la réalité DB.
    state.users_exist.store(true, Ordering::Release);

    // Step 6 — JWT + refresh token (cohérent /login).
    let access_token = jwt::encode(
        user.id,
        user.role,
        user.company_id,
        state.config.jwt_secret_bytes(),
        state.config.jwt_expiry,
    )?;
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

    // Reset rate-limit pour cette IP (cohérent /login).
    state.rate_limiter.reset(ip);

    // Step 8 — cookies HttpOnly (réutilise le helper auth.rs).
    let (access_cookie, refresh_cookie) = build_auth_cookies(&state, &access_token, &refresh_token);
    let jar = jar.add(access_cookie).add(refresh_cookie);

    Ok((
        jar,
        Json(LoginResponse {
            access_token,
            refresh_token,
            expires_in: state.config.jwt_expiry.num_seconds(),
            user_id: user.id,
            username: user.username,
            role: format!("{:?}", user.role),
        }),
    ))
}
