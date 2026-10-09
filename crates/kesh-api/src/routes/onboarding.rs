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

use axum::extract::State;
use axum::{Extension, Json};
use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

use kesh_db::entities::audit_log::{AUDIT_ENTITY_ID_NONE, NewAuditLogEntry};
use kesh_db::entities::onboarding::UiMode;
use kesh_db::entities::{
    BankAccount, Company, CompanyInvoiceSettings, CompanyUpdate, Language, NewBankAccount,
    NewFiscalYear, OnboardingState, OrgType, VatRate,
};
use kesh_db::errors::map_db_error;
use kesh_db::repositories::bank_accounts::{self, UpsertPrimaryOutcome};
use kesh_db::repositories::{audit_log, companies, onboarding};

use crate::AppState;
use crate::audit::AuditActor;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;

/// P1-H1: Helper for graceful transaction rollback
/// Rollback errors are best-effort cleanup; don't fail the request if rollback fails
async fn best_effort_rollback(tx: sqlx::Transaction<'_, sqlx::MySql>) {
    if let Err(e) = tx.rollback().await {
        tracing::warn!("Transaction rollback failed (best-effort cleanup): {}", e);
        // Continue anyway — connection pool handles cleanup
    }
}

/// P6-M1: Parse a boolean-like environment variable.
///
/// Accepts (case-insensitive) `"1"`, `"true"`, `"yes"`, `"on"` as `true`.
/// Anything else (including unset, empty, or unrecognized values) is `false`.
///
/// Used for `KESH_PRODUCTION_RESET` and any future opt-in env flag where a
/// silent mismatch (e.g. operator setting `"true"` for a `"1"`-only check)
/// would lead to confusing rejection of legitimate operations.
fn env_flag_enabled(name: &str) -> bool {
    // Lecture par `config::env_nonempty` (Story 15-11b) : valeur trimée,
    // vide = absente.
    match crate::config::env_nonempty(name) {
        Some(v) => matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"),
        None => false,
    }
}

/// Réponse JSON pour l'état d'onboarding (camelCase).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingResponse {
    pub step_completed: i32,
    pub is_demo: bool,
    pub ui_mode: Option<UiMode>,
    /// `true` si la company courante est un placeholder (créée par le bootstrap
    /// sur DB vide, ou par le wizard avant complétion). Le frontend l'utilise
    /// pour un nudge de renommage non-bloquant. Renseigné par `response_with_stub`
    /// (le `From<OnboardingState>` met `false` par défaut car il n'a pas accès
    /// à la company — toujours passer par `response_with_stub` pour le valuer).
    pub is_stub: bool,
}

impl From<kesh_db::entities::OnboardingState> for OnboardingResponse {
    fn from(s: kesh_db::entities::OnboardingState) -> Self {
        Self {
            step_completed: s.step_completed,
            is_demo: s.is_demo,
            ui_mode: s.ui_mode,
            is_stub: false,
        }
    }
}

/// Construit une `OnboardingResponse` en renseignant `is_stub` depuis la
/// company courante (première company ; `false` si aucune company). Toute
/// réponse onboarding renvoyée au client DOIT passer par ce helper pour que
/// le frontend ait toujours le `isStub` à jour (sinon le `From` met `false`).
async fn response_with_stub(
    pool: &sqlx::MySqlPool,
    state: kesh_db::entities::OnboardingState,
) -> Result<OnboardingResponse, AppError> {
    let is_stub: Option<bool> =
        sqlx::query_scalar("SELECT is_stub FROM companies ORDER BY id LIMIT 1")
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::Internal(format!("onboarding read is_stub: {e}")))?;
    let mut resp = OnboardingResponse::from(state);
    resp.is_stub = is_stub.unwrap_or(false);
    Ok(resp)
}

/// GET /api/v1/onboarding/state
pub async fn get_state(
    State(state): State<AppState>,
) -> Result<Json<OnboardingResponse>, AppError> {
    let current = get_or_init_state(&state).await?;
    Ok(Json(response_with_stub(&state.pool, current).await?))
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
///
/// Story 15-7a2 : une transaction (`onboarding_state → companies`), trace
/// `company.created` ou `company.updated {instance_language}` puis l'étape.
pub async fn set_language(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(body): Json<LanguageRequest>,
) -> Result<Json<OnboardingResponse>, AppError> {
    let lang: Language = body
        .language
        .parse()
        .map_err(|_| AppError::Validation(format!("Langue invalide : {}", body.language)))?;

    get_or_init_state(&state).await?;
    let mut tx = state.pool.begin().await.map_err(map_db_error)?;
    let result = language_in_tx(&mut tx, &current_user, lang).await;
    conclude_step(&state.pool, tx, result).await
}

/// Corps transactionnel de [`set_language`].
async fn language_in_tx(
    tx: &mut Tx<'_>,
    user: &CurrentUser,
    lang: Language,
) -> Result<OnboardingState, AppError> {
    let locked = lock_state_at_step(tx, 0, false).await?;
    ensure_company_with_language_in_tx(tx, user, lang).await?;
    complete_step(tx, user, &locked, locked.ui_mode, "language").await
}

#[derive(Debug, Deserialize)]
pub struct ModeRequest {
    pub mode: String,
}

/// POST /api/v1/onboarding/mode — step 1→2
///
/// Story 15-7a2 : `installation.ui_mode_changed` **si** le mode change (même
/// action et même forme que `PUT /api/v1/profile/mode`), puis l'étape.
/// ⚠️ `profile.rs` trace, lui, même sans changement : divergence assumée.
pub async fn set_mode(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(body): Json<ModeRequest>,
) -> Result<Json<OnboardingResponse>, AppError> {
    let ui_mode: UiMode = body
        .mode
        .parse()
        .map_err(|_| AppError::Validation(format!("Mode invalide : {}", body.mode)))?;

    get_or_init_state(&state).await?;
    let mut tx = state.pool.begin().await.map_err(map_db_error)?;
    let result = mode_in_tx(&mut tx, &current_user, ui_mode).await;
    conclude_step(&state.pool, tx, result).await
}

/// Corps transactionnel de [`set_mode`].
async fn mode_in_tx(
    tx: &mut Tx<'_>,
    user: &CurrentUser,
    ui_mode: UiMode,
) -> Result<OnboardingState, AppError> {
    let locked = lock_state_at_step(tx, 1, false).await?;
    if locked.ui_mode != Some(ui_mode) {
        audit_log::insert_in_tx(
            tx,
            NewAuditLogEntry::from_current_user(
                user,
                "installation.ui_mode_changed",
                "installation",
                AUDIT_ENTITY_ID_NONE,
                Some(json!({ "before": locked.ui_mode, "after": ui_mode })),
            ),
        )
        .await?;
    }
    complete_step(tx, user, &locked, Some(ui_mode), "mode").await
}

/// POST /api/v1/onboarding/seed-demo — step 2→3
///
/// Story 15-7b1 : la dernière transaction de `kesh_seed::seed_demo` lève le
/// drapeau provisoire de la société (`clear_stub_in_tx`), franchit l'étape et
/// écrit `installation.demo_seeded` puis `installation.step_completed`,
/// attribuées à l'acteur (jeton d'API compris). La version et le mode
/// d'affichage y sont relus **sous verrou**.
///
/// La pré-vérification **non verrouillée** de l'étape est conservée — à
/// l'inverse des routes de la 15-7a2 : sans elle, le renommage de la société,
/// le plan et l'exercice s'exécuteraient avant que la garde sous verrou ne
/// refuse. Correspondance d'erreurs : `StepAlreadyCompleted` (étape franchie
/// sous verrou, par `start-production`) ⇒ 400 ; comptes de rôle introuvables
/// ⇒ 422 ; toute autre erreur ⇒ 500 (un 1213 épuisé compris).
pub async fn seed_demo(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<OnboardingResponse>, AppError> {
    let current = get_or_init_state(&state).await?;
    if current.step_completed != 2 {
        return Err(AppError::OnboardingStepAlreadyCompleted);
    }

    // P11: surface actionable validation errors (chart de comptes mal configuré)
    // as 422 instead of 500 so the client can show a concrete remediation message.
    kesh_seed::seed_demo(
        &state.pool,
        &state.config.locale,
        (current_user.user_id, current_user.api_key_id),
    )
    .await
    .map_err(|e| match e {
        kesh_seed::SeedError::StepAlreadyCompleted => AppError::OnboardingStepAlreadyCompleted,
        kesh_seed::SeedError::Db(kesh_db::errors::DbError::InactiveOrInvalidAccounts) => {
            AppError::Validation(
                "Comptes par défaut introuvables (1100, 3000). \
                 Vérifiez que le plan comptable a bien été chargé avant de relancer la démo."
                    .into(),
            )
        }
        other => AppError::Internal(format!("Seed demo failed: {other}")),
    })?;

    // seed_demo a franchi l'étape 3 dans sa dernière transaction — relire l'état.
    let updated = get_or_init_state(&state).await?;
    Ok(Json(response_with_stub(&state.pool, updated).await?))
}

/// POST /api/v1/onboarding/reset — Step gating: allow demo, block post-production (E2-002 fix)
///
/// Step gating rules:
/// - SECURITY: step >= 7 is finalization (irreversible) — NEVER allow reset regardless of is_demo
/// - SECURITY (P4): production users (is_demo=false) can only reset up to step 2.
///   The is_demo flag alone is not a sufficient gate because corruption / manual DB edit
///   could flip it to true, allowing reset on a partially-configured production tenant
///   at steps 3-6. The KESH_PRODUCTION_RESET env var (default false) is the second factor:
///   in production deployments it must remain unset; only demo deployments set it.
/// - Demo users (is_demo=true) can reset at steps 0..=6
///
/// LOCK ORDERING (P3 — partial protection only):
/// We acquire SELECT FOR UPDATE on onboarding_state to serialize the gate-check
/// against concurrent finalize() / seed_demo() / step-progression endpoints.
/// The lock is **released (commit) before reset_demo runs**, so a concurrent
/// finalize() could still flip the state to 8 between commit and reset_demo's
/// DELETE. Under v0.1 single-tenant single-user this race is essentially
/// unreachable; full serialization (single-tx covering reset_demo) is tracked
/// under KF-002-H-002 (issue #43).
pub async fn reset(State(state): State<AppState>) -> Result<Json<OnboardingResponse>, AppError> {
    // Ensure the onboarding_state row exists before locking (idempotent init).
    let _ = get_or_init_state(&state).await?;

    // P3 fix: lock-and-check inside a transaction to close the read/action TOCTOU window.
    let mut tx = state.pool.begin().await.map_err(map_db_error)?;
    let current = sqlx::query_as::<_, kesh_db::entities::OnboardingState>(
        "SELECT id, singleton, step_completed, is_demo, ui_mode, version, created_at, updated_at \
         FROM onboarding_state WHERE singleton = TRUE FOR UPDATE",
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_error)?;

    // P1-H4: step >= 7 is irreversible finalization — never reset
    if current.step_completed >= 7 {
        best_effort_rollback(tx).await;
        return Err(AppError::OnboardingStepAlreadyCompleted);
    }

    // P4: production-mode safety net — refuse reset for is_demo=false past step 2
    // even if KESH_PRODUCTION_RESET is set, to avoid accidental wipes.
    // P6-L8: distinct ResetForbidden error so the client can show "reset not
    // permitted in production" rather than the misleading "step already completed".
    if !current.is_demo && current.step_completed > 2 {
        best_effort_rollback(tx).await;
        return Err(AppError::OnboardingResetForbidden);
    }

    // P4 hardening: even for is_demo=true at steps 3..=6, require explicit demo
    // deployment confirmation. This blocks the "corrupted is_demo flag" attack
    // path between steps 3 and 6 that the old gate ignored.
    // P6-M1: accept "1" | "true" | "yes" | "on" (case-insensitive) so an operator
    // setting `KESH_PRODUCTION_RESET=true` in docker-compose isn't silently denied.
    if current.step_completed > 2 && !env_flag_enabled("KESH_PRODUCTION_RESET") {
        best_effort_rollback(tx).await;
        return Err(AppError::OnboardingResetForbidden);
    }

    // Release the lock before invoking reset_demo: reset_demo internally acquires
    // its own connection and DELETEs onboarding_state, which would deadlock if we
    // kept the FOR UPDATE lock held here.
    // The lock therefore covers ONLY the gate-check above, NOT the destructive
    // reset_demo work. Under v0.1 single-tenant single-user the residual window
    // is unreachable in practice; full serialization is tracked under KF-002-H-002.
    tx.commit().await.map_err(map_db_error)?;

    kesh_seed::reset_demo(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("Reset demo failed: {e}")))?;

    // reset_demo recrée onboarding_state à step=0
    let updated = get_or_init_state(&state).await?;
    Ok(Json(response_with_stub(&state.pool, updated).await?))
}

// --- Path B endpoints (Story 2.3) ---

/// POST /api/v1/onboarding/start-production — step 2→3
///
/// Story 15-7a2 : aucune entrée de domaine — l'étape **est** le fait :
/// l'installation devient non réinitialisable.
pub async fn start_production(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<OnboardingResponse>, AppError> {
    get_or_init_state(&state).await?;
    let mut tx = state.pool.begin().await.map_err(map_db_error)?;
    let result = async {
        let locked = lock_state_at_step(&mut tx, 2, true).await?;
        complete_step(
            &mut tx,
            &current_user,
            &locked,
            locked.ui_mode,
            "start_production",
        )
        .await
    }
    .await;
    conclude_step(&state.pool, tx, result).await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgTypeRequest {
    pub org_type: String,
}

/// POST /api/v1/onboarding/org-type — step 3→4
///
/// Story 15-7a2 : `company.updated {org_type}` **si** il change, puis l'étape.
pub async fn set_org_type(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(body): Json<OrgTypeRequest>,
) -> Result<Json<OnboardingResponse>, AppError> {
    let org_type: OrgType = body.org_type.parse().map_err(|_| {
        AppError::Validation(format!("Type d'organisation invalide : {}", body.org_type))
    })?;

    get_or_init_state(&state).await?;
    let mut tx = state.pool.begin().await.map_err(map_db_error)?;
    let result = async {
        let locked = lock_state_at_step(&mut tx, 3, true).await?;
        let company = lock_company(&mut tx).await?;
        let mut changes = company_update_of(&company);
        changes.org_type = org_type;
        update_company_in_tx(
            &mut tx,
            &current_user,
            &company,
            changes,
            |c| json!({ "org_type": c.org_type }),
        )
        .await?;
        complete_step(&mut tx, &current_user, &locked, locked.ui_mode, "org_type").await
    }
    .await;
    conclude_step(&state.pool, tx, result).await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountingLanguageRequest {
    pub language: String,
}

/// POST /api/v1/onboarding/accounting-language — step 4→5
///
/// Story 15-7a2 : `company.updated {accounting_language}` **si** elle change,
/// puis `account.chart_loaded` **si** le plan est chargé, puis l'étape — en
/// une transaction `onboarding_state → companies → accounts`. La garde « aucun
/// compte » est lue **dans** la transaction, après le verrou d'état : deux
/// requêtes concurrentes ne chargent plus le plan deux fois.
pub async fn set_accounting_language(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(body): Json<AccountingLanguageRequest>,
) -> Result<Json<OnboardingResponse>, AppError> {
    let lang: Language = body
        .language
        .parse()
        .map_err(|_| AppError::Validation(format!("Langue invalide : {}", body.language)))?;

    get_or_init_state(&state).await?;
    let mut tx = state.pool.begin().await.map_err(map_db_error)?;
    let result = accounting_language_in_tx(&mut tx, &current_user, lang).await;
    conclude_step(&state.pool, tx, result).await
}

/// Corps transactionnel de [`set_accounting_language`].
async fn accounting_language_in_tx(
    tx: &mut Tx<'_>,
    user: &CurrentUser,
    lang: Language,
) -> Result<OnboardingState, AppError> {
    use kesh_db::repositories::accounts;

    let locked = lock_state_at_step(tx, 4, true).await?;
    let company = lock_company(tx).await?;
    let mut changes = company_update_of(&company);
    changes.accounting_language = lang;
    let company = update_company_in_tx(
        tx,
        user,
        &company,
        changes,
        |c| json!({ "accounting_language": c.accounting_language }),
    )
    .await?;

    // Story 3-1 (FR5) : charger le plan comptable adapté au org_type + accounting_language.
    // À ce stade (step 4→5), org_type ET accounting_language sont tous deux connus.
    // Guard idempotence : ne pas recharger si des comptes existent déjà (retry/navigation arrière).
    // ⚠️ Lue DANS la transaction, après `lock_state_at_step` (Story 15-7a2, E-3 de
    // la revue de la 15-7a1) : sur le pool, deux appels concurrents lisaient 0.
    let existing = accounts::count_by_company(&mut **tx, company.id).await?;
    if existing == 0 {
        let chart = kesh_core::chart_of_accounts::load_chart(company.org_type.as_str())
            .map_err(|e| AppError::Internal(format!("Chargement plan comptable : {e}")))?;
        let lang_key = lang.as_str().to_lowercase();
        let loaded =
            accounts::bulk_create_from_chart_in_tx(tx, company.id, &chart, &lang_key).await?;
        if !loaded.is_empty() {
            // Choix C-15-7-3 : une entrée AGRÉGÉE, non une par compte.
            let listed: Vec<serde_json::Value> = loaded
                .iter()
                .map(|a| json!({ "id": a.id, "number": a.number }))
                .collect();
            audit_log::insert_in_tx(
                tx,
                NewAuditLogEntry::from_current_user(
                    user,
                    "account.chart_loaded",
                    "account",
                    AUDIT_ENTITY_ID_NONE,
                    Some(json!({
                        "company_id": company.id,
                        "org_type": company.org_type,
                        "language": lang,
                        "count": loaded.len(),
                        "accounts": listed,
                    })),
                ),
            )
            .await?;
        }
    }

    complete_step(tx, user, &locked, locked.ui_mode, "accounting_language").await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoordinatesRequest {
    pub name: String,
    /// Prénom / nom (#213) — fournis si la société est une personne physique
    /// (raison individuelle) ; `name` est alors recomposé « Prénom Nom ».
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    /// Adresse structurée de la société (créancier QR-bill type S, #213).
    pub address: crate::address_input::StructuredAddressInput,
    pub ide_number: Option<String>,
}

/// POST /api/v1/onboarding/coordinates — step 5→6
pub async fn set_coordinates(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(body): Json<CoordinatesRequest>,
) -> Result<Json<OnboardingResponse>, AppError> {
    // #213 — personne physique (prénom + nom) → `name` recomposé ; sinon raison
    // sociale telle quelle. Le frontend décide selon l'OrgType choisi en amont.
    let clean = |o: &Option<String>| {
        o.as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };
    let (first_name, last_name) = (clean(&body.first_name), clean(&body.last_name));
    let name = match (&first_name, &last_name) {
        (Some(f), Some(l)) => format!("{f} {l}"),
        _ => body.name.trim().to_string(),
    };
    if name.is_empty() {
        return Err(AppError::Validation(
            "Le nom (ou prénom + nom) ne peut pas être vide".into(),
        ));
    }
    // Adresse structurée requise (créancier QR-bill type S, #213).
    let address = body.address.validate_required()?;

    // Validate IDE via kesh-core if provided
    let normalized_ide = match &body.ide_number {
        Some(ide) if !ide.trim().is_empty() => {
            let che = kesh_core::types::CheNumber::new(ide)
                .map_err(|e| AppError::Validation(format!("IDE invalide : {e}")))?;
            Some(che.as_str().to_string())
        }
        _ => None,
    };

    get_or_init_state(&state).await?;
    let mut tx = state.pool.begin().await.map_err(map_db_error)?;
    let result = async {
        let locked = lock_state_at_step(&mut tx, 5, true).await?;
        update_company_coordinates_in_tx(
            &mut tx,
            &current_user,
            CompanyCoordinates {
                name,
                first_name,
                last_name,
                address,
                ide_number: normalized_ide,
            },
        )
        .await?;
        complete_step(
            &mut tx,
            &current_user,
            &locked,
            locked.ui_mode,
            "coordinates",
        )
        .await
    }
    .await;
    conclude_step(&state.pool, tx, result).await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankAccountRequest {
    pub bank_name: String,
    pub iban: String,
    pub qr_iban: Option<String>,
}

/// POST /api/v1/onboarding/bank-account — step 6→7
///
/// Story 15-7a2 : une transaction `onboarding_state → companies →
/// bank_accounts` ; `bank_account.created`, `bank_account.updated` ou rien
/// selon [`UpsertPrimaryOutcome`], puis l'étape. Aucun IBAN en clair dans la
/// trace : seulement `iban_present` / `iban_changed` (AC 9).
pub async fn set_bank_account(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
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

    get_or_init_state(&state).await?;
    let mut tx = state.pool.begin().await.map_err(map_db_error)?;
    let result = async {
        let locked = lock_state_at_step(&mut tx, 6, true).await?;
        // Société verrouillée DANS la transaction (Pattern 5 : companies avant
        // bank_accounts) — et non lue sur le pool : un prélèvement du pool pendant
        // que la transaction tient sa connexion attendrait sous un pool saturé.
        let company = lock_company(&mut tx).await?;
        let outcome = bank_accounts::upsert_primary_in_tx(
            &mut tx,
            NewBankAccount {
                company_id: company.id,
                bank_name,
                iban: iban.as_str().to_string(),
                qr_iban: normalized_qr,
                is_primary: true,
            },
        )
        .await?;
        audit_bank_account_upsert(&mut tx, &current_user, &outcome).await?;
        complete_step(
            &mut tx,
            &current_user,
            &locked,
            locked.ui_mode,
            "bank_account",
        )
        .await
    }
    .await;
    conclude_step(&state.pool, tx, result).await
}

/// Trace l'upsert du compte bancaire principal de l'onboarding : créé, modifié
/// ou rien (`Unchanged`, AC 3). Formes de `routes/bank_accounts.rs` (création,
/// modification), avec `"trigger": "onboarding"` et les booléens
/// `iban_changed` / `qr_iban_changed` — jamais l'IBAN lui-même.
async fn audit_bank_account_upsert(
    tx: &mut Tx<'_>,
    user: &CurrentUser,
    outcome: &UpsertPrimaryOutcome,
) -> Result<(), AppError> {
    match outcome {
        UpsertPrimaryOutcome::Created(created) => {
            audit_log::insert_in_tx(
                tx,
                NewAuditLogEntry::from_current_user(
                    user,
                    "bank_account.created",
                    "bank_account",
                    created.id,
                    Some(json!({
                        "bank_account_id": created.id,
                        "is_primary": created.is_primary,
                        "iban_present": true,
                        "qr_iban_present": created.qr_iban.is_some(),
                        "journal_account_id": created.journal_account_id,
                    })),
                ),
            )
            .await?;
        }
        UpsertPrimaryOutcome::Updated { before, after } => {
            let snapshot = |b: &BankAccount| {
                json!({
                    "bank_name": b.bank_name,
                    "iban_present": true,
                    "qr_iban_present": b.qr_iban.is_some(),
                    "is_primary": b.is_primary,
                    "journal_account_id": b.journal_account_id,
                    "version": b.version,
                })
            };
            audit_log::insert_in_tx(
                tx,
                NewAuditLogEntry::from_current_user(
                    user,
                    "bank_account.updated",
                    "bank_account",
                    after.id,
                    Some(json!({
                        "bank_account_id": after.id,
                        "trigger": "onboarding",
                        "iban_changed": before.iban != after.iban,
                        "qr_iban_changed": before.qr_iban != after.qr_iban,
                        "before": snapshot(before),
                        "after": snapshot(after),
                    })),
                ),
            )
            .await?;
        }
        UpsertPrimaryOutcome::Unchanged(_) => {}
    }
    Ok(())
}

/// POST /api/v1/onboarding/skip-bank — step 6→7 without creating bank account
pub async fn skip_bank(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<OnboardingResponse>, AppError> {
    get_or_init_state(&state).await?;
    let mut tx = state.pool.begin().await.map_err(map_db_error)?;
    let result = async {
        let locked = lock_state_at_step(&mut tx, 6, true).await?;
        complete_step(&mut tx, &current_user, &locked, locked.ui_mode, "skip_bank").await
    }
    .await;
    conclude_step(&state.pool, tx, result).await
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
///
/// ORDRE DES VERROUS (cf. `docs/MULTI-TENANT-SCOPING-PATTERNS.md`, Pattern 5) :
/// ce handler prend trois `FOR UPDATE` successifs, dans l'ordre
/// `onboarding_state → companies → accounts`. Cet ordre est une **convention
/// de fréquence** : il réduit les interblocages, il ne les exclut pas — la
/// défense est le rejeu (Story 15-5e2).
///
/// KF-002-H-002 (#43) closed 2026-05-03 : la fonction est enveloppée dans
/// l'enveloppe `AppError` [`crate::retry::retry_app_on_deadlock`] (Story
/// 15-5e2 — jusque-là la primitive, à prédicat écrit en ligne), qui reconnaît
/// ER_LOCK_DEADLOCK (1213) et rejoue la tentative jusqu'à
/// `DEFAULT_MAX_DEADLOCK_ATTEMPTS` fois (3) avec backoff exponentiel. Les
/// business errors (`OnboardingStepAlreadyCompleted`, `Validation`, etc.) ne
/// sont PAS retryables et passent immédiatement.
pub async fn finalize(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<OnboardingResponse>, AppError> {
    // KF-002-H-002 (#43) : la closure ci-dessous est rappelée intégralement
    // si MariaDB rollback la tx pour deadlock (cycle de locks détecté avec
    // une autre tx). Le rollback est implicite côté DB ; côté Rust on ne
    // garde rien — chaque retry refait `pool.begin()`. La fermeture ne prête
    // que des références (`&state.pool`, `&current_user`) : elle est `Fn` et
    // non `FnOnce`, sans clone par tentative (revue P1 de la 15-5e2, B-5).
    crate::retry::retry_app_on_deadlock("onboarding::finalize", || {
        finalize_inner(&state.pool, &current_user)
    })
    .await
    .map(Json)
}

/// Logique transactionnelle de `finalize()`. Extraite pour être enveloppable
/// par [`crate::retry::retry_app_on_deadlock`] — la fermeture rejouée doit
/// pouvoir relancer toute la tx
/// (BEGIN → SELECT FOR UPDATE → … → COMMIT) après un rollback déclenché par
/// MariaDB sur deadlock.
///
/// **Idempotence requise** : la fonction est rappelée à l'identique en cas
/// de retry. C'est sûr ici car (a) le `step_completed == 8` court-circuite
/// proprement (return Ok early), (b) `INSERT IGNORE` sur invoice_settings
/// et vat_rates est idempotent, (c) `create_if_absent_in_tx` sur fiscal_year
/// est idempotent, (d) l'`UPDATE` final ne se déclenche que si step était
/// 7 → bumpe à 8 ou laisse no-op si quelqu'un d'autre a déjà bumpé.
///
/// **Audit (Story 15-7a2)** : les entrées s'écrivent DANS cette fonction, donc
/// dans la tentative rejouée : le booléen « inséré » des réglages et la liste
/// des taux insérés sont ceux de la tentative qui commite — l'audit d'un essai
/// annulé disparaît avec lui. Ordre : `company_invoice_settings.created`,
/// `vat_rate.created` (ordre du seed), `fiscal_year.created`, l'étape.
async fn finalize_inner(
    pool: &sqlx::MySqlPool,
    current_user: &CurrentUser,
) -> Result<OnboardingResponse, AppError> {
    // F2/F3/F4 CRITICAL FIX: Pessimistic locking strategy.
    // 1. Lock onboarding_state (serializes all finalize() calls on same session)
    // 2. Check state is still at step 7 or 8 (prevents TOCTOU on onboarding progression)
    // 3. Lock company row (prevents deletion during finalize)
    // 4. Proceed with insert_with_defaults() with guaranteed exclusive access
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let onboarding = match onboarding::lock_state_in_tx(&mut tx).await {
        Ok(Some(row)) => row,
        Ok(None) => {
            best_effort_rollback(tx).await;
            return Err(AppError::Internal(
                "onboarding_state absent sous verrou pendant finalize".into(),
            ));
        }
        Err(e) => {
            best_effort_rollback(tx).await;
            return Err(AppError::Database(e));
        }
    };

    // Reject demo path finalize (demo is finalized via seed_demo)
    if onboarding.is_demo {
        best_effort_rollback(tx).await;
        return Err(AppError::OnboardingStepAlreadyCompleted);
    }

    // Allow idempotent retry if already finalized (step == 8)
    if onboarding.step_completed < 7 || onboarding.step_completed > 8 {
        best_effort_rollback(tx).await;
        return Err(AppError::OnboardingStepAlreadyCompleted);
    }

    // If already finalized, release lock and return (idempotent).
    // P17: this path is read-only — rollback releases the FOR UPDATE lock
    // without writing an empty commit record. Returned snapshot is the row
    // observed at lock acquisition; under FOR UPDATE no concurrent writer can
    // have changed it before we release. Story 15-7a2 : il ne mute rien, il
    // n'écrit donc aucune trace.
    if onboarding.step_completed == 8 {
        best_effort_rollback(tx).await;
        return response_with_stub(pool, onboarding).await;
    }

    // F3 CRITICAL FIX: Lock company row before insert_with_defaults()
    // Prevents concurrent deletion between our check and INSERT INTO company_invoice_settings.
    // R2-001 Fix: Add explicit rollback on error
    // P5: ORDER BY id for deterministic row selection. v0.1 is mono-tenant so the
    // result is unambiguous, but explicit ordering matches Pattern 5 lock-discipline
    // and protects against multi-tenant drift in dev/test DBs.
    let company = match lock_company(&mut tx).await {
        Ok(c) => c,
        Err(e) => {
            best_effort_rollback(tx).await;
            return Err(e);
        }
    };

    // Pre-fill invoice settings with default accounts (1100, 3000).
    // Uses INSERT IGNORE pattern for database-level idempotency.
    // Account lookups use SELECT FOR UPDATE to prevent concurrent deletes.
    // F2/F3/F4: Transaction-level variant keeps account locks within this transaction,
    // preserving company and onboarding_state locks until step update completes.
    // R2-002 Fix: Add explicit rollback on error.
    // P6-L4: insert_with_defaults_in_tx fails fast with DbError::InactiveOrInvalidAccounts
    // when accounts 1100/3000 are missing — surface that as an actionable 422 with
    // remediation guidance instead of a generic 500. The previous downstream NULL
    // re-check on `settings.default_*_account_id` was dead code after the fail-fast
    // change (P1-004 + P1-007); it has been removed in this pass.
    // Open follow-up: KF-002-CR-001 (issue #44) — fallback UI to add missing accounts
    // during onboarding so the user has a recovery path.
    let (settings, settings_inserted) =
        match kesh_db::repositories::company_invoice_settings::insert_with_defaults_in_tx(
            &mut tx, company.id,
        )
        .await
        {
            Ok(s) => s,
            Err(kesh_db::errors::DbError::InactiveOrInvalidAccounts) => {
                best_effort_rollback(tx).await;
                return Err(AppError::Validation(
                "Impossible de pré-remplir les comptes de facturation (1100, 3000 manquants du plan comptable). \
                 Veuillez ajouter ces comptes avant de finaliser l'onboarding.".into(),
            ));
            }
            Err(e) => {
                best_effort_rollback(tx).await;
                return Err(AppError::Database(e));
            }
        };

    // Story 7.2 (KF-003) — seed des 4 taux TVA suisses 2024+ pour la nouvelle
    // company, dans la même transaction (atomicité avec invoice_settings et
    // fiscal_year ; rollback global si l'un échoue). INSERT IGNORE en interne
    // → idempotent sous re-finalize. Story 15-7a2 : seuls les taux RÉELLEMENT
    // insérés sont rendus, et tracés.
    let inserted_rates =
        match kesh_db::repositories::vat_rates::seed_default_swiss_rates_in_tx(&mut tx, company.id)
            .await
        {
            Ok(rates) => rates,
            Err(e) => {
                best_effort_rollback(tx).await;
                return Err(AppError::Database(e));
            }
        };

    let seeded = audit_finalize_seed(
        &mut tx,
        current_user,
        settings_inserted.then_some(&settings),
        &inserted_rates,
    )
    .await;
    if let Err(e) = seeded {
        best_effort_rollback(tx).await;
        return Err(e);
    }

    // Story 3.7 AC #13 — auto-create fiscal_year for current calendar year if
    // none exists (Path B). L'INSERT atomique anti-TOCTOU
    // (`create_if_absent_in_tx`) garantit l'idempotence sous finalize concurrent.
    // Audit log inséré uniquement si la création a effectivement eu lieu — par
    // le dépôt lui-même : `finalize` n'écrit PAS sa propre entrée d'exercice
    // (Story 15-7a2, AC 7 ; son attribution par `::user` est la dette #431).
    let year = Utc::now().naive_utc().date().year();
    let fy_name = format!("Exercice {year}");
    let fy_start = chrono::NaiveDate::from_ymd_opt(year, 1, 1).expect("valid date");
    let fy_end = chrono::NaiveDate::from_ymd_opt(year, 12, 31).expect("valid date");
    let new_fy = NewFiscalYear {
        company_id: company.id,
        name: fy_name,
        start_date: fy_start,
        end_date: fy_end,
    };
    if let Err(e) = kesh_db::repositories::fiscal_years::create_if_absent_in_tx(
        &mut tx,
        current_user.user_id,
        new_fy,
    )
    .await
    {
        best_effort_rollback(tx).await;
        return Err(AppError::Database(e));
    }

    // Mark onboarding as complete while holding locks.
    // P15: under FOR UPDATE the singleton row cannot be modified by another tx,
    // so a 0-row UPDATE indicates the singleton was deleted (corruption), not an
    // optimistic-lock conflict. We still bump version for downstream observers.
    let rows = sqlx::query(
        "UPDATE onboarding_state SET step_completed = 8, version = version + 1 \
         WHERE singleton = TRUE",
    )
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        best_effort_rollback(tx).await;
        return Err(AppError::Database(kesh_db::errors::DbError::Invariant(
            "onboarding_state singleton row missing during finalize (FOR UPDATE lock should prevent this)"
                .into(),
        )));
    }

    // Story 15-7a2 — l'étape s'inscrit EN DERNIER (AC 2).
    if let Err(e) = onboarding::record_step_completed_in_tx(
        &mut tx,
        current_user.user_id,
        current_user.api_key_id,
        7,
        8,
        "finalize",
    )
    .await
    {
        best_effort_rollback(tx).await;
        return Err(AppError::Database(e));
    }

    // R2-003 Fix: Add explicit rollback on final SELECT error.
    // P1-H6: Use fetch_optional and handle None explicitly instead of fetch_one panic.
    // P6-L6: Use FOR UPDATE on the read-back so the SELECT is consistent with the
    // earlier locks under any isolation level, including non-default READ COMMITTED
    // production tunings. Pure cost: one extra IS-already-mine lock acquisition;
    // no extra contention since this tx already holds the row exclusively.
    // Story 15-7a2 (E-1 de la revue de la 15-7a1) : par `lock_state_in_tx`, la
    // requête du dépôt, et non une copie en ligne.
    let updated = match onboarding::lock_state_in_tx(&mut tx).await {
        Ok(Some(row)) => row,
        Ok(None) => {
            best_effort_rollback(tx).await;
            return Err(AppError::Database(kesh_db::errors::DbError::Invariant(
                "onboarding_state row disappeared after update (FOR UPDATE lock should prevent this)"
                    .into(),
            )));
        }
        Err(e) => {
            best_effort_rollback(tx).await;
            return Err(AppError::Database(e));
        }
    };

    tx.commit().await.map_err(map_db_error)?;
    response_with_stub(pool, updated).await
}

/// Trace ce que `finalize` a **réellement inséré** (Story 15-7a2, AC 5) :
/// `company_invoice_settings.created` si la ligne de réglages est neuve, puis
/// un `vat_rate.created` par taux inséré, dans l'ordre du seed.
async fn audit_finalize_seed(
    tx: &mut Tx<'_>,
    user: &CurrentUser,
    inserted_settings: Option<&CompanyInvoiceSettings>,
    inserted_rates: &[VatRate],
) -> Result<(), AppError> {
    if let Some(s) = inserted_settings {
        // La table est clée par société : `entity_id = company_id`. Clés en
        // snake_case (convention des entrées neuves) — `.updated`, écrit par
        // `settings_snapshot_json`, est en camelCase par héritage.
        audit_log::insert_in_tx(
            tx,
            NewAuditLogEntry::from_current_user(
                user,
                "company_invoice_settings.created",
                "company_invoice_settings",
                s.company_id,
                Some(json!({
                    "company_id": s.company_id,
                    "invoice_number_format": s.invoice_number_format,
                    "default_receivable_account_id": s.default_receivable_account_id,
                    "default_revenue_account_id": s.default_revenue_account_id,
                    "default_payable_account_id": s.default_payable_account_id,
                    "default_rounding_account_id": s.default_rounding_account_id,
                    "default_discount_account_id": s.default_discount_account_id,
                    "default_bank_fees_account_id": s.default_bank_fees_account_id,
                    "default_bad_debt_account_id": s.default_bad_debt_account_id,
                })),
            ),
        )
        .await?;
    }
    for rate in inserted_rates {
        // Forme de `routes/vat.rs` (création d'un taux).
        audit_log::insert_in_tx(
            tx,
            NewAuditLogEntry::from_current_user(
                user,
                "vat_rate.created",
                "vat_rate",
                rate.id,
                Some(json!({
                    "vat_rate_id": rate.id,
                    "category": rate.category,
                    "rate": rate.rate.to_string(),
                    "valid_from": rate.valid_from,
                    "valid_to": rate.valid_to,
                })),
            ),
        )
        .await?;
    }
    Ok(())
}

// --- Helpers ---

/// La transaction d'une route d'onboarding.
type Tx<'a> = sqlx::Transaction<'a, sqlx::MySql>;

/// Retourne l'état d'onboarding existant ou en crée un nouveau.
async fn get_or_init_state(
    state: &AppState,
) -> Result<kesh_db::entities::OnboardingState, AppError> {
    match onboarding::get_state(&state.pool).await? {
        Some(s) => Ok(s),
        None => Ok(onboarding::init_state(&state.pool).await?),
    }
}

/// Verrouille l'état d'onboarding **en premier** dans la transaction et
/// **revérifie sous verrou** l'étape attendue — Story 15-7a2 (AC 8.2, choix
/// C-15-7-12), sur la primitive [`onboarding::lock_state_in_tx`] (15-7a1).
///
/// Rend l'état verrouillé ; `OnboardingStepAlreadyCompleted` si l'étape
/// diffère (ou si `require_not_demo` et l'installation est de démonstration) ;
/// `Internal` si la ligne manque. ⚠️ Reçoit `tx` par référence : elle **ne peut
/// pas** annuler — l'appelant annule ([`conclude_step`]).
///
/// La lecture non verrouillée de la garde disparaît : le perdant d'une course
/// à la même étape reçoit ici 400 `ONBOARDING_STEP_ALREADY_COMPLETED` (et non
/// plus 409 `OPTIMISTIC_LOCK_CONFLICT` au moment de `update_step`).
async fn lock_state_at_step(
    tx: &mut Tx<'_>,
    expected: i32,
    require_not_demo: bool,
) -> Result<OnboardingState, AppError> {
    let locked = onboarding::lock_state_in_tx(tx)
        .await?
        .ok_or_else(|| AppError::Internal("onboarding_state absent sous verrou".into()))?;
    if locked.step_completed != expected || (require_not_demo && locked.is_demo) {
        return Err(AppError::OnboardingStepAlreadyCompleted);
    }
    Ok(locked)
}

/// Franchit l'étape verrouillée (`n → n+1`, `version` verrouillée) et
/// l'inscrit au journal d'audit — **en dernier**, après les entrées de domaine
/// (AC 2). Ne commite pas.
async fn complete_step(
    tx: &mut Tx<'_>,
    user: &CurrentUser,
    locked: &OnboardingState,
    ui_mode: Option<UiMode>,
    step: &'static str,
) -> Result<OnboardingState, AppError> {
    let to = locked.step_completed + 1;
    let updated =
        onboarding::update_step_in_tx(tx, to, locked.is_demo, ui_mode, locked.version).await?;
    onboarding::record_step_completed_in_tx(
        tx,
        user.user_id,
        user.api_key_id,
        locked.step_completed,
        to,
        step,
    )
    .await?;
    Ok(updated)
}

/// Conclut la transaction unique d'une route d'étape : `COMMIT` puis réponse
/// (`response_with_stub` **après** commit) si le corps a réussi ; sinon
/// annulation, et l'erreur d'origine. Une route refusée ne laisse ni mutation
/// ni trace (AC 8).
async fn conclude_step(
    pool: &sqlx::MySqlPool,
    tx: Tx<'_>,
    result: Result<OnboardingState, AppError>,
) -> Result<Json<OnboardingResponse>, AppError> {
    match result {
        Ok(updated) => {
            tx.commit().await.map_err(map_db_error)?;
            Ok(Json(response_with_stub(pool, updated).await?))
        }
        Err(e) => {
            best_effort_rollback(tx).await;
            Err(e)
        }
    }
}

/// La liste de colonnes de `Company`, suivie de la fin de requête donnée.
macro_rules! company_select {
    ($tail:literal) => {
        concat!(
            "SELECT id, name, first_name, last_name, address, address_street, address_building, \
             address_postal_code, address_city, address_country, ide_number, org_type, accounting_language, \
             instance_language, email, phone, website, is_stub, books_locked_through, version, created_at, updated_at \
             FROM companies ",
            $tail
        )
    };
}

// P5: ORDER BY id for deterministic row selection (Pattern 5 lock-discipline).
const COMPANY_SELECT_FOR_UPDATE: &str = company_select!("ORDER BY id LIMIT 1 FOR UPDATE");
const COMPANY_BY_ID_FOR_UPDATE: &str = company_select!("WHERE id = ? FOR UPDATE");

/// Verrouille la company (première et unique) dans la transaction.
/// `Internal` si aucune company n'existe.
async fn lock_company(tx: &mut Tx<'_>) -> Result<Company, AppError> {
    sqlx::query_as::<_, Company>(COMPANY_SELECT_FOR_UPDATE)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            AppError::Internal(
                "Aucune company en base (company supprimée pendant onboarding ?)".into(),
            )
        })
}

/// `CompanyUpdate` reconstruit à l'identique depuis la ligne verrouillée :
/// `companies::update_in_tx` est un full-replace, l'appelant ne change que les
/// champs qu'il vise (AC 3).
fn company_update_of(c: &Company) -> CompanyUpdate {
    CompanyUpdate {
        name: c.name.clone(),
        first_name: c.first_name.clone(),
        last_name: c.last_name.clone(),
        address_structured: c.structured_address(),
        ide_number: c.ide_number.clone(),
        org_type: c.org_type,
        accounting_language: c.accounting_language,
        instance_language: c.instance_language,
        email: c.email.clone(),
        phone: c.phone.clone(),
        website: c.website.clone(),
    }
}

/// Écrit `company.updated` (`entity_type = "company"`, `entity_id` = la
/// société), `details = {"before", "after"}` réduits aux champs de la route.
async fn audit_company_updated(
    tx: &mut Tx<'_>,
    user: &CurrentUser,
    company_id: i64,
    before: serde_json::Value,
    after: serde_json::Value,
) -> Result<(), AppError> {
    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::from_current_user(
            user,
            "company.updated",
            "company",
            company_id,
            Some(json!({ "before": before, "after": after })),
        ),
    )
    .await?;
    Ok(())
}

/// Modifie la société verrouillée par `companies::update_in_tx` (court-circuit
/// no-op KF-004) et trace `company.updated` **si et seulement si** `version` a
/// bougé ; `project` réduit une société aux champs que la route écrit. Rend la
/// société après écriture.
async fn update_company_in_tx(
    tx: &mut Tx<'_>,
    user: &CurrentUser,
    locked: &Company,
    changes: CompanyUpdate,
    project: fn(&Company) -> serde_json::Value,
) -> Result<Company, AppError> {
    let updated = companies::update_in_tx(tx, locked.id, locked.version, changes).await?;
    if updated.version != locked.version {
        audit_company_updated(tx, user, locked.id, project(locked), project(&updated)).await?;
    }
    Ok(updated)
}

/// S'assure qu'une company existe avec la bonne `instance_language`, dans la
/// transaction de `language` (après le verrou d'état).
///
/// `SELECT … FOR UPDATE` contre la race TOCTOU (deux requêtes créant chacune
/// une company). Aucune société ⇒ insertion d'une société provisoire et
/// `company.created` ; sinon `company.updated {instance_language}` si la langue
/// change, rien sinon.
async fn ensure_company_with_language_in_tx(
    tx: &mut Tx<'_>,
    user: &CurrentUser,
    lang: Language,
) -> Result<(), AppError> {
    let existing = sqlx::query_as::<_, Company>(COMPANY_SELECT_FOR_UPDATE)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?;

    match existing {
        None => {
            // Story v011-2 : placeholder marqué `is_stub = TRUE` (cohérent avec
            // le stub du bootstrap, mêmes constantes partagées). `set_coordinates`
            // lèvera le drapeau (`clear_stub_in_tx`). Ce chemin ne se déclenche que si aucune
            // company n'existe (rare hors bootstrap, ex. après une remise à zéro).
            let result = sqlx::query(
                "INSERT INTO companies \
                 (name, address, org_type, accounting_language, instance_language, is_stub) \
                 VALUES (?, ?, ?, ?, ?, TRUE)",
            )
            .bind(crate::auth::bootstrap::STUB_COMPANY_NAME)
            .bind(crate::auth::bootstrap::STUB_COMPANY_ADDRESS)
            .bind(OrgType::Independant)
            .bind(Language::Fr)
            .bind(lang)
            .execute(&mut **tx)
            .await
            .map_err(map_db_error)?;
            let id = i64::try_from(result.last_insert_id())
                .ok()
                .filter(|id| *id > 0)
                .ok_or_else(|| {
                    AppError::Internal("last_insert_id invalide après INSERT companies".into())
                })?;
            audit_log::insert_in_tx(
                tx,
                NewAuditLogEntry::from_current_user(
                    user,
                    "company.created",
                    "company",
                    id,
                    Some(json!({ "instance_language": lang, "is_stub": true })),
                ),
            )
            .await?;
        }
        Some(company) => {
            let mut changes = company_update_of(&company);
            changes.instance_language = lang;
            update_company_in_tx(
                tx,
                user,
                &company,
                changes,
                |c| json!({ "instance_language": c.instance_language }),
            )
            .await?;
        }
    }
    Ok(())
}

/// Les coordonnées que la route `coordinates` écrit, validées.
struct CompanyCoordinates {
    name: String,
    first_name: Option<String>,
    last_name: Option<String>,
    address: kesh_db::entities::address::StructuredAddress,
    ide_number: Option<String>,
}

/// Les seuls champs que `coordinates` écrit, plus `version` (AC 1, 3) — la
/// colonne combinée `address`, dérivée des cinq champs, n'y figure pas.
fn coordinates_snapshot(c: &Company) -> serde_json::Value {
    json!({
        "name": c.name,
        "first_name": c.first_name,
        "last_name": c.last_name,
        "address_street": c.address_street,
        "address_building": c.address_building,
        "address_postal_code": c.address_postal_code,
        "address_city": c.address_city,
        "address_country": c.address_country,
        "ide_number": c.ide_number,
        "is_stub": c.is_stub,
        "version": c.version,
    })
}

/// Met à jour les coordonnées de la company dans la transaction de la route —
/// Story 15-7a2 (AC 3, règle composée ; choix C-15-7-15, C-15-7-21).
///
/// `companies::update_in_tx` (court-circuit no-op KF-004) avec la `version`
/// verrouillée, **puis** `companies::clear_stub_in_tx` : l'utilisateur a
/// renseigné ses vraies coordonnées, la société n'est plus provisoire — même
/// quand elles sont identiques au placeholder. La société est **relue** après
/// les deux écritures ; `company.updated` s'écrit ssi `version` a bougé à
/// l'`update_in_tx` **ou** le drapeau a été levé.
async fn update_company_coordinates_in_tx(
    tx: &mut Tx<'_>,
    user: &CurrentUser,
    coords: CompanyCoordinates,
) -> Result<(), AppError> {
    let company = lock_company(tx).await?;
    let mut changes = company_update_of(&company);
    changes.name = coords.name;
    changes.first_name = coords.first_name;
    changes.last_name = coords.last_name;
    changes.address_structured = coords.address;
    changes.ide_number = coords.ide_number;
    let updated = companies::update_in_tx(tx, company.id, company.version, changes).await?;
    let stub_cleared = companies::clear_stub_in_tx(tx, company.id).await?;
    if updated.version != company.version || stub_cleared {
        let after = sqlx::query_as::<_, Company>(COMPANY_BY_ID_FOR_UPDATE)
            .bind(company.id)
            .fetch_one(&mut **tx)
            .await
            .map_err(map_db_error)?;
        audit_company_updated(
            tx,
            user,
            company.id,
            coordinates_snapshot(&company),
            coordinates_snapshot(&after),
        )
        .await?;
    }
    Ok(())
}
