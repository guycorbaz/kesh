//! Routes d'onboarding — wizard de configuration initiale.
//!
//! Progression stricte par step :
//! - POST language : step == 0
//! - POST mode : step == 1
//! - POST seed-demo : step == 2
//! - POST reset : aucun prérequis

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};

use kesh_db::entities::onboarding::UiMode;
use kesh_db::entities::{Language, OrgType};
use kesh_db::repositories::onboarding;

use crate::errors::AppError;
use crate::AppState;

/// Réponse JSON pour l'état d'onboarding (camelCase).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingResponse {
    pub step_completed: i32,
    pub is_demo: bool,
    pub ui_mode: Option<UiMode>,
}

impl From<kesh_db::entities::OnboardingState> for OnboardingResponse {
    fn from(s: kesh_db::entities::OnboardingState) -> Self {
        Self {
            step_completed: s.step_completed,
            is_demo: s.is_demo,
            ui_mode: s.ui_mode,
        }
    }
}

/// GET /api/v1/onboarding/state
pub async fn get_state(
    State(state): State<AppState>,
) -> Result<Json<OnboardingResponse>, AppError> {
    let current = get_or_init_state(&state).await?;
    Ok(Json(current.into()))
}

#[derive(Debug, Deserialize)]
pub struct LanguageRequest {
    pub language: String,
}

/// POST /api/v1/onboarding/language — step 0→1
///
/// Note : `ONBOARDING_STEP_ALREADY_COMPLETED` est utilisé comme code unique
/// pour toute violation de progression (step trop bas ET step trop haut).
/// Décision simplifiée : un code par type d'erreur suffit pour le MVP.
pub async fn set_language(
    State(state): State<AppState>,
    Json(body): Json<LanguageRequest>,
) -> Result<Json<OnboardingResponse>, AppError> {
    let lang: Language = body
        .language
        .parse()
        .map_err(|_| AppError::Validation(format!("Langue invalide : {}", body.language)))?;

    let current = get_or_init_state(&state).await?;
    if current.step_completed != 0 {
        return Err(AppError::OnboardingStepAlreadyCompleted);
    }

    // Créer ou mettre à jour la company
    ensure_company_with_language(&state, lang).await?;

    let updated = onboarding::update_step(
        &state.pool,
        1,
        current.is_demo,
        current.ui_mode,
        current.version,
    )
    .await?;

    Ok(Json(updated.into()))
}

#[derive(Debug, Deserialize)]
pub struct ModeRequest {
    pub mode: String,
}

/// POST /api/v1/onboarding/mode — step 1→2
pub async fn set_mode(
    State(state): State<AppState>,
    Json(body): Json<ModeRequest>,
) -> Result<Json<OnboardingResponse>, AppError> {
    let ui_mode: UiMode = body
        .mode
        .parse()
        .map_err(|_| AppError::Validation(format!("Mode invalide : {}", body.mode)))?;

    let current = get_or_init_state(&state).await?;
    if current.step_completed != 1 {
        return Err(AppError::OnboardingStepAlreadyCompleted);
    }

    let updated = onboarding::update_step(
        &state.pool,
        2,
        current.is_demo,
        Some(ui_mode),
        current.version,
    )
    .await?;

    Ok(Json(updated.into()))
}

/// POST /api/v1/onboarding/seed-demo — step 2→3
pub async fn seed_demo(
    State(state): State<AppState>,
) -> Result<Json<OnboardingResponse>, AppError> {
    let current = get_or_init_state(&state).await?;
    if current.step_completed != 2 {
        return Err(AppError::OnboardingStepAlreadyCompleted);
    }

    let ui_mode = current
        .ui_mode
        .unwrap_or(UiMode::Guided);

    kesh_seed::seed_demo(&state.pool, &state.config.locale, ui_mode, current.version)
        .await
        .map_err(|e| AppError::Internal(format!("Seed demo failed: {e}")))?;

    // seed_demo met déjà step=3 via update_step — relire l'état
    let updated = get_or_init_state(&state).await?;
    Ok(Json(updated.into()))
}

/// POST /api/v1/onboarding/reset — aucun prérequis de step
pub async fn reset(
    State(state): State<AppState>,
) -> Result<Json<OnboardingResponse>, AppError> {
    kesh_seed::reset_demo(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Reset demo failed: {e}")))?;

    // reset_demo recrée onboarding_state à step=0
    let updated = get_or_init_state(&state).await?;
    Ok(Json(updated.into()))
}

// --- Helpers ---

/// Retourne l'état d'onboarding existant ou en crée un nouveau.
async fn get_or_init_state(
    state: &AppState,
) -> Result<kesh_db::entities::OnboardingState, AppError> {
    match onboarding::get_state(&state.pool).await? {
        Some(s) => Ok(s),
        None => Ok(onboarding::init_state(&state.pool).await?),
    }
}

/// S'assure qu'une company existe avec la bonne `instance_language`.
///
/// Utilise une transaction avec SELECT FOR UPDATE pour éviter la race condition
/// TOCTOU (deux requêtes concurrentes créant chacune une company).
async fn ensure_company_with_language(
    state: &AppState,
    lang: Language,
) -> Result<(), AppError> {
    use kesh_db::errors::map_db_error;

    let mut tx = state.pool.begin().await.map_err(map_db_error)?;

    // SELECT FOR UPDATE verrouille la row (ou rien si table vide)
    let existing = sqlx::query_as::<_, kesh_db::entities::Company>(
        "SELECT id, name, address, ide_number, org_type, accounting_language, \
                instance_language, version, created_at, updated_at \
         FROM companies LIMIT 1 FOR UPDATE",
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?;

    match existing {
        None => {
            sqlx::query(
                "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind("(en cours de configuration)")
            .bind("-")
            .bind(OrgType::Independant)
            .bind(Language::Fr)
            .bind(lang)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?;
        }
        Some(company) => {
            let rows = sqlx::query(
                "UPDATE companies SET instance_language = ?, version = version + 1 \
                 WHERE id = ? AND version = ?",
            )
            .bind(lang)
            .bind(company.id)
            .bind(company.version)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?
            .rows_affected();
            if rows == 0 {
                tx.rollback().await.map_err(map_db_error)?;
                return Err(AppError::Database(kesh_db::errors::DbError::OptimisticLockConflict));
            }
        }
    }

    tx.commit().await.map_err(map_db_error)?;
    Ok(())
}
