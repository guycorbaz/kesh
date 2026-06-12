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
//! 3. Fetch company stub + hash Argon2id (hors transaction).
//! 4. Section critique atomique (Story 17-1) : `begin` → verrou sentinelle
//!    `_kesh_version id=1 FOR UPDATE` → re-check `user_count == 0` sous verrou
//!    (`> 0` → 410 SETUP_ALREADY_COMPLETE) → INSERT user `Admin` → `commit`.
//! 5. Set `state.users_exist = true` (Release) → désactive le gate 423.
//! 6. JWT + refresh token + reset rate-limit (post-commit).
//! 7. Set cookies HttpOnly (réutilise `build_auth_cookies` pub(crate)) et
//!    retourne `LoginResponse` (cohérent /login pour fluidité du frontend).
//!
//! **Sécurité** :
//! - Race TOCTOU 2 usernames distincts (issue #133, ex-limitation L1) : **fermée
//!   Story 17-1**. Le check+insert est sérialisé par un `SELECT _kesh_version
//!   id=1 FOR UPDATE` en tête de transaction → au plus 1 admin créé même sous
//!   requêtes concurrentes à usernames distincts. Rate-limit IP + auto-disable
//!   410 conservés en défense en profondeur.
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
    /// Email optionnel du 1er admin (Story 17-4a, recovery). Validé si non-vide.
    #[serde(default)]
    pub email: Option<String>,
}

impl std::fmt::Debug for SetupAdminRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SetupAdminRequest")
            .field("username", &self.username)
            .field("password", &"***")
            .field("email", &self.email)
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
    // Story 17-4c (P5, DC6) — `@` est réservé à l'aiguillage email du recovery
    // forgot-password : un username qui en contiendrait serait structurellement
    // non-recouvrable en self-service.
    if username.contains('@') {
        state.rate_limiter.record_failed_attempt(ip);
        return Err(AppError::Validation(
            "username must not contain '@' (reserved for email recovery routing)".into(),
        ));
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
    // Email optionnel (Story 17-4a) — validé/normalisé via le helper partagé.
    let email = match crate::routes::users::validate_optional_email(&state, req.email.clone()) {
        Ok(e) => e,
        Err(e) => {
            state.rate_limiter.record_failed_attempt(ip);
            return Err(e);
        }
    };

    // Step 2 — fetch de la company stub (créée par bootstrap cas 1). Lecture
    // hors transaction (non-locking) AVANT d'ouvrir la tx verrouillée : évite de
    // figer prématurément le snapshot MVCC de la section critique (F3) et garde
    // la tenue du verrou InnoDB minimale.
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

    // Step 3 — hash Argon2id (CPU coûteux). Réalisé HORS de la section
    // verrouillée pour minimiser la durée de tenue du verrou InnoDB ; ne dépend
    // que du password déjà validé (Step 1).
    let hash = password::hash_password_async(req.password.clone()).await?;

    // Step 4 — section critique atomique (Story 17-1, fix TOCTOU #133).
    //
    // Le check `user_count == 0` et l'INSERT du 1er admin s'exécutent dans UNE
    // seule transaction précédée d'un verrou sérialisant sur la row sentinelle
    // globale `_kesh_version id=1`. Deux requêtes concurrentes avec des usernames
    // distincts se sérialisent sur ce verrou → au plus 1 admin créé.
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("setup begin tx: {e}")))?;

    // 4a — verrou sentinelle EN PREMIÈRE INSTRUCTION de la tx (avant tout SELECT
    // non-locking — cf. snapshot MVCC REPEATABLE READ). `None` (row sentinelle
    // absente) → DbError::Invariant → AppError::Database → 500 (bug structurel
    // d'installation, migration 20260522000001 manquante).
    users::acquire_setup_sentinel_lock(&mut tx)
        .await
        .map_err(AppError::Database)?;

    // 4b — re-check `user_count` SOUS verrou. C'est le 1er read non-locking de la
    // tx → son snapshot MVCC est figé APRÈS le commit de toute tx concurrente
    // déjà sérialisée par le verrou ci-dessus (donc voit ses INSERT committés).
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(format!("setup user count: {e}")))?;
    if user_count > 0 {
        // Rollback explicite (relâche le verrou sentinelle) avant le 410.
        // record_failed_attempt + users_exist : comportement identique à l'ancien
        // gate hors-tx, désormais race-safe. Un échec de rollback est loggé (et
        // non propagé) : la connexion sera de toute façon recyclée par sqlx, ce
        // qui relâchera le verrou — mais on trace l'anomalie pour l'observabilité.
        if let Err(e) = tx.rollback().await {
            tracing::warn!(error = %e, "setup: rollback échoué (chemin 410 user_count>0)");
        }
        state.rate_limiter.record_failed_attempt(ip);
        state.users_exist.store(true, Ordering::Release);
        return Err(AppError::SetupAlreadyComplete);
    }

    // 4c — INSERT du 1er admin DANS la même tx via le variant transaction-aware
    // (`users::create` ouvrirait sa propre tx interne et casserait l'atomicité).
    //
    // Gestion `UniqueConstraintViolation` (défense en profondeur same-username) :
    // désormais redondante avec le verrou — la tx concurrente a déjà committé et
    // le re-check 4b l'aurait vue — mais conservée, inoffensive. État logique =
    // « setup déjà complété » → 410 + flip `users_exist`.
    let user = match users::create_in_tx(
        &mut tx,
        NewUser {
            username: username.clone(),
            password_hash: hash,
            role: Role::Admin,
            active: true,
            company_id,
            email,
        },
    )
    .await
    {
        Ok(u) => u,
        Err(DbError::UniqueConstraintViolation(_)) => {
            if let Err(e) = tx.rollback().await {
                tracing::warn!(error = %e, "setup: rollback échoué (chemin 410 UniqueConstraintViolation)");
            }
            state.users_exist.store(true, Ordering::Release);
            state.rate_limiter.record_failed_attempt(ip);
            tracing::info!(
                "setup: UniqueConstraintViolation sur INSERT user — race concurrente, setup déjà complété"
            );
            return Err(AppError::SetupAlreadyComplete);
        }
        // Toute autre erreur : le `return` droppe `tx` → rollback automatique sqlx.
        Err(e) => return Err(AppError::Database(e)),
    };

    // 4d — commit : valide check+insert ensemble et relâche le verrou sentinelle.
    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("setup commit tx: {e}")))?;

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
