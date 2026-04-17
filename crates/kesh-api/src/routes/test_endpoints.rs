//! Story 6.4 — endpoints de test `/api/v1/_test/*` gated par `KESH_TEST_MODE`.
//!
//! **⚠️ NEVER EXPOSE IN PRODUCTION** — ces endpoints permettent de truncate
//! la DB entière et de re-seeder avec un état déterministe. Le montage dans
//! [`crate::build_router`] est **conditionnel à `config.test_mode == true`**,
//! lui-même **incompatible avec un bind non-loopback** (refus de démarrage
//! via [`crate::config::ConfigError::TestModeWithPublicBind`]).
//!
//! Usage Playwright (`frontend/tests/e2e/helpers/test-state.ts`) :
//!
//! ```text
//! POST /api/v1/_test/seed { "preset": "with-company" }
//! POST /api/v1/_test/reset   (équivalent à seed { preset: "fresh" })
//! ```
//!
//! Presets :
//! - `fresh` : uniquement user `changeme/changeme` (AC #7)
//! - `post-onboarding` / `with-company` : state complet post-onboarding (AC #8/#9)
//! - `with-data` : `with-company` + contact + product (AC #10, pas de facture)
//!
//! **Concurrence** (code review P2) : un `tokio::sync::Mutex` statique
//! sérialise tous les seed/reset server-side. Sans ça, deux workers
//! Playwright appelant `seedTestState` en parallèle pourraient produire
//! des doublons (pas d'UNIQUE sur company name) ou des FK partielles.

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Request, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{Router, post};
use kesh_db::test_fixtures::{
    mark_onboarding_complete, seed_accounting_company, seed_changeme_user_only,
    seed_contact_and_product, truncate_all,
};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tokio::sync::Mutex;

use crate::AppState;
use crate::errors::AppError;

/// Construit le sous-routeur des endpoints de test. Monté sous
/// `/api/v1/_test` dans `build_router` **uniquement si `config.test_mode == true`**.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/seed", post(seed_handler))
        .route("/reset", post(reset_handler))
}

/// Mutex global sérialisant tous les appels seed/reset (code review P2).
/// Protège contre les races Playwright workers parallèles : sans ce lock,
/// deux `beforeAll(seedTestState('with-company'))` concurrents pourraient
/// laisser la DB avec 2 companies (pas d'UNIQUE sur `companies.name`) ou
/// en état partiel (TRUNCATE en cours côté A pendant INSERT côté B).
fn seed_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// Presets supportés par l'endpoint de seed. L'enum limite la combinatoire
/// et empêche les callers d'inventer des presets ad-hoc (cf. décision
/// conception story 6.4).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Preset {
    /// AC #7 : DB vidée + 1 user `changeme/changeme` (pas de company, pas
    /// d'onboarding_state).
    Fresh,
    /// AC #8 : state complet post-onboarding (company + 2 users +
    /// fiscal_year + 5 accounts + company_invoice_settings +
    /// onboarding_state.step_completed = 10).
    PostOnboarding,
    /// AC #9 : alias sémantique de `PostOnboarding` (même code path).
    WithCompany,
    /// AC #10 : `WithCompany` + 1 contact `'CI Contact SA'` + 1 product
    /// `'CI Product'`. **Pas de facture pré-seedée** — les specs créent
    /// leurs fixtures dynamiquement.
    WithData,
}

#[derive(Debug, Deserialize)]
pub struct SeedRequest {
    pub preset: Preset,
}

const VALID_PRESETS: &str = "fresh, post-onboarding, with-company, with-data";

/// Extracteur custom (code review AC #11) qui wrappe `Json<SeedRequest>`
/// et intercepte les rejets serde pour produire un **400 Bad Request** avec
/// un message listant les presets valides, au lieu du 422 serde par défaut.
pub struct SeedRequestExtractor(pub SeedRequest);

impl<S> FromRequest<S> for SeedRequestExtractor
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<SeedRequest>::from_request(req, state).await {
            Ok(Json(body)) => Ok(Self(body)),
            Err(rej) => {
                let message = match &rej {
                    JsonRejection::JsonDataError(_) | JsonRejection::JsonSyntaxError(_) => {
                        format!(
                            "preset invalide ou corps JSON malformé, valeurs acceptées : {}",
                            VALID_PRESETS
                        )
                    }
                    JsonRejection::MissingJsonContentType(_) => {
                        "Content-Type attendu : application/json".to_string()
                    }
                    // Code review pass 2 E3 : message générique pour éviter
                    // de leaker les détails internes des variants futurs
                    // (ex: `BytesRejection` peut inclure des infos hyper/h2
                    // sensibles dans son Display). Les détails vont en log.
                    _ => {
                        tracing::warn!(
                            rejection = %rej,
                            "SeedRequestExtractor: unhandled JsonRejection variant"
                        );
                        "requête invalide (corps non-parsable)".to_string()
                    }
                };
                Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": message })),
                )
                    .into_response())
            }
        }
    }
}

/// Corps de réponse JSON minimal (code review P11). Les callers fiables
/// asserent via `SELECT COUNT(*)` sur la DB — on n'expose plus des comptes
/// hardcodés qui mentiraient si `seed_accounting_company` changeait.
#[derive(Debug, Serialize)]
pub struct SeedResponse {
    pub preset: &'static str,
    pub ok: bool,
}

/// `POST /api/v1/_test/seed` — truncate puis re-seed selon le preset.
///
/// Sérialisé par `seed_lock()` pour empêcher les races inter-workers
/// (code review P2).
async fn seed_handler(
    State(state): State<AppState>,
    SeedRequestExtractor(req): SeedRequestExtractor,
) -> Result<Json<SeedResponse>, AppError> {
    let _guard = seed_lock().lock().await;

    truncate_all(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("test_fixtures truncate_all: {e}")))?;

    let preset_label: &'static str = match req.preset {
        Preset::Fresh => {
            seed_changeme_user_only(&state.pool)
                .await
                .map_err(|e| AppError::Internal(format!("seed_changeme_user_only: {e}")))?;
            "fresh"
        }
        Preset::PostOnboarding | Preset::WithCompany => {
            seed_accounting_company(&state.pool)
                .await
                .map_err(|e| AppError::Internal(format!("seed_accounting_company: {e}")))?;
            mark_onboarding_complete(&state.pool)
                .await
                .map_err(|e| AppError::Internal(format!("mark_onboarding_complete: {e}")))?;
            match req.preset {
                Preset::PostOnboarding => "post-onboarding",
                Preset::WithCompany => "with-company",
                _ => unreachable!(),
            }
        }
        Preset::WithData => {
            let seeded = seed_accounting_company(&state.pool)
                .await
                .map_err(|e| AppError::Internal(format!("seed_accounting_company: {e}")))?;
            mark_onboarding_complete(&state.pool)
                .await
                .map_err(|e| AppError::Internal(format!("mark_onboarding_complete: {e}")))?;
            seed_contact_and_product(&state.pool, seeded.company_id)
                .await
                .map_err(|e| AppError::Internal(format!("seed_contact_and_product: {e}")))?;
            "with-data"
        }
    };

    Ok(Json(SeedResponse {
        preset: preset_label,
        ok: true,
    }))
}

/// `POST /api/v1/_test/reset` — alias de `seed { preset: "fresh" }`.
/// Également sérialisé par `seed_lock()` (code review P2).
async fn reset_handler(State(state): State<AppState>) -> Result<Json<SeedResponse>, AppError> {
    let _guard = seed_lock().lock().await;

    truncate_all(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("test_fixtures truncate_all: {e}")))?;
    seed_changeme_user_only(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("seed_changeme_user_only: {e}")))?;
    Ok(Json(SeedResponse {
        preset: "fresh",
        ok: true,
    }))
}
