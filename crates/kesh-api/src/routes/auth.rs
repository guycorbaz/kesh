//! Handlers HTTP pour `/api/v1/auth/*`.

use std::net::SocketAddr;

use axum::Json;
use axum::extract::{ConnectInfo, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::{NaiveDateTime, SubsecRound, TimeDelta, Utc};
use kesh_db::entities::{NewAuditLogEntry, NewRefreshToken};
use kesh_db::repositories::{
    audit_log, installation_settings, password_reset_tokens, refresh_tokens, users,
};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::{jwt, password};
use crate::errors::AppError;
use crate::extractors::impl_validation_extractor;
use crate::mail;
use crate::middleware::auth::CurrentUser;
use crate::routes::api_keys::ensure_not_pat;

/// Story v0172-session (0.17.3, #876, C-v0172s-32) — l'échéance d'inactivité
/// d'une session qui commence ou reprend par un geste de l'utilisateur
/// (connexion, assistant, changement de mot de passe) : `now + N`, N lu en
/// base (`installation_settings`), tronquée à la milliseconde comme la colonne
/// `refresh_tokens.expires_at` — si bien que `idle_exp` (secondes, arrondi
/// vers le bas) se relit à l'identique de la ligne.
pub(crate) struct SessionDeadline {
    /// À écrire dans `refresh_tokens.expires_at`.
    pub expires_at: NaiveDateTime,
    /// La même échéance en secondes Unix, pour la revendication `idle_exp`.
    pub idle_exp: i64,
    /// La durée N (minutes) en vigueur.
    pub minutes: i32,
}

/// Lit N et calcule l'échéance d'une session neuve (cf. [`SessionDeadline`]).
pub(crate) async fn new_session_deadline(state: &AppState) -> Result<SessionDeadline, AppError> {
    let minutes = installation_settings::get(&state.pool)
        .await?
        .session_idle_minutes;
    let expires_at = (Utc::now() + TimeDelta::minutes(i64::from(minutes)))
        .naive_utc()
        .trunc_subsecs(3);
    Ok(SessionDeadline {
        expires_at,
        idle_exp: expires_at.and_utc().timestamp(),
        minutes,
    })
}

/// Secondes restantes avant `idle_exp` (0 si passée).
pub(crate) fn idle_expires_in(idle_exp: i64) -> i64 {
    idle_exp.saturating_sub(Utc::now().timestamp()).max(0)
}

/// Story 10-5 — construit les 2 cookies HttpOnly d'auth (access + refresh).
///
/// Centralise la création pour login/refresh/change_password (le seul endroit
/// où la logique de scope/durée diverge est logout qui émet des cookies expirés
/// via `build_expired_auth_cookies`).
///
/// - `access` : `Path=/`, `Max-Age=jwt_expiry` (typiquement 15 min)
/// - `refresh` : `Path=/api/v1/auth`, `Max-Age=refresh_token_max_lifetime` (30 jours hard ceiling browser
///   — distinct de l'échéance d'inactivité de la ligne, `expires_at` en base,
///   cf. Pass 3 F-COOKIE-LIFETIME-P3-1 et Story v0172-session)
/// - `.secure(state.config.cookie_secure)` : hotfix v0.1.3 (Issue #136) — par
///   défaut `true` (sécurité maximale, requiert HTTPS côté client). Override
///   via `KESH_COOKIE_SECURE=false` pour les déploiements LAN strict HTTP-only
///   (e.g. domaine RFC 8375 `*.home.arpa` derrière un Traefik en HTTP). Avant
///   v0.1.3 cette logique était `!test_mode` (couplée au mode test E2E), ce
///   qui rendait impossible un déploiement HTTP-only prod sans activer
///   simultanément les endpoints `/api/v1/_test/*` (footgun sécu).
///
/// **Story v011-5** — promu `pub(crate)` pour réutilisation par
/// `routes::setup::create_admin` (le 1er admin créé via web reçoit les mêmes
/// cookies HttpOnly que post-`/login`). Sans cette promotion, le helper
/// serait dupliqué dans setup.rs → risque de dérive sécu (cookies
/// HttpOnly/SameSite=Strict).
pub(crate) fn build_auth_cookies(
    state: &AppState,
    access_token: &str,
    refresh_token: &str,
) -> (Cookie<'static>, Cookie<'static>) {
    let access = build_access_cookie(state, access_token, state.config.jwt_expiry.num_seconds());

    let refresh = Cookie::build(("kesh_refresh_token", refresh_token.to_string()))
        .http_only(true)
        .secure(state.config.cookie_secure)
        .same_site(SameSite::Strict)
        .path("/api/v1/auth")
        .max_age(time::Duration::seconds(
            state.config.refresh_token_max_lifetime.num_seconds(),
        ))
        .build();

    (access, refresh)
}

/// Le cookie d'accès seul, d'âge `max_age_secs` — partagé par
/// [`build_auth_cookies`] et le battement d'activité, qui réémet le jeton
/// d'accès au même `exp` (cookie d'âge `exp − now`).
fn build_access_cookie(state: &AppState, access_token: &str, max_age_secs: i64) -> Cookie<'static> {
    Cookie::build(("kesh_access_token", access_token.to_string()))
        .http_only(true)
        .secure(state.config.cookie_secure)
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(time::Duration::seconds(max_age_secs))
        .build()
}

/// Story 10-5 — construit les 2 cookies d'auth expirés (Max-Age=0) pour logout.
/// Browser purge immédiatement à la réception.
///
/// v0.1.3 hotfix — `secure` lit `state.config.cookie_secure` cohérent
/// avec `build_auth_cookies` pour la symétrie. Sans ce match, un browser en
/// HTTP-only LAN ne reconnaîtrait pas les cookies expirés (Secure mismatch)
/// et garderait les cookies de session jusqu'à expiration max-age naturelle.
fn build_expired_auth_cookies(state: &AppState) -> (Cookie<'static>, Cookie<'static>) {
    let access = Cookie::build(("kesh_access_token", ""))
        .http_only(true)
        .secure(state.config.cookie_secure)
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(time::Duration::seconds(0))
        .build();

    let refresh = Cookie::build(("kesh_refresh_token", ""))
        .http_only(true)
        .secure(state.config.cookie_secure)
        .same_site(SameSite::Strict)
        .path("/api/v1/auth")
        .max_age(time::Duration::seconds(0))
        .build();

    (access, refresh)
}

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
///
/// Story 10-5 (D1 + D6 actés) : `access_token`/`refresh_token` conservés en
/// body pour rétro-compat des 19+ tests `*_e2e.rs` historiques (D1 Option A).
/// `user_id`/`username`/`role` ajoutés (D6) pour permettre au frontend de set
/// directement `_currentUser` sans round-trip `/me` post-login. Les cookies
/// `kesh_access_token`/`kesh_refresh_token` sont émis en parallèle via
/// `Set-Cookie` headers (cf. `build_auth_cookies`).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub user_id: i64,
    pub username: String,
    pub role: String,
    /// Story v0172-session (#876) — la durée d'inactivité N (minutes).
    pub idle_timeout_minutes: i32,
    /// Story v0172-session (#876) — secondes restantes avant l'échéance
    /// d'inactivité de la session (`N × 60` à la connexion).
    pub idle_expires_in: Option<i64>,
}

/// Corps de `POST /api/v1/auth/logout`.
///
/// Story 10-5 (T2.3) : `refresh_token` rendu optionnel — le handler logout
/// lit le token en priorité depuis le cookie `kesh_refresh_token` (Path=/api/v1/auth)
/// et fallback sur le body si le cookie est absent (rétro-compat 3 tests
/// `auth_e2e.rs:606,647,677` qui envoient `refreshToken` dans body JSON).
///
/// `Debug` manuel : masque le `refresh_token`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogoutRequest {
    pub refresh_token: Option<String>,
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
    jar: CookieJar,
    Json(req): Json<LoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), AppError> {
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

    // Step 4 : JWT (Story 6.2: include company_id) — Story v0172-session :
    // l'échéance d'inactivité de la session (`now + N`) est écrite dans la
    // ligne de renouvellement et copiée dans le jeton (`idle_exp`).
    let deadline = new_session_deadline(&state).await?;
    let access_token = jwt::encode_session(
        user.id,
        user.role,
        user.company_id,
        state.config.jwt_secret_bytes(),
        state.config.jwt_expiry,
        deadline.idle_exp,
    )?;

    // Step 5 : refresh token — sa ligne porte l'échéance d'inactivité.
    let refresh_token = uuid::Uuid::new_v4().to_string();

    refresh_tokens::create(
        &state.pool,
        NewRefreshToken {
            user_id: user.id,
            token: refresh_token.clone(),
            expires_at: deadline.expires_at,
        },
    )
    .await?;

    // Rate limiter : reset après succès
    state.rate_limiter.reset(ip);

    tracing::info!(user_id = user.id, "login success");

    // Story 10-5 — émettre les 2 cookies HttpOnly (access + refresh).
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
            idle_timeout_minutes: deadline.minutes,
            idle_expires_in: Some(idle_expires_in(deadline.idle_exp)),
        }),
    ))
}

/// `POST /api/v1/auth/logout`
///
/// Invalide le refresh_token en base (`revoked_at = NOW()`). Idempotent :
/// un token déjà révoqué, expiré ou inexistant retourne aussi 204.
/// N'exige PAS de JWT valide — un client avec un access_token expiré doit
/// pouvoir invalider sa session.
pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
    body: Option<Json<LogoutRequest>>,
) -> Result<impl IntoResponse, AppError> {
    // Story 10-5 (T2.3) : lire refresh_token en priorité depuis le cookie,
    // fallback sur le body (compat tests historiques `auth_e2e.rs:606,647,677`).
    let cookie_rt = jar.get("kesh_refresh_token").map(|c| c.value().to_string());
    let body_rt = body.and_then(|Json(req)| req.refresh_token);
    let rt = cookie_rt.or(body_rt);

    if let Some(rt) = rt {
        let revoked = refresh_tokens::revoke_by_token(&state.pool, &rt, "logout").await?;
        tracing::info!(revoked = revoked, "logout");
    } else {
        tracing::info!("logout without token (no cookie, no body)");
    }

    // Émettre cookies expirés (Max-Age=0) pour invalidation browser immédiate.
    let (access_expired, refresh_expired) = build_expired_auth_cookies(&state);
    let jar = jar.add(access_expired).add(refresh_expired);

    Ok((jar, StatusCode::NO_CONTENT))
}

// === Story 1.6 : refresh & change_password ===

/// Corps de la requête `POST /api/v1/auth/refresh`.
///
/// Story 10-5 (T2.2) : `refresh_token` rendu optionnel — le handler refresh
/// lit le token en priorité depuis le cookie `kesh_refresh_token` et
/// fallback sur le body si le cookie est absent (rétro-compat tests historiques).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRequest {
    pub refresh_token: Option<String>,
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
    /// Story v0172-session (#876) — la durée d'inactivité N (minutes).
    pub idle_timeout_minutes: i32,
    /// Story v0172-session (#876) — secondes restantes avant l'échéance
    /// d'inactivité de la session.
    pub idle_expires_in: Option<i64>,
}

/// Story v0172-session (#876, C-v0172s-17) — fenêtre de grâce d'une rotation :
/// un jeton tourné depuis moins de 30 secondes, présenté de nouveau (deux
/// onglets qui renouvellent ensemble), rend 401 sans révocation générale.
const ROTATION_GRACE_SECS: i64 = 30;

/// `POST /api/v1/auth/refresh`
///
/// Token rotation avec détection de vol (OWASP best practice).
/// Séquence complète documentée dans story 1.6, Task 3.3.
pub async fn refresh(
    State(state): State<AppState>,
    jar: CookieJar,
    body: Option<Json<RefreshRequest>>,
) -> Result<(CookieJar, Json<RefreshResponse>), AppError> {
    // Story 10-5 (T2.2) : lire refresh_token cookie-first, fallback body.
    let cookie_rt = jar.get("kesh_refresh_token").map(|c| c.value().to_string());
    let body_rt = body.and_then(|Json(req)| req.refresh_token);
    let refresh_token_input = cookie_rt.or(body_rt).ok_or_else(|| {
        AppError::InvalidRefreshToken("missing refresh_token (no cookie, no body)".into())
    })?;

    // Step 1 : chercher le token (actif OU révoqué)
    let token_opt =
        refresh_tokens::find_by_token_include_revoked(&state.pool, &refresh_token_input).await?;

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
            // Story v0172-session (#876, C-v0172s-17) — grâce de rotation :
            // deux onglets qui renouvellent avec le même jeton à quelques
            // secondes d'intervalle. Mesurée à l'horloge de la base, celle
            // qui a écrit `revoked_at`.
            if refresh_tokens::was_rotated_within(
                &state.pool,
                &refresh_token_input,
                ROTATION_GRACE_SECS,
            )
            .await?
            {
                tracing::info!(
                    user_id = token.user_id,
                    "refresh with a just-rotated token (grace), no mass revoke"
                );
                return Err(AppError::InvalidRefreshToken("token just rotated".into()));
            }
            // Détection de vol : mass revoke tous les tokens de l'utilisateur
            let revoked_count =
                refresh_tokens::revoke_all_for_user(&state.pool, token.user_id, "theft_detected")
                    .await?;
            tracing::warn!(
                user_id = token.user_id,
                revoked_count = revoked_count,
                "token replay detected, revoking all sessions"
            );
            return Err(AppError::InvalidRefreshToken(
                "token replay detected".into(),
            ));
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
    refresh_tokens::revoke_by_token(&state.pool, &refresh_token_input, "rotation").await?;

    // Step 9 : créer nouveau refresh_token + JWT — Story v0172-session (#876,
    // C-v0172s-32) : le renouvellement n'est PAS une activité, il ne prolonge
    // rien. La ligne neuve RECOPIE l'échéance d'inactivité de l'ancienne, et le
    // jeton neuf la porte (`idle_exp`) ; seul le battement la prolonge.
    let new_refresh_token = uuid::Uuid::new_v4().to_string();
    let idle_exp = token.expires_at.and_utc().timestamp();

    refresh_tokens::create(
        &state.pool,
        NewRefreshToken {
            user_id: user.id,
            token: new_refresh_token.clone(),
            expires_at: token.expires_at,
        },
    )
    .await?;

    let access_token = jwt::encode_session(
        user.id,
        user.role,
        user.company_id,
        state.config.jwt_secret_bytes(),
        state.config.jwt_expiry,
        idle_exp,
    )?;
    let idle_minutes = installation_settings::get(&state.pool)
        .await?
        .session_idle_minutes;

    tracing::info!(user_id = user.id, "refresh successful");

    // Story 10-5 — émettre 2 nouveaux cookies (rotation access + refresh).
    let (access_cookie, refresh_cookie) =
        build_auth_cookies(&state, &access_token, &new_refresh_token);
    let jar = jar.add(access_cookie).add(refresh_cookie);

    Ok((
        jar,
        Json(RefreshResponse {
            access_token,
            refresh_token: new_refresh_token,
            expires_in: state.config.jwt_expiry.num_seconds(),
            idle_timeout_minutes: idle_minutes,
            idle_expires_in: Some(idle_expires_in(idle_exp)),
        }),
    ))
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
    jar: CookieJar,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<(CookieJar, Json<RefreshResponse>), AppError> {
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

    // Step 5 : créer nouveau refresh_token + JWT — Story v0172-session : un
    // geste de l'utilisateur, la session repart à `now + N`.
    let new_refresh_token = uuid::Uuid::new_v4().to_string();
    let deadline = new_session_deadline(&state).await?;

    refresh_tokens::create(
        &state.pool,
        NewRefreshToken {
            user_id: user.id,
            token: new_refresh_token.clone(),
            expires_at: deadline.expires_at,
        },
    )
    .await?;

    let access_token = jwt::encode_session(
        user.id,
        user.role,
        user.company_id,
        state.config.jwt_secret_bytes(),
        state.config.jwt_expiry,
        deadline.idle_exp,
    )?;

    // Story 10-5 (T2.4 Pass 3 F-EXPIRES-IN-CHANGE-PASSWORD-P3-15) — émettre
    // les 2 nouveaux cookies (rotation post-change_password) pour ne pas
    // laisser le user sur l'ancien cookie expiré qui causerait une
    // déconnexion silencieuse au prochain refresh.
    let (access_cookie, refresh_cookie) =
        build_auth_cookies(&state, &access_token, &new_refresh_token);
    let jar = jar.add(access_cookie).add(refresh_cookie);

    Ok((
        jar,
        Json(RefreshResponse {
            access_token,
            refresh_token: new_refresh_token,
            expires_in: state.config.jwt_expiry.num_seconds(),
            idle_timeout_minutes: deadline.minutes,
            idle_expires_in: Some(idle_expires_in(deadline.idle_exp)),
        }),
    ))
}

// --- Story v0172-session (0.17.3, #876) : le battement d'activité ---

/// Corps de `POST /api/v1/auth/activity` : l'âge de la dernière activité de
/// l'utilisateur au moment de l'envoi, mesuré par le navigateur.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityRequest {
    /// Millisecondes écoulées depuis la dernière activité (entier ≥ 0).
    pub idle_ms: u64,
}

impl_validation_extractor!(
    ActivityBody,
    ActivityRequest,
    "idleMs : entier positif ou nul (millisecondes)"
);

/// Réponse du battement : N et les secondes restantes avant l'échéance.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityResponse {
    pub idle_timeout_minutes: i32,
    pub idle_expires_in: i64,
}

/// `POST /api/v1/auth/activity` — Story v0172-session (#876, C-v0172s-33).
///
/// Prolonge l'échéance d'inactivité de **cette** session (la ligne que
/// désigne le cookie de renouvellement, reçu puisque la route est sous
/// `/api/v1/auth/`) : `échéance = now − âge + N`, l'âge étant borné à
/// `[0, N]`. Une requête, une écriture ; aucune rotation, aucun jeton de
/// renouvellement émis — deux onglets qui battent ensemble ne déclenchent rien.
///
/// Le jeton d'accès est réémis avec les mêmes `sub`, `role`, `company_id` et
/// **le même `exp`** que celui présenté (le battement ne prolonge ni la
/// fenêtre d'un rôle périmé, ni le rythme des renouvellements), et le nouvel
/// `idle_exp`. Si `exp` est déjà passé (jeton présenté dans la tolérance de
/// 60 s), aucun cookie d'accès n'est posé — un `Max-Age` nul le supprimerait —
/// et le prochain appel passera par le renouvellement.
///
/// Refus : clé d'API ⇒ 403 ; cookie absent, ligne d'un autre utilisateur,
/// révoquée ou échue ⇒ 401 ; corps illisible ⇒ 400 `VALIDATION_ERROR`.
///
/// L'ordre est celui-là, corps compris (revue de code P1, S-B4/S-E8) : le corps
/// est extrait en `Result` et ne se juge **qu'après** `ensure_not_pat`, si
/// bien qu'une clé d'API reçoit 403 même avec un corps illisible.
pub async fn activity(
    State(state): State<AppState>,
    axum::Extension(current_user): axum::Extension<CurrentUser>,
    jar: CookieJar,
    body: Result<ActivityBody, axum::response::Response>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    if let Err(e) = ensure_not_pat(&current_user) {
        return e.into_response();
    }
    let ActivityBody(req) = match body {
        Ok(b) => b,
        Err(rejet) => return rejet,
    };
    activity_with(state, current_user, jar, req)
        .await
        .into_response()
}

/// Le battement, une fois la clé d'API écartée et le corps lu ([`activity`]).
async fn activity_with(
    state: AppState,
    current_user: CurrentUser,
    jar: CookieJar,
    req: ActivityRequest,
) -> Result<(CookieJar, Json<ActivityResponse>), AppError> {
    let refresh_token = jar
        .get("kesh_refresh_token")
        .map(|c| c.value().to_string())
        .ok_or_else(|| AppError::Unauthenticated("activity: missing refresh cookie".into()))?;

    let minutes = installation_settings::get(&state.pool)
        .await?
        .session_idle_minutes;
    let window_ms = u64::try_from(minutes).unwrap_or(0) * 60_000;
    let idle_ms = req.idle_ms.min(window_ms);
    let now = Utc::now();
    let deadline = (now - TimeDelta::milliseconds(i64::try_from(idle_ms).unwrap_or(i64::MAX))
        + TimeDelta::minutes(i64::from(minutes)))
    .naive_utc()
    .trunc_subsecs(3);

    let written = refresh_tokens::set_idle_deadline(
        &state.pool,
        &refresh_token,
        current_user.user_id,
        deadline,
        now.naive_utc(),
    )
    .await?;
    if !written {
        return Err(AppError::Unauthenticated(
            "activity: no live session for this refresh cookie".into(),
        ));
    }
    let idle_exp = deadline.and_utc().timestamp();

    let remaining = current_user.exp.saturating_sub(now.timestamp());
    let jar = if remaining > 0 {
        let token = jwt::encode_session_claims(
            &jwt::SessionClaims {
                claims: jwt::Claims {
                    sub: current_user.user_id.to_string(),
                    role: current_user.role.as_str().to_owned(),
                    company_id: current_user.company_id,
                    iat: now.timestamp(),
                    exp: current_user.exp,
                },
                idle_exp: Some(idle_exp),
            },
            state.config.jwt_secret_bytes(),
        )?;
        jar.add(build_access_cookie(&state, &token, remaining))
    } else {
        jar
    };

    Ok((
        jar,
        Json(ActivityResponse {
            idle_timeout_minutes: minutes,
            idle_expires_in: idle_expires_in(idle_exp),
        }),
    ))
}

// === Story 10-5 — GET /api/v1/auth/me ===

/// Réponse de `GET /api/v1/auth/me`.
///
/// Story 10-5 (T4.1) : permet au frontend de restaurer l'état utilisateur
/// (`authState.currentUser`) au boot via `hydrate()` sans avoir besoin de
/// décoder le JWT (qui est inaccessible côté JS, stocké en cookie HttpOnly).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeResponse {
    pub user_id: i64,
    pub username: String,
    pub role: String,
    /// Secondes restantes avant expiration du JWT (calculé depuis claims.exp).
    pub expires_in: i64,
    /// Story v0172-session (#876) — la durée d'inactivité N (minutes).
    pub idle_timeout_minutes: i32,
    /// Story v0172-session (#876) — secondes restantes avant l'échéance
    /// d'inactivité (`idle_exp` du jeton), `null` pour un jeton qui n'en porte
    /// pas (émis avant la 0.17.3).
    pub idle_expires_in: Option<i64>,
}

/// `GET /api/v1/auth/me`
///
/// Retourne l'identité de l'utilisateur authentifié (via cookie ou Bearer header,
/// résolu par `require_auth` middleware). Nécessaire post-Story 10-5 pour que le
/// frontend puisse restaurer son state auth après navigation/reload sans pouvoir
/// décoder le JWT (HttpOnly cookie inaccessible JS).
pub async fn me(
    State(state): State<AppState>,
    axum::Extension(current_user): axum::Extension<CurrentUser>,
) -> Result<Json<MeResponse>, AppError> {
    let user = users::find_by_id(&state.pool, current_user.user_id)
        .await?
        .ok_or_else(|| AppError::Internal("current user not found in DB".into()))?;

    let now_secs = chrono::Utc::now().timestamp();
    // CR Pass 3 ECH3-L3 : `saturating_sub` protège contre un `exp` théoriquement
    // négatif extrême (`i64::MIN`) qui produirait un panic en debug build sur la
    // soustraction native. `.max(0)` reste nécessaire pour clamper les expirations
    // déjà passées (cas normal d'un token expiré).
    let expires_in = current_user.exp.saturating_sub(now_secs).max(0);
    let idle_timeout_minutes = installation_settings::get(&state.pool)
        .await?
        .session_idle_minutes;

    Ok(Json(MeResponse {
        user_id: user.id,
        username: user.username,
        role: format!("{:?}", user.role),
        expires_in,
        idle_timeout_minutes,
        idle_expires_in: current_user.idle_exp.map(idle_expires_in),
    }))
}

// === Story 17-4c — Recovery self-service (forgot-password / reset-password) ===

/// Corps de `POST /api/v1/auth/forgot-password`.
///
/// `identifier` = username OU email (DC6 : un `@` aiguille vers `find_by_email`,
/// sinon `find_by_username`). Pas de secret ici, mais on garde un `Debug` sobre
/// par cohérence (l'identifiant n'est pas confidentiel, on l'affiche en clair).
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ForgotPasswordRequest {
    pub identifier: String,
}

/// Corps de `POST /api/v1/auth/reset-password`.
///
/// `Debug` manuel : masque **`token`** ET **`new_password`** (calque
/// `ChangePasswordRequest`) — ne jamais leaker le secret ni le token brut via
/// `tracing::debug!("{:?}", req)`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

impl std::fmt::Debug for ResetPasswordRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResetPasswordRequest")
            .field("token", &"***")
            .field("new_password", &"***")
            .finish()
    }
}

/// Applique le rate-limit recovery (DC5) : check + record **atomiques** sous un
/// seul verrou ([`crate::middleware::rate_limit::RateLimiter::check_and_record`],
/// P8 — jamais `reset`).
///
/// Sémantique always-200 du flux forgot-password : le record inconditionnel est
/// le SEUL incrément du compteur — sans lui, le check serait inerte (catch P3
/// Opus F1). Chaque requête recovery (succès ou non) consomme un slot ; au 5e
/// slot dans la fenêtre, l'IP est bloquée 30 min.
fn enforce_recovery_rate_limit(state: &AppState, ip: std::net::IpAddr) -> Result<(), AppError> {
    if let Err(reject) = state.rate_limiter_recovery.check_and_record(ip) {
        tracing::warn!(ip = %ip, "recovery rate limit triggered");
        return Err(AppError::RateLimited {
            retry_after: reject.retry_after_secs,
        });
    }
    Ok(())
}

/// `POST /api/v1/auth/forgot-password` (public, gated `forgot_password_enabled`).
///
/// Anti-énumération (DC4) : retourne **TOUJOURS `200`** corps générique, que le
/// compte existe ou non. Flux :
/// 0. Rate-limit recovery (DC5, check + record inconditionnel).
/// 1. **TOUT** le travail dépendant de l'identifiant — lookup DC6 compris —
///    part dans une tâche détachée `tokio::spawn` (cf.
///    [`process_forgot_password_request`]). Le handler ne touche JAMAIS la DB :
///    la réponse `200` a un timing strictement constant (Pass 1 P1 avait
///    détaché les écritures ; Pass 2 BH2-M6 détache aussi le lookup, dont la
///    latence présent-vs-absent restait observable).
/// 2. Toujours `200`.
pub async fn forgot_password(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<StatusCode, AppError> {
    let ip = addr.ip();
    enforce_recovery_rate_limit(&state, ip)?;

    let identifier = req.identifier.trim().to_string();

    // Pass 2 ECH2-L1 — identifiant vide après trim : no-op silencieux (200
    // anti-énum), sans spawn ni DB. Évite `find_by_username("")` qui matcherait
    // un éventuel username vide legacy pathologique.
    if !identifier.is_empty() {
        let task_state = state.clone();
        tokio::spawn(async move {
            process_forgot_password_request(task_state, identifier).await;
        });
    }

    // DC4 — toujours `200`, aucun signal existant/inexistant.
    Ok(StatusCode::OK)
}

/// Lookup DC6 + traitement du match de `forgot_password`, en **tâche détachée**.
///
/// Pass 2 BH2-M6 : le lookup lui-même est ici (et plus dans le handler) — le
/// handler répond sans aucun accès DB, timing constant absolu. Les erreurs de
/// lookup sont loggées, jamais propagées (le 200 est déjà parti).
///
/// DC6 — un identifiant contenant `@` est traité comme un email (non-unique →
/// exactement 1 match actif requis), sinon comme un username (UNIQUE, ≤ 1).
/// Note (Pass 2 BH2-M3) : `find_by_username` ne filtre PAS `active` — un match
/// username inactif arrive jusqu'à [`process_forgot_password_match`], qui
/// l'audite avec `recoverable: false` et n'envoie rien. Côté email, le filtre
/// `retain(active)` s'applique AVANT le comptage « exactement 1 » (P4) pour
/// qu'un doublon désactivé ne prive pas le compte actif du recovery.
async fn process_forgot_password_request(state: AppState, identifier: String) {
    let (matched_user, identifier_kind) = if identifier.contains('@') {
        match users::find_by_email(&state.pool, &identifier).await {
            Ok(mut candidates) => {
                // P4 — seuls les comptes ACTIFS comptent pour « exactement 1 ».
                candidates.retain(|u| u.active);
                let user = if candidates.len() == 1 {
                    Some(candidates.remove(0))
                } else {
                    // Pass 3 BH3 (LOW) — > 1 actifs sur le même email : ces
                    // comptes sont privés du recovery self-service (anti-énum
                    // oblige, no-op client) ; signal SERVEUR pour le diagnostic
                    // admin. 0 actif → no-op normal, pas de log.
                    if candidates.len() > 1 {
                        tracing::warn!(
                            match_count = candidates.len(),
                            "forgot-password: email partagé par plusieurs comptes actifs — recovery self-service impossible pour ces comptes"
                        );
                    }
                    None
                };
                (user, "email")
            }
            Err(e) => {
                tracing::error!(error = %e, "forgot-password: échec lookup email (non-fatal)");
                return;
            }
        }
    } else {
        match users::find_by_username(&state.pool, &identifier).await {
            Ok(user) => (user, "username"),
            Err(e) => {
                tracing::error!(error = %e, "forgot-password: échec lookup username (non-fatal)");
                return;
            }
        }
    };

    if let Some(user) = matched_user {
        process_forgot_password_match(state, user, identifier_kind).await;
    }
}

/// Travail post-match de `forgot_password`, exécuté en **tâche détachée** (P1).
///
/// Tourne après que le `200` générique soit parti : aucune erreur n'est
/// propagée (loggée seulement), aucun timing n'est observable côté client (DC4).
/// Ordre :
/// 1. Audit `auth.password_reset_requested` — écrit pour **tout** match (AC12),
///    y compris non-recouvrable (inactif / sans email) : c'est précisément la
///    demande qu'un admin veut voir. `details_json.recoverable` porte l'issue.
/// 2. Si recouvrable (actif + email + `public_base_url`) : invalidation des
///    tokens pendants, création du nouveau token (hash seul, DC3), envoi email.
///
/// On ne logge JAMAIS `reset_url` ni le token brut — au plus `user_id`.
async fn process_forgot_password_match(
    state: AppState,
    user: kesh_db::entities::User,
    identifier_kind: &'static str,
) {
    let email = user.email.clone().filter(|e| !e.trim().is_empty());
    let recoverable = user.active && email.is_some();

    // 1. Audit du match — `NewAuditLogEntry::user` (pas de CurrentUser pré-auth).
    // Best-effort : un échec d'audit ne doit pas empêcher l'envoi du lien.
    match state.pool.begin().await {
        Ok(mut tx) => {
            let insert = audit_log::insert_in_tx(
                &mut tx,
                NewAuditLogEntry::user(
                    user.id,
                    "auth.password_reset_requested",
                    "user",
                    user.id,
                    Some(serde_json::json!({
                        "identifier_kind": identifier_kind,
                        "recoverable": recoverable,
                    })),
                ),
            )
            .await;
            match insert {
                Ok(_) => {
                    if let Err(e) = tx.commit().await {
                        tracing::error!(user_id = user.id, error = %e, "forgot-password: échec commit audit (non-fatal)");
                    }
                }
                Err(e) => {
                    tracing::error!(user_id = user.id, error = %e, "forgot-password: échec insert audit (non-fatal)");
                }
            }
        }
        Err(e) => {
            tracing::error!(user_id = user.id, error = %e, "forgot-password: échec begin tx audit (non-fatal)");
        }
    }

    // 2. Compte non-recouvrable par ce flux (fallback break-glass #121) — déjà
    // audité ci-dessus avec `recoverable: false`.
    if !user.active {
        tracing::info!(
            user_id = user.id,
            "forgot-password: compte inactif — non-recouvrable par ce flux"
        );
        return;
    }
    let Some(email) = email else {
        tracing::info!(
            user_id = user.id,
            "forgot-password: compte sans email — non-recouvrable par ce flux"
        );
        return;
    };
    // Résolution de la base AVANT toute création de token : ne jamais laisser en
    // DB un token impossible à envoyer. `public_base_url` est garanti `Some`
    // non-vide par le fail-fast boot quand le feature est activé — branche
    // défensive (pas d'`unwrap`).
    let Some(base) = state.config.public_base_url.clone() else {
        tracing::error!(
            user_id = user.id,
            "forgot-password: public_base_url absent malgré feature activé — envoi sauté"
        );
        return;
    };

    // DC3 — token base62 sans préfixe ; seul le SHA-256 est persisté.
    let (token_clear, token_hash) = crate::auth::api_key::generate_reset_token();
    let expires_at = (chrono::Utc::now()
        + chrono::TimeDelta::minutes(mail::PASSWORD_RESET_TTL_MINUTES))
    .naive_utc();

    // Anti-accumulation **best-effort** (P9) : invalidate + create ne sont pas
    // transactionnels — deux requêtes concurrentes peuvent laisser deux tokens
    // valides (les deux emails partent vers la même boîte du même user, impact
    // nul). L'invariant « au plus 1 token pendant » ne tient qu'en régime
    // non-concurrent ; un échec de purge ne casse pas le flux.
    if let Err(e) = password_reset_tokens::invalidate_all_for_user(&state.pool, user.id).await {
        tracing::warn!(user_id = user.id, error = %e, "forgot-password: échec invalidate tokens pendants (non-fatal)");
    }

    // Stocke le hash AVANT l'envoi (le token doit exister en DB quand
    // l'utilisateur clique le lien). Échec loggé seulement : pas d'email envoyé,
    // l'utilisateur retentera (anti-oracle 500, P1).
    if let Err(e) =
        password_reset_tokens::create(&state.pool, user.id, &token_hash, expires_at).await
    {
        tracing::error!(user_id = user.id, error = %e, "forgot-password: échec création token — pas d'email envoyé (non-fatal)");
        return;
    }

    let reset_url = format!("{base}/reset-password?token={token_clear}");
    if let Err(e) = state
        .mailer
        .send_password_reset(&email, &reset_url, state.config.locale)
        .await
    {
        tracing::error!(user_id = user.id, error = %e, "forgot-password: envoi email échoué (non-fatal)");
    }
}

/// Réponse minimale de `POST /api/v1/auth/reset-password`.
#[derive(Debug, Serialize)]
pub struct ResetPasswordResponse {
    pub status: &'static str,
}

/// `POST /api/v1/auth/reset-password` (public, gated `forgot_password_enabled`).
///
/// Flux :
/// 0. Rate-limit recovery (DC5, check + record inconditionnel).
/// 1. `sha256_hex(token.trim())` → `find_valid_by_hash`. Absent/expiré/utilisé →
///    `400 INVALID_OR_EXPIRED_TOKEN` générique (DC4, anti-fuite).
/// 2. Re-check `user.active` au moment de la consommation (P3) — même `400`.
/// 3. Valide la politique du nouveau mot de passe + hash Argon2id.
/// 4. **Transaction unique** (P2) : `mark_used_in_tx` (garde TOCTOU — `NotFound`
///    ⇒ même `400`) + `update_password_in_tx` + audit
///    `auth.password_reset_completed`. Un échec rollback TOUT : le token n'est
///    jamais brûlé sans que le mot de passe ait changé.
/// 5. Post-commit best-effort : `revoke_all_for_user(_, "password_change")`
///    (raison réutilisée, DC8/AC14 — pas de migration CHECK), loggé si échec.
/// 6. `200 { "status": "ok" }`.
pub async fn reset_password(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<Json<ResetPasswordResponse>, AppError> {
    let ip = addr.ip();
    enforce_recovery_rate_limit(&state, ip)?;

    // 1. Lookup par hash. Les trois cas (inconnu/expiré/utilisé) convergent vers
    // le même `400` générique (DC4). Trim (P6) : les clients mail wrappent les
    // URLs longues — un copier-coller peut embarquer espace/retour-ligne ;
    // l'alphabet base62 étant strictement alphanumérique, le trim est sans
    // ambiguïté.
    let token_hash = crate::auth::api_key::sha256_hex(req.token.trim());
    let token = password_reset_tokens::find_valid_by_hash(&state.pool, &token_hash)
        .await?
        .ok_or(AppError::InvalidOrExpiredToken)?;

    // 2. Re-check de l'état du compte AU MOMENT de la consommation (P3) : un
    // compte désactivé par un admin pendant le TTL du token (gel sécurité) ne
    // doit plus pouvoir muter son état. Même `400` générique (DC4, pas de fuite).
    users::find_by_id(&state.pool, token.user_id)
        .await?
        .filter(|u| u.active)
        .ok_or(AppError::InvalidOrExpiredToken)?;

    // 3. Validation politique mdp + hash (avant toute mutation).
    password::validate_password(&req.new_password, state.config.password_min_length)?;
    let new_hash = password::hash_password_async(req.new_password.clone()).await?;

    // 4. Transaction unique (P2) : mark_used_in_tx (garde TOCTOU `AND used_at IS
    // NULL`, identique sous verrou de ligne InnoDB) + update_password_in_tx
    // (garde rows_affected, bump version) + audit. Un retour anticipé droppe la
    // transaction → rollback : le token reste consommable si le changement de
    // mot de passe n'a pas abouti.
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("reset-password tx begin: {e}")))?;

    if let Err(e) = password_reset_tokens::mark_used_in_tx(&mut tx, token.id).await {
        return match e {
            // Course concurrente / double-consume → même `400` anti-fuite (DC4).
            kesh_db::errors::DbError::NotFound => Err(AppError::InvalidOrExpiredToken),
            other => Err(AppError::Database(other)),
        };
    }

    if let Err(e) = users::update_password_in_tx(&mut tx, token.user_id, &new_hash).await {
        return match e {
            // User supprimé entre le re-check et la tx (ultra-rare) → même `400`
            // générique, cohérent DC4.
            kesh_db::errors::DbError::NotFound => Err(AppError::InvalidOrExpiredToken),
            other => Err(AppError::Database(other)),
        };
    }

    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            token.user_id,
            "auth.password_reset_completed",
            "user",
            token.user_id,
            None,
        ),
    )
    .await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("reset-password tx commit: {e}")))?;

    // 4. Post-commit best-effort : révoque tous les refresh tokens du user. Hors
    // transaction par design (`revoke_all_for_user` prend `&MySqlPool`), pattern
    // identique à `change_password` et break-glass `bootstrap.rs`. Raison
    // `"password_change"` réutilisée (DC8/AC14 — le distinguo recovery est porté
    // par `audit_log.action`, pas par `revoked_reason`).
    match refresh_tokens::revoke_all_for_user(&state.pool, token.user_id, "password_change").await {
        Ok(count) => tracing::info!(
            user_id = token.user_id,
            revoked_count = count,
            "password reset completed"
        ),
        Err(e) => tracing::warn!(
            user_id = token.user_id,
            error = %e,
            "reset-password: échec revoke refresh tokens post-reset (non-fatal)"
        ),
    }

    Ok(Json(ResetPasswordResponse { status: "ok" }))
}
