//! Routes d'onboarding — wizard de configuration initiale.
//!
//! Progression stricte par step :
//! - POST language : step == 0
//! - POST mode : step == 1
//! - POST seed-demo : step == 2 (Path A)
//! - POST start-production : step == 2 (Path B)
//! - POST org-type : step == 3
//! - POST accounting-language : step == 4
//! - POST coordinates : step == 5
//! - POST bank-account : step == 6
//! - POST skip-bank : step == 6
//! - POST reset : aucun prérequis

use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};

use kesh_db::entities::onboarding::UiMode;
use kesh_db::entities::{Language, OrgType};
use kesh_db::repositories::onboarding;

use crate::AppState;
use crate::errors::AppError;

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

    let ui_mode = current.ui_mode.unwrap_or(UiMode::Guided);

    kesh_seed::seed_demo(&state.pool, &state.config.locale, ui_mode, current.version)
        .await
        .map_err(|e| AppError::Internal(format!("Seed demo failed: {e}")))?;

    // seed_demo met déjà step=3 via update_step — relire l'état
    let updated = get_or_init_state(&state).await?;
    Ok(Json(updated.into()))
}

/// POST /api/v1/onboarding/reset — aucun prérequis de step
pub async fn reset(State(state): State<AppState>) -> Result<Json<OnboardingResponse>, AppError> {
    kesh_seed::reset_demo(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Reset demo failed: {e}")))?;

    // reset_demo recrée onboarding_state à step=0
    let updated = get_or_init_state(&state).await?;
    Ok(Json(updated.into()))
}

// --- Path B endpoints (Story 2.3) ---

/// POST /api/v1/onboarding/start-production — step 2→3
pub async fn start_production(
    State(state): State<AppState>,
) -> Result<Json<OnboardingResponse>, AppError> {
    let current = get_or_init_state(&state).await?;
    if current.step_completed != 2 || current.is_demo {
        return Err(AppError::OnboardingStepAlreadyCompleted);
    }

    let updated =
        onboarding::update_step(&state.pool, 3, false, current.ui_mode, current.version).await?;

    Ok(Json(updated.into()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgTypeRequest {
    pub org_type: String,
}

/// POST /api/v1/onboarding/org-type — step 3→4
pub async fn set_org_type(
    State(state): State<AppState>,
    Json(body): Json<OrgTypeRequest>,
) -> Result<Json<OnboardingResponse>, AppError> {
    let org_type: OrgType = body.org_type.parse().map_err(|_| {
        AppError::Validation(format!("Type d'organisation invalide : {}", body.org_type))
    })?;

    let current = get_or_init_state(&state).await?;
    if current.step_completed != 3 || current.is_demo {
        return Err(AppError::OnboardingStepAlreadyCompleted);
    }

    update_company_org_type(&state, org_type).await?;

    let updated = onboarding::update_step(
        &state.pool,
        4,
        current.is_demo,
        current.ui_mode,
        current.version,
    )
    .await?;

    Ok(Json(updated.into()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountingLanguageRequest {
    pub language: String,
}

/// POST /api/v1/onboarding/accounting-language — step 4→5
pub async fn set_accounting_language(
    State(state): State<AppState>,
    Json(body): Json<AccountingLanguageRequest>,
) -> Result<Json<OnboardingResponse>, AppError> {
    let lang: Language = body
        .language
        .parse()
        .map_err(|_| AppError::Validation(format!("Langue invalide : {}", body.language)))?;

    let current = get_or_init_state(&state).await?;
    if current.step_completed != 4 || current.is_demo {
        return Err(AppError::OnboardingStepAlreadyCompleted);
    }

    update_company_accounting_language(&state, lang).await?;

    // Story 3-1 (FR5) : charger le plan comptable adapté au org_type + accounting_language.
    // À ce stade (step 4→5), org_type ET accounting_language sont tous deux connus.
    // Guard idempotence : ne pas recharger si des comptes existent déjà (retry/navigation arrière).
    let company = get_company(&state).await?;
    let existing =
        kesh_db::repositories::accounts::count_by_company(&state.pool, company.id).await?;
    if existing == 0 {
        let chart = kesh_core::chart_of_accounts::load_chart(company.org_type.as_str())
            .map_err(|e| AppError::Internal(format!("Chargement plan comptable : {e}")))?;
        let lang_key = lang.as_str().to_lowercase();
        kesh_db::repositories::accounts::bulk_create_from_chart(
            &state.pool,
            company.id,
            &chart,
            &lang_key,
        )
        .await?;
    }

    let updated = onboarding::update_step(
        &state.pool,
        5,
        current.is_demo,
        current.ui_mode,
        current.version,
    )
    .await?;

    Ok(Json(updated.into()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoordinatesRequest {
    pub name: String,
    pub address: String,
    pub ide_number: Option<String>,
}

/// POST /api/v1/onboarding/coordinates — step 5→6
pub async fn set_coordinates(
    State(state): State<AppState>,
    Json(body): Json<CoordinatesRequest>,
) -> Result<Json<OnboardingResponse>, AppError> {
    let name = body.name.trim().to_string();
    let address = body.address.trim().to_string();

    if name.is_empty() {
        return Err(AppError::Validation("Le nom ne peut pas être vide".into()));
    }
    if address.is_empty() {
        return Err(AppError::Validation(
            "L'adresse ne peut pas être vide".into(),
        ));
    }

    // Validate IDE via kesh-core if provided
    let normalized_ide = match &body.ide_number {
        Some(ide) if !ide.trim().is_empty() => {
            let che = kesh_core::types::CheNumber::new(ide)
                .map_err(|e| AppError::Validation(format!("IDE invalide : {e}")))?;
            Some(che.as_str().to_string())
        }
        _ => None,
    };

    let current = get_or_init_state(&state).await?;
    if current.step_completed != 5 || current.is_demo {
        return Err(AppError::OnboardingStepAlreadyCompleted);
    }

    update_company_coordinates(&state, &name, &address, normalized_ide.as_deref()).await?;

    let updated = onboarding::update_step(
        &state.pool,
        6,
        current.is_demo,
        current.ui_mode,
        current.version,
    )
    .await?;

    Ok(Json(updated.into()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankAccountRequest {
    pub bank_name: String,
    pub iban: String,
    pub qr_iban: Option<String>,
}

/// POST /api/v1/onboarding/bank-account — step 6→7
pub async fn set_bank_account(
    State(state): State<AppState>,
    Json(body): Json<BankAccountRequest>,
) -> Result<Json<OnboardingResponse>, AppError> {
    let bank_name = body.bank_name.trim().to_string();
    if bank_name.is_empty() {
        return Err(AppError::Validation(
            "Le nom de la banque ne peut pas être vide".into(),
        ));
    }

    // Validate IBAN via kesh-core
    let iban = kesh_core::types::Iban::new(&body.iban)
        .map_err(|e| AppError::Validation(format!("IBAN invalide : {e}")))?;

    // Validate QR-IBAN via kesh-core if provided
    let normalized_qr = match &body.qr_iban {
        Some(qr) if !qr.trim().is_empty() => {
            let qr_iban = kesh_core::types::QrIban::new(qr)
                .map_err(|e| AppError::Validation(format!("QR-IBAN invalide : {e}")))?;
            Some(qr_iban.as_iban().as_str().to_string())
        }
        _ => None,
    };

    let current = get_or_init_state(&state).await?;
    if current.step_completed != 6 || current.is_demo {
        return Err(AppError::OnboardingStepAlreadyCompleted);
    }

    // Get company_id for FK
    let company = get_company(&state).await?;

    // Upsert primary bank account (idempotent in case of retry)
    use kesh_db::entities::NewBankAccount;
    use kesh_db::repositories::bank_accounts;

    bank_accounts::upsert_primary(
        &state.pool,
        NewBankAccount {
            company_id: company.id,
            bank_name,
            iban: iban.as_str().to_string(),
            qr_iban: normalized_qr,
            is_primary: true,
        },
    )
    .await?;

    let updated = onboarding::update_step(
        &state.pool,
        7,
        current.is_demo,
        current.ui_mode,
        current.version,
    )
    .await?;

    Ok(Json(updated.into()))
}

/// POST /api/v1/onboarding/skip-bank — step 6→7 without creating bank account
pub async fn skip_bank(
    State(state): State<AppState>,
) -> Result<Json<OnboardingResponse>, AppError> {
    let current = get_or_init_state(&state).await?;
    if current.step_completed != 6 || current.is_demo {
        return Err(AppError::OnboardingStepAlreadyCompleted);
    }

    let updated = onboarding::update_step(
        &state.pool,
        7,
        current.is_demo,
        current.ui_mode,
        current.version,
    )
    .await?;

    Ok(Json(updated.into()))
}

/// POST /api/v1/onboarding/finalize — step 7→complete (Path B only)
/// Pre-fills invoice settings with default accounts (1100, 3000) if they exist in the chart.
///
/// F2 CRITICAL FIX: Added check for already-finalized state (step == 8) to allow safe retries.
/// If step is already 8, finalization already succeeded → idempotent at HTTP level.
/// Settings INSERT IGNORE ensures database-level idempotency as well.
///
/// F14 MEDIUM FIX: Updated docstring — route IS idempotent if already at step 8 (allows retries).
/// Rejects only if at step < 7 or >= 8 (wrong state).
///
/// F15 MEDIUM FIX: Added explicit backend validation — if both account IDs are NULL,
/// return 400 error instead of silently accepting incomplete settings.
pub async fn finalize(State(state): State<AppState>) -> Result<Json<OnboardingResponse>, AppError> {
    let current = get_or_init_state(&state).await?;

    // F2 CRITICAL FIX: Allow idempotent retries if already finalized (step == 8)
    if current.is_demo {
        return Err(AppError::OnboardingStepAlreadyCompleted);
    }

    if current.step_completed != 7 && current.step_completed != 8 {
        return Err(AppError::OnboardingStepAlreadyCompleted);
    }

    // If already finalized, return current state (idempotent)
    if current.step_completed == 8 {
        return Ok(Json(current.into()));
    }

    // Get company to create invoice settings
    let company = get_company(&state).await?;

    // Pre-fill invoice settings with default accounts (1100, 3000).
    // Uses INSERT IGNORE pattern for database-level idempotency.
    // F1 CRITICAL FIX: Account lookups use SELECT FOR UPDATE to prevent concurrent deletes.
    let settings = kesh_db::repositories::company_invoice_settings::insert_with_defaults(&state.pool, company.id)
        .await
        .map_err(|e| AppError::Database(e))?;

    // F15 MEDIUM FIX: Validate that settings are not completely unconfigured.
    // Both accounts NULL means the chart doesn't have standard Swiss accounts → cannot proceed.
    if settings.default_receivable_account_id.is_none() && settings.default_revenue_account_id.is_none() {
        return Err(AppError::Internal(
            "Cannot finalize onboarding: accounts 1100 (Receivables) and 3000 (Revenue) not found in chart. \
             Please reload the chart or contact support.".to_string(),
        ));
    }

    // Mark onboarding as complete (step 8 indicates completion)
    let updated = onboarding::update_step(
        &state.pool,
        8,
        current.is_demo,
        current.ui_mode,
        current.version,
    )
    .await?;

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
async fn ensure_company_with_language(state: &AppState, lang: Language) -> Result<(), AppError> {
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
                return Err(AppError::Database(
                    kesh_db::errors::DbError::OptimisticLockConflict,
                ));
            }
        }
    }

    tx.commit().await.map_err(map_db_error)?;
    Ok(())
}

const COMPANY_SELECT_FOR_UPDATE: &str = "SELECT id, name, address, ide_number, org_type, accounting_language, \
            instance_language, version, created_at, updated_at \
     FROM companies LIMIT 1 FOR UPDATE";

/// Retourne la company (première et unique). Erreur si aucune company n'existe.
async fn get_company(state: &AppState) -> Result<kesh_db::entities::Company, AppError> {
    use kesh_db::repositories::companies;
    let list = companies::list(&state.pool, 1, 0).await?;
    list.into_iter()
        .next()
        .ok_or_else(|| AppError::Internal("Aucune company en base".into()))
}

/// Met à jour `company.org_type` via SELECT FOR UPDATE + OL.
async fn update_company_org_type(state: &AppState, org_type: OrgType) -> Result<(), AppError> {
    use kesh_db::errors::map_db_error;
    let mut tx = state.pool.begin().await.map_err(map_db_error)?;
    let company = sqlx::query_as::<_, kesh_db::entities::Company>(COMPANY_SELECT_FOR_UPDATE)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_error)?;
    let rows = sqlx::query(
        "UPDATE companies SET org_type = ?, version = version + 1 WHERE id = ? AND version = ?",
    )
    .bind(org_type)
    .bind(company.id)
    .bind(company.version)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    if rows == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(AppError::Database(
            kesh_db::errors::DbError::OptimisticLockConflict,
        ));
    }
    tx.commit().await.map_err(map_db_error)?;
    Ok(())
}

/// Met à jour `company.accounting_language` via SELECT FOR UPDATE + OL.
async fn update_company_accounting_language(
    state: &AppState,
    lang: Language,
) -> Result<(), AppError> {
    use kesh_db::errors::map_db_error;
    let mut tx = state.pool.begin().await.map_err(map_db_error)?;
    let company = sqlx::query_as::<_, kesh_db::entities::Company>(COMPANY_SELECT_FOR_UPDATE)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_error)?;
    let rows = sqlx::query(
        "UPDATE companies SET accounting_language = ?, version = version + 1 WHERE id = ? AND version = ?",
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
        return Err(AppError::Database(
            kesh_db::errors::DbError::OptimisticLockConflict,
        ));
    }
    tx.commit().await.map_err(map_db_error)?;
    Ok(())
}

/// Met à jour les coordonnées de la company (name, address, ide_number).
async fn update_company_coordinates(
    state: &AppState,
    name: &str,
    address: &str,
    ide_number: Option<&str>,
) -> Result<(), AppError> {
    use kesh_db::errors::map_db_error;
    let mut tx = state.pool.begin().await.map_err(map_db_error)?;
    let company = sqlx::query_as::<_, kesh_db::entities::Company>(COMPANY_SELECT_FOR_UPDATE)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_error)?;
    let rows = sqlx::query(
        "UPDATE companies SET name = ?, address = ?, ide_number = ?, version = version + 1 \
         WHERE id = ? AND version = ?",
    )
    .bind(name)
    .bind(address)
    .bind(ide_number)
    .bind(company.id)
    .bind(company.version)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    if rows == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(AppError::Database(
            kesh_db::errors::DbError::OptimisticLockConflict,
        ));
    }
    tx.commit().await.map_err(map_db_error)?;
    Ok(())
}
