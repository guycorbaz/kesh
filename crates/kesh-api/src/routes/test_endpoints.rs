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

use axum::Json;
use axum::extract::State;
use axum::routing::{Router, post};
use kesh_db::test_fixtures::{
    mark_onboarding_complete, seed_accounting_company, seed_changeme_user_only,
    seed_contact_and_product, truncate_all,
};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::errors::AppError;

/// Construit le sous-routeur des endpoints de test. Monté sous
/// `/api/v1/_test` dans `build_router` **uniquement si `config.test_mode == true`**.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/seed", post(seed_handler))
        .route("/reset", post(reset_handler))
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

/// Corps de réponse JSON. Informational — les callers fiables s'appuient
/// sur le status 200 + leur propre assert sur l'état DB plutôt que parser
/// ce payload (cf. AC #14d).
#[derive(Debug, Serialize, Default)]
pub struct SeedResponse {
    pub preset: &'static str,
    pub users: usize,
    pub companies: usize,
    pub fiscal_years: usize,
    pub accounts: usize,
    pub contacts: usize,
    pub products: usize,
    pub onboarding_complete: bool,
}

impl SeedResponse {
    fn fresh() -> Self {
        Self {
            preset: "fresh",
            users: 1,
            ..Self::default()
        }
    }

    fn post_onboarding(alias: &'static str) -> Self {
        Self {
            preset: alias,
            users: 2,
            companies: 1,
            fiscal_years: 1,
            accounts: 5,
            onboarding_complete: true,
            ..Self::default()
        }
    }

    fn with_data() -> Self {
        Self {
            preset: "with-data",
            users: 2,
            companies: 1,
            fiscal_years: 1,
            accounts: 5,
            contacts: 1,
            products: 1,
            onboarding_complete: true,
        }
    }
}

/// `POST /api/v1/_test/seed` — truncate puis re-seed selon le preset.
async fn seed_handler(
    State(state): State<AppState>,
    Json(req): Json<SeedRequest>,
) -> Result<Json<SeedResponse>, AppError> {
    truncate_all(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("test_fixtures truncate_all: {e}")))?;

    let response = match req.preset {
        Preset::Fresh => {
            seed_changeme_user_only(&state.pool)
                .await
                .map_err(|e| AppError::Internal(format!("seed_changeme_user_only: {e}")))?;
            SeedResponse::fresh()
        }
        Preset::PostOnboarding | Preset::WithCompany => {
            seed_accounting_company(&state.pool)
                .await
                .map_err(|e| AppError::Internal(format!("seed_accounting_company: {e}")))?;
            mark_onboarding_complete(&state.pool)
                .await
                .map_err(|e| AppError::Internal(format!("mark_onboarding_complete: {e}")))?;
            let alias = match req.preset {
                Preset::PostOnboarding => "post-onboarding",
                Preset::WithCompany => "with-company",
                _ => unreachable!(),
            };
            SeedResponse::post_onboarding(alias)
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
            SeedResponse::with_data()
        }
    };

    Ok(Json(response))
}

/// `POST /api/v1/_test/reset` — alias de `seed { preset: "fresh" }`.
async fn reset_handler(State(state): State<AppState>) -> Result<Json<SeedResponse>, AppError> {
    truncate_all(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("test_fixtures truncate_all: {e}")))?;
    seed_changeme_user_only(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("seed_changeme_user_only: {e}")))?;
    Ok(Json(SeedResponse::fresh()))
}
