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

    // seed_demo already calls insert_with_defaults internally (Story 2.6)
    // to pre-fill invoice accounts with 1100 (receivable) and 3000 (revenue).
    // seed_demo updates onboarding_state to step=3 via update_step — relire l'état
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
/// F2 CRITICAL FIX: SELECT FOR UPDATE on onboarding_state serializes all finalize() calls.
/// Prevents multiple concurrent requests from both passing the step check and duplicating work.
/// Broader locking scope ensures deterministic behavior under concurrent load.
///
/// F3 CRITICAL FIX: SELECT FOR UPDATE on company prevents deletion between check and insert.
/// Company is locked for update, so DELETE from another transaction must wait.
/// If company was deleted before we acquired lock, SELECT returns no row → error.
///
/// F4 HIGH FIX: Pessimistic lock on onboarding_state prevents concurrent finalize() races.
/// Once locked, only one finalize() can proceed. INSERT IGNORE remains idempotent.
///
/// F1 CRITICAL VALIDATION: Ensure account pre-fill succeeded (1100, 3000 not NULL).
pub async fn finalize(State(state): State<AppState>) -> Result<Json<OnboardingResponse>, AppError> {
    use kesh_db::errors::map_db_error;

    // F2/F3/F4 CRITICAL FIX: Pessimistic locking strategy.
    // 1. Lock onboarding_state (serializes all finalize() calls on same session)
    // 2. Check state is still at step 7 or 8 (prevents TOCTOU on onboarding progression)
    // 3. Lock company row (prevents deletion during finalize)
    // 4. Proceed with insert_with_defaults() with guaranteed exclusive access
    let mut tx = state.pool.begin().await.map_err(map_db_error)?;

    let onboarding = sqlx::query_as::<_, kesh_db::entities::OnboardingState>(
        "SELECT id, singleton, step_completed, is_demo, ui_mode, version, created_at, updated_at \
         FROM onboarding_state WHERE singleton = TRUE FOR UPDATE",
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_error)?;

    // Reject demo path finalize (demo is finalized via seed_demo)
    if onboarding.is_demo {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(AppError::OnboardingStepAlreadyCompleted);
    }

    // Allow idempotent retry if already finalized (step == 8)
    if onboarding.step_completed < 7 || onboarding.step_completed > 8 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(AppError::OnboardingStepAlreadyCompleted);
    }

    // If already finalized, release lock and return (idempotent)
    if onboarding.step_completed == 8 {
        tx.commit().await.map_err(map_db_error)?;
        return Ok(Json(onboarding.into()));
    }

    // F3 CRITICAL FIX: Lock company row before insert_with_defaults()
    // Prevents concurrent deletion between our check and INSERT INTO company_invoice_settings.
    let company = sqlx::query_as::<_, kesh_db::entities::Company>(
        "SELECT id, name, address, ide_number, org_type, accounting_language, \
                instance_language, version, created_at, updated_at \
         FROM companies LIMIT 1 FOR UPDATE",
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?
    .ok_or_else(|| {
        AppError::Internal("Aucune company en base (company supprimée pendant onboarding ?)".into())
    })?;

    // Pre-fill invoice settings with default accounts (1100, 3000).
    // Uses INSERT IGNORE pattern for database-level idempotency.
    // Account lookups use SELECT FOR UPDATE to prevent concurrent deletes.
    // F2/F3/F4: Transaction-level variant keeps account locks within this transaction,
    // preserving company and onboarding_state locks until step update completes.
    let settings = kesh_db::repositories::company_invoice_settings::insert_with_defaults_in_tx(
        &mut tx, company.id,
    )
    .await
    .map_err(AppError::Database)?;

    // F1 CRITICAL VALIDATION: Ensure account pre-fill succeeded.
    // Swiss PME/Association/Independant charts must contain accounts 1100 (receivable) and 3000 (revenue).
    // If missing, the onboarding cannot proceed (AC 3 fallback UI not yet implemented).
    if settings.default_receivable_account_id.is_none()
        || settings.default_revenue_account_id.is_none()
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(AppError::Validation(
            "Impossible de pré-remplir les comptes de facturation (1100, 3000 manquants du plan comptable). \
             Veuillez ajouter ces comptes avant de finaliser l'onboarding.".into(),
        ));
    }

    // Mark onboarding as complete while holding locks.
    // Uses optimistic locking with expected version to detect concurrent updates (shouldn't happen due to SELECT FOR UPDATE).
    let rows = sqlx::query(
        "UPDATE onboarding_state SET step_completed = 8, version = version + 1 \
         WHERE singleton = TRUE AND version = ?",
    )
    .bind(onboarding.version)
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

    let updated = sqlx::query_as::<_, kesh_db::entities::OnboardingState>(
        "SELECT id, singleton, step_completed, is_demo, ui_mode, version, created_at, updated_at \
         FROM onboarding_state WHERE singleton = TRUE",
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_error)?;

    tx.commit().await.map_err(map_db_error)?;
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
