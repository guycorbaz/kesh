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
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use serde_json::json;

use kesh_db::entities::audit_log::{AUDIT_ENTITY_ID_NONE, NewAuditLogEntry};
use kesh_db::entities::onboarding::UiMode;
use kesh_db::entities::{
    BankAccount, Company, CompanyUpdate, Language, NewBankAccount, NewFiscalYear, OnboardingState,
    OrgType, VatRate,
};
use kesh_db::errors::{
    DefaultInvoiceAccountCause, DefaultInvoiceSetting, UnavailableDefaultInvoiceAccount,
    map_db_error,
};
use kesh_db::repositories::bank_accounts::{self, UpsertPrimaryOutcome};
use kesh_db::repositories::company_invoice_settings::{
    DefaultAccountsOutcome, settings_snapshot_json,
};
use kesh_db::repositories::{audit_log, companies, onboarding};

use crate::AppState;
use crate::audit::AuditActor;
use crate::dates_recues::borner;
use crate::errors::{AppError, t};
use crate::middleware::auth::CurrentUser;

/// P1-H1: Helper for graceful transaction rollback
/// Rollback errors are best-effort cleanup; don't fail the request if rollback fails
async fn best_effort_rollback(tx: sqlx::Transaction<'_, sqlx::MySql>) {
    if let Err(e) = tx.rollback().await {
        tracing::warn!("Transaction rollback failed (best-effort cleanup): {}", e);
        // Continue anyway — connection pool handles cleanup
    }
}

/// Installe les prompts standard du serveur MCP dans chaque société (Story
/// 32-10a-i, AC-i11) — à la fin d'un point d'entrée qui crée une société,
/// **après** la validation de sa transaction. **Non fatal** : l'opération
/// est faite, elle répond comme avant ; un échec est journalisé, sans texte de
/// prompt, et les standards reviennent au démarrage suivant (`main.rs`).
async fn amorcer_prompts_standard(pool: &sqlx::MySqlPool) {
    use kesh_db::entities::mcp_prompt_standards::STANDARDS;
    match kesh_db::repositories::mcp_prompts::amorcer_standards(pool, STANDARDS).await {
        Ok(r) => tracing::info!(
            societes = r.societes,
            creations = r.creations,
            mises_a_jour = r.mises_a_jour,
            retraits = r.retraits,
            "prompts standard du serveur MCP amorcés"
        ),
        Err(e) => tracing::error!(
            "amorçage des prompts standard du serveur MCP en échec (non fatal, repris au \
             prochain démarrage) : {e}"
        ),
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
    /// sur DB vide, par le wizard avant complétion, ou remise à l'état
    /// provisoire par la remise à zéro — Story 15-7b2). Le frontend l'utilise
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
///
/// Story 32-10a-i (AC-i11) : sur une base sans société, cette route crée la
/// société provisoire ; après la validation de la transaction, elle installe
/// donc les prompts standard du serveur MCP ([`amorcer_prompts_standard`],
/// non fatal, idempotent).
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
    let reponse = conclude_step(&state.pool, tx, result).await?;
    amorcer_prompts_standard(&state.pool).await;
    Ok(reponse)
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
    // Story 35-5b (#929) — l'exercice de démonstration est celui de l'année du
    // jour de Zurich.
    kesh_seed::seed_demo(
        &state.pool,
        &state.config.locale,
        (current_user.user_id, current_user.api_key_id),
        state.horloge.aujourd_hui(),
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

/// POST /api/v1/onboarding/reset — remet l'installation à zéro et **inscrit son
/// geste** au journal d'audit (Story 15-7b2, #434, #279, #528).
///
/// Tout l'effacement est dans `kesh_seed::reset_demo` : **une** transaction
/// par essai, rejouée sur interblocage, qui verrouille `onboarding_state`,
/// évalue **sous ce verrou** les trois gardes — étape ≥ 7 (finalisation,
/// irréversible) ⇒ `400 ONBOARDING_STEP_ALREADY_COMPLETED` ; installation de
/// production au-delà de l'étape 2 ⇒ `403 ONBOARDING_RESET_FORBIDDEN`, même
/// avec le drapeau ; au-delà de l'étape 2 sans `KESH_PRODUCTION_RESET` ⇒ `403`
/// —, vide toutes les données de la société, la remet à l'état provisoire en
/// place, rattache les principaux orphelins, remet l'état à zéro et écrit
/// `installation.reset`. Le handler ne porte ni transaction ni garde : la
/// seule copie des gardes est celle de la transaction qui efface. **Après**
/// elle (Story 32-10a-i, AC-i11), il réinstalle les prompts standard du
/// serveur MCP, que la remise à zéro a vidés avec les données de la société
/// ([`amorcer_prompts_standard`] : une transaction par société, hors de celle
/// de `reset_demo`, sans entrée d'audit ; non fatal — en cas d'échec, ils
/// reviennent au démarrage suivant).
///
/// `KESH_PRODUCTION_RESET` : une démonstration est toujours à l'étape 3, si
/// bien que **toute sortie d'une démonstration** exige ce drapeau — posé par
/// l'exploitant le temps de la réinitialisation, puis retiré. Une installation
/// de production n'est jamais remise à zéro au-delà de l'étape 2.
///
/// `get_or_init_state` garantit la ligne d'état avant l'appel (sans elle, le
/// verrou rendrait `None`, que `reset_demo` refuse en `Invariant`) ; la réponse
/// relit l'état et la société (`isStub` vaut `true` après une remise à zéro).
/// Route d'administration, inaccessible aux jetons d'API (`require_not_pat`).
pub async fn reset(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<OnboardingResponse>, AppError> {
    let _ = get_or_init_state(&state).await?;
    let production_reset_allowed = env_flag_enabled("KESH_PRODUCTION_RESET");

    kesh_seed::reset_demo(
        &state.pool,
        (current_user.user_id, current_user.api_key_id),
        production_reset_allowed,
    )
    .await
    .map_err(|e| match e {
        kesh_seed::SeedError::StepAlreadyCompleted => AppError::OnboardingStepAlreadyCompleted,
        kesh_seed::SeedError::ResetForbidden => AppError::OnboardingResetForbidden,
        kesh_seed::SeedError::Db(d) => AppError::Database(d),
        // Inatteignable par `reset_demo` (toute erreur sqlx y passe par
        // `map_db_error`) : 500 générique.
        kesh_seed::SeedError::Sqlx(e) => AppError::Internal(format!("Reset demo failed: {e}")),
    })?;
    amorcer_prompts_standard(&state.pool).await;

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
    // Story 48-15 (#1334) — le prénom et le nom, **chacun quand il est fourni**,
    // sont bornés à leur colonne (`companies.first_name` / `last_name`,
    // `VARCHAR(70)`), avant le nom composé : un prénom seul (le nom de la société
    // vient alors de `name`) partait sinon à la base sans contrôle et revenait en
    // `DATA_LENGTH_OR_RANGE`, qui ne nomme pas le champ.
    if let Some(f) = &first_name {
        crate::helpers::validate_text_len(f, crate::helpers::PERSON_NAME_PART_MAX, "Prénom")?;
    }
    if let Some(l) = &last_name {
        crate::helpers::validate_text_len(l, crate::helpers::PERSON_NAME_PART_MAX, "Nom")?;
    }
    let name = match (&first_name, &last_name) {
        (Some(f), Some(l)) => format!("{f} {l}"),
        _ => body.name.trim().to_string(),
    };
    // Story 48-13 (#1226, D5) — le nom (composé « prénom nom » pour une personne
    // physique : c'est lui qui part au PDF) et l'adresse sont jugés par les règles
    // de la QR-facture, celles des routes de correction des réglages : vide au sens
    // de la QR, 70 caractères au plus et jeu de caractères pour le nom ; longueurs,
    // NPA et localité, jeu de caractères pour l'adresse. Sans quoi un nom ou une
    // adresse acceptés ici faisaient refuser la première QR-facture. Refus de forme
    // avant le verrou d'étape, comme avant.
    let name = super::companies::validate_company_name_for_qr(&name)?;
    // Adresse structurée requise (créancier QR-bill type S, #213).
    let address = body.address.validate_required_for_qr()?;

    // Validate IDE via kesh-core if provided. Le message de `kesh-core` nomme
    // déjà le champ (« Numéro IDE invalide : … ») : servi tel quel, sans second
    // préfixe (Story v017-6, #832).
    let normalized_ide = match &body.ide_number {
        Some(ide) if !ide.trim().is_empty() => {
            let che = kesh_core::types::CheNumber::new(ide)
                .map_err(|e| AppError::Validation(e.to_string()))?;
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
    /// Story 34-1 (#966) — description du compte. **Absent : inchangée** (un
    /// corps de la 0.17.5 n'efface rien) ; **`null` : effacée** ; sinon
    /// normalisée par la même fonction que `POST /bank-accounts`
    /// (C-e341-34, C-e341-2).
    #[serde(default, deserialize_with = "crate::helpers::double_option")]
    pub description: Option<Option<String>>,
    /// Story 34-1 (#966) — compte du plan lié. **Absent : inchangé** ;
    /// **`null` : effacé** ; sinon contrôlé comme à la création (existe, actif,
    /// `Asset | Liability`, imputable) et refusé s'il porte le rôle
    /// `Receivable` ou `Payable` (C-e341-3).
    #[serde(default, deserialize_with = "crate::helpers::double_option")]
    pub journal_account_id: Option<Option<i64>>,
}

/// POST /api/v1/onboarding/bank-account — step 6→7
///
/// Story 15-7a2 : une transaction `onboarding_state → companies →
/// bank_accounts` ; `bank_account.created`, `bank_account.updated` ou rien
/// selon [`UpsertPrimaryOutcome`], puis l'étape. Aucun IBAN en clair dans la
/// trace : seulement `iban_present` / `iban_changed` (AC 9).
///
/// Story 34-1 (#966) — description et compte lié, facultatifs (absent =
/// inchangé, `null` = effacé). **Ordre des refus** : comme à la création
/// (`routes::bank_accounts`), les contrôles de forme et du compte lié lisent
/// sur le **pool**, avant la transaction — banque vide, banque trop longue
/// (Story 34-2), numéro de carte complet dans le nom (Story 34-11a2), description,
/// numéro de carte complet dans la description (intégration de la 0.17.6,
/// raccord 3), IBAN, QR-IBAN, compte lié (règle d'un compte bancaire) ; un 400
/// de forme précède donc le refus d'étape. Dans la
/// transaction, après `lock_state_at_step` et `lock_company`, le refus d'un
/// compte de rôle `Receivable` / `Payable` (AC-4).
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
    // Story 34-2 (revue P1 E-1) : la borne de la colonne, comme à la création.
    super::bank_accounts::refuse_too_long_bank_name(&bank_name)?;
    // Story 34-11a2 (#972, C-e341-15, C-e341-19) — la garde du numéro de carte
    // complet atteint l'assistant : il ne passe pas par `validate_bank_input`.
    crate::routes::bank_accounts::refuse_card_number("bankName", &bank_name)?;

    // Story 34-1 (AC-2) : la fonction de la création, même borne, même message.
    // Une description présente mais vide ou faite d'espaces vaut `null`.
    let description = match &body.description {
        None => None,
        Some(d) => Some(super::bank_accounts::normalize_description(d.as_deref())?),
    };
    // Intégration de la 0.17.6 (C-e34a2-1, raccord 3) — la description de
    // l'assistant (Story 34-1) reçoit la garde du numéro complet, comme à la
    // création (`validate_bank_input`), après sa normalisation.
    if let Some(Some(d)) = description.as_ref() {
        crate::routes::bank_accounts::refuse_card_number("description", d)?;
    }

    // Validate IBAN via kesh-core — message servi tel quel, il nomme déjà le
    // champ (Story v017-6, #832).
    let iban =
        kesh_core::types::Iban::new(&body.iban).map_err(|e| AppError::Validation(e.to_string()))?;

    // Validate QR-IBAN via kesh-core if provided
    let normalized_qr = match &body.qr_iban {
        Some(qr) if !qr.trim().is_empty() => {
            let qr_iban = kesh_core::types::QrIban::new(qr).map_err(crate::util::refus_qr_iban)?;
            Some(qr_iban.as_iban().as_str().to_string())
        }
        _ => None,
    };

    // Story 34-1 (AC-2) : la fonction de la création — 404 inexistant ou
    // inactif, 400 hors `Asset | Liability`, 400 non imputable. Sur le pool,
    // avant la transaction, comme `create_bank_account`.
    let requested_ledger = body.journal_account_id.flatten();
    // Story 34-11a2 (#972) : l'assistant ne crée qu'un compte bancaire — la
    // règle d'avant (actif ou passif, lien facultatif).
    super::bank_accounts::validate_journal_account_id(
        &state.pool,
        current_user.company_id,
        kesh_db::entities::BankAccountKind::Bank,
        requested_ledger,
        true,
    )
    .await?;
    let details = bank_accounts::PrimaryDetails {
        description,
        journal_account_id: body.journal_account_id,
    };

    get_or_init_state(&state).await?;
    let mut tx = state.pool.begin().await.map_err(map_db_error)?;
    let result = async {
        let locked = lock_state_at_step(&mut tx, 6, true).await?;
        // Société verrouillée DANS la transaction (Pattern 5 : companies avant
        // bank_accounts) — et non lue sur le pool : un prélèvement du pool pendant
        // que la transaction tient sa connexion attendrait sous un pool saturé.
        let company = lock_company(&mut tx).await?;
        if let Some(ledger) = requested_ledger {
            refuse_future_claim_account(&mut tx, company.id, ledger).await?;
        }
        let outcome = bank_accounts::upsert_primary_in_tx(
            &mut tx,
            NewBankAccount {
                company_id: company.id,
                bank_name,
                iban: iban.as_str().to_string(),
                qr_iban: normalized_qr,
                is_primary: true,
            },
            details,
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

/// Refuse de lier le compte bancaire de l'assistant à un **futur compte de
/// créance** — Story 34-1 (#966, AC-4, C-e341-3).
///
/// À l'étape 6, les réglages de facturation n'existent pas encore : la
/// finalisation les créera en y désignant comme comptes débiteurs et
/// créanciers les comptes de **rôle** `Receivable` et `Payable`
/// (`company_invoice_settings::complete_default_accounts_in_tx`). Les comptes de
/// créance que lirait `claim_accounts_in_share_mode` sont donc vides, et un lien
/// vers le futur compte débiteurs passerait. Les rôles sont lus ici par la même
/// recherche que la finalisation (`accounts::find_singleton_role_in_tx`, source
/// unique), puis `refuse_if_ledger_is_claim_account` rend le refus de
/// `POST /bank-accounts` (`400 BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT`, même
/// rendu). ⚠️ La finalisation écarte en plus un compte de rôle **non imputable**
/// (Story 36-19, D6) ; ce garde-ci, plus strict, refuse le lien au compte qui
/// porte le rôle, imputable ou non.
///
/// ⛔ Lit le **rôle**, jamais le numéro (Story 14-3a).
///
/// **Sans verrou** (`RoleLookupLock::None`) : l'étape est sérialisée par
/// `lock_state_at_step` (`FOR UPDATE` sur `onboarding_state`), aucune
/// finalisation ne peut courir en même temps (elle attend l'étape 7), et la
/// finalisation reverrouille ces lignes en `FOR UPDATE`. Lu dans la
/// transaction, jamais sur le pool.
///
/// Entre l'étape 6 et la finalisation, `PUT /api/v1/accounts/{id}` peut poser le
/// rôle sur le compte qui vient d'être lié, ou `POST /api/v1/bank-accounts` lier
/// un compte de rôle. **Fermé côté finalisation depuis la Story 36-19** (D6,
/// refs #987) : un compte de rôle lié à un compte bancaire n'est jamais désigné —
/// refus pour le compte débiteurs, réglage laissé vide pour le compte
/// créanciers. La désignation de rôle elle-même (`PUT /api/v1/accounts/{id}`)
/// reste à #987.
async fn refuse_future_claim_account(
    tx: &mut Tx<'_>,
    company_id: i64,
    ledger: i64,
) -> Result<(), AppError> {
    use kesh_db::entities::AccountRole;
    use kesh_db::repositories::accounts::{RoleLookupLock, find_singleton_role_in_tx};
    use kesh_db::repositories::company_invoice_settings::ClaimAccounts;

    let receivable = find_singleton_role_in_tx(
        tx,
        company_id,
        AccountRole::Receivable,
        RoleLookupLock::None,
    )
    .await?;
    let payable =
        find_singleton_role_in_tx(tx, company_id, AccountRole::Payable, RoleLookupLock::None)
            .await?;
    let claims = ClaimAccounts {
        receivable,
        payable,
    };
    bank_accounts::refuse_if_ledger_is_claim_account(tx, company_id, ledger, &claims).await?;
    Ok(())
}

/// Trace l'upsert du compte bancaire principal de l'onboarding : créé, modifié
/// ou rien (`Unchanged`, AC 3). Formes de `routes/bank_accounts.rs` (création,
/// modification), avec `"trigger": "onboarding"` et les booléens
/// `iban_changed` / `qr_iban_changed` — jamais l'IBAN lui-même.
///
/// Story 34-1 (AC-5) : `description` dans la création (forme de
/// `create_bank_account`) et dans les deux instantanés de la modification.
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
                        "description": created.description,
                        "is_primary": created.is_primary,
                        // Story 34-11a1 (#972, AC-8) — lu ; vrai par construction
                        // (l'assistant exige l'IBAN, C-e341-19).
                        "iban_present": created.iban.is_some(),
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
                    "description": b.description,
                    "iban_present": b.iban.is_some(),
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

/// Corps facultatif de `POST /api/v1/onboarding/finalize` (Story 37-1, AC-11) :
/// les dates du premier exercice choisies dans l'assistant.
///
/// Corps absent ou champs absents : début au 1er janvier de l'année du jour du
/// serveur, fin à la veille du même jour un an plus tard
/// ([`kesh_core::accounting::exercice::fin_par_defaut`]) — sans début fourni,
/// le 31 décembre, comme avant l'AC-11. Les dates d'un exercice ne se modifient
/// plus une fois créé : c'est ici qu'elles se choisissent.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalizeRequest {
    /// Premier jour du premier exercice (`AAAA-MM-JJ`).
    #[serde(default)]
    pub fiscal_year_start_date: Option<chrono::NaiveDate>,
    /// Dernier jour du premier exercice (`AAAA-MM-JJ`).
    #[serde(default)]
    pub fiscal_year_end_date: Option<chrono::NaiveDate>,
}

/// Les dates et le nom du premier exercice, calculés **une fois**, avant la
/// transaction (Story 37-1, AC-11) : la fermeture rejouée de
/// [`crate::retry::retry_app_on_deadlock`] les reçoit par référence.
#[derive(Debug, Clone, PartialEq, Eq)]
struct PremierExercice {
    nom: String,
    debut: chrono::NaiveDate,
    fin: chrono::NaiveDate,
}

/// Refus des dates du premier exercice, gardé tel quel jusqu'à la lecture de
/// l'étape sous verrou (revue de code P2 de la 37-1, 37-1-CR-P1-3) : une
/// finalisation déjà faite ignore le corps (AC-11), de sorte que le refus ne
/// s'oppose qu'à une finalisation à faire. Clonable — la fermeture rejouée de
/// [`crate::retry::retry_app_on_deadlock`] le relit à chaque tentative —, à la
/// différence d'[`AppError`].
#[derive(Debug, Clone, PartialEq, Eq)]
enum RefusPremierExercice {
    /// 400 `VALIDATION_ERROR`, message déjà résolu.
    Validation(String),
    /// 500 — inatteignable avec des dates bornées, gardé pour ne rien masquer.
    Interne(String),
}

impl From<AppError> for RefusPremierExercice {
    fn from(e: AppError) -> Self {
        match e {
            AppError::Validation(m) => Self::Validation(m),
            AppError::Internal(m) => Self::Interne(m),
            autre => Self::Interne(autre.to_string()),
        }
    }
}

impl From<&RefusPremierExercice> for AppError {
    fn from(r: &RefusPremierExercice) -> Self {
        match r {
            RefusPremierExercice::Validation(m) => AppError::Validation(m.clone()),
            RefusPremierExercice::Interne(m) => AppError::Internal(m.clone()),
        }
    }
}

/// Résout les dates du premier exercice depuis le corps de `finalize` et le
/// jour du serveur. Refus en 400 (`VALIDATION_ERROR`) : une date hors de la
/// plage de la base (`borner`, y compris la fin calculée d'un début en 9999) ;
/// une fin qui ne suit pas strictement le début (`error-fiscal-year-dates-invalid`,
/// le message de la création d'un exercice). Aucune borne de durée : six ou
/// dix-huit mois sont admis.
fn premier_exercice(
    req: &FinalizeRequest,
    aujourd_hui: chrono::NaiveDate,
) -> Result<PremierExercice, AppError> {
    if let Some(d) = req.fiscal_year_start_date {
        borner(d, "fiscalYearStartDate")?;
    }
    if let Some(f) = req.fiscal_year_end_date {
        borner(f, "fiscalYearEndDate")?;
    }
    let debut = match req.fiscal_year_start_date {
        Some(d) => d,
        None => chrono::NaiveDate::from_ymd_opt(aujourd_hui.year(), 1, 1).ok_or_else(|| {
            AppError::Internal("1er janvier de l'année du jour non représentable".into())
        })?,
    };
    let fin = match req.fiscal_year_end_date {
        Some(f) => f,
        None => {
            // Début borné à 1000..=9999 : l'année suivante est toujours représentable.
            let f = kesh_core::accounting::exercice::fin_par_defaut(debut).ok_or_else(|| {
                AppError::Internal("fin par défaut du premier exercice non représentable".into())
            })?;
            borner(f, "fiscalYearEndDate")?;
            f
        }
    };
    if fin <= debut {
        return Err(AppError::Validation(t(
            "error-fiscal-year-dates-invalid",
            "Les dates de l'exercice sont invalides (la date de fin doit être strictement postérieure à la date de début)",
        )));
    }
    Ok(PremierExercice {
        nom: kesh_core::accounting::exercice::nom_par_defaut(debut, fin),
        debut,
        fin,
    })
}

/// POST /api/v1/onboarding/finalize — step 7→complete (Path B only)
/// Pose les comptes par défaut de facturation que le plan fournit, par **rôle** —
/// en créant la ligne des réglages, ou en **complétant** celle qu'un écran a créée
/// vide avant la fin de l'assistant, sans remplacer un compte déjà choisi (Story
/// 36-19, #1200 : `company_invoice_settings::complete_default_accounts_in_tx`).
///
/// Story 37-1 (AC-11) : corps facultatif [`FinalizeRequest`] — les dates du
/// premier exercice, résolues une fois avant la transaction ; leur refus (400
/// sans rien créer, étape inchangée) n'est rendu qu'après la lecture de l'étape
/// sous verrou : une finalisation déjà faite (`step_completed == 8`) ignore le
/// corps, dates invalides comprises, et rend l'état, comme avant (revue de code
/// P2, 37-1-CR-P1-3).
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
/// Once locked, only one finalize() can proceed. Les réglages ne s'écrivent que sur
/// une colonne vide (`COALESCE`), et pas du tout s'il n'y a rien à poser (Story
/// 36-19) : rejouée, la finalisation n'écrit rien de plus.
///
/// F1 CRITICAL VALIDATION: le compte débiteurs et le compte de produit par défaut
/// doivent être posés ; sinon refus `400 VALIDATION_ERROR` qui nomme chaque
/// réglage en défaut, sa cause et son remède (Story 36-19, D5).
///
/// ORDRE DES VERROUS (cf. `docs/MULTI-TENANT-SCOPING-PATTERNS.md`, Pattern 5) :
/// ce handler verrouille, dans l'ordre, `onboarding_state` (`FOR UPDATE`),
/// `companies` (`FOR UPDATE`), les comptes candidats (`FOR UPDATE` : les trois
/// rôles, puis les comptes que le plan marque), la ligne des réglages (`INSERT
/// IGNORE` puis `FOR UPDATE`), puis, en partagé, les comptes déjà désignés et les
/// comptes bancaires (Story 36-19 ; le détail est au doc-comment de
/// `company_invoice_settings::complete_default_accounts_in_tx`). Cet ordre est une **convention
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
    body: Option<Json<FinalizeRequest>>,
) -> Result<Json<OnboardingResponse>, AppError> {
    // KF-002-H-002 (#43) : la closure ci-dessous est rappelée intégralement
    // si MariaDB rollback la tx pour deadlock (cycle de locks détecté avec
    // une autre tx). Le rollback est implicite côté DB ; côté Rust on ne
    // garde rien — chaque retry refait `pool.begin()`. La fermeture ne prête
    // que des références (`&state.pool`, `&current_user`) : elle est `Fn` et
    // non `FnOnce`, sans clone par tentative (revue P1 de la 15-5e2, B-5).
    // Story 35-5a0 — la date du jour, lue une fois sur l'horloge de l'état.
    let aujourd_hui = state.horloge.aujourd_hui();
    // Story 37-1 (AC-11) — les dates du premier exercice, résolues et validées
    // une fois, hors de la fermeture rejouée. Le refus éventuel est rendu par
    // `finalize_inner` après la lecture de l'étape : une finalisation déjà faite
    // ignore le corps (37-1-CR-P1-3) ; à faire, le refus ne touche ni la base ni
    // l'étape.
    let req = body.map(|Json(r)| r).unwrap_or_default();
    let exercice = premier_exercice(&req, aujourd_hui).map_err(RefusPremierExercice::from);
    crate::retry::retry_app_on_deadlock("onboarding::finalize", || {
        finalize_inner(&state.pool, &current_user, &exercice)
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
/// proprement (return Ok early), (b) les réglages ne s'écrivent que sur une
/// colonne vide (`COALESCE`, et aucun `UPDATE` s'il n'y a rien à poser — Story
/// 36-19), et l'`INSERT IGNORE` des vat_rates est idempotent, (c)
/// `create_if_absent_in_tx` sur fiscal_year
/// est idempotent, (d) l'`UPDATE` final ne se déclenche que si step était
/// 7 → bumpe à 8 ou laisse no-op si quelqu'un d'autre a déjà bumpé.
///
/// **Audit (Story 15-7a2)** : les entrées s'écrivent DANS cette fonction, donc
/// dans la tentative rejouée : l'issue des réglages et la liste des taux
/// insérés sont celles de la tentative qui commite — l'audit d'un essai annulé
/// disparaît avec lui. Ordre : `company_invoice_settings.created` (ligne neuve)
/// ou `company_invoice_settings.updated` (ligne complétée, Story 36-19), puis
/// `vat_rate.created` (ordre du seed), `fiscal_year.created`, l'étape.
///
/// `exercice` (Story 37-1, AC-11, après la 35-5a0) : le nom et les dates du
/// premier exercice, résolus par l'appelant depuis le corps et la date du jour
/// de l'horloge de l'état — identiques d'une tentative à l'autre —, ou leur
/// refus, rendu seulement si la finalisation reste à faire (37-1-CR-P1-3).
async fn finalize_inner(
    pool: &sqlx::MySqlPool,
    current_user: &CurrentUser,
    exercice: &Result<PremierExercice, RefusPremierExercice>,
) -> Result<OnboardingResponse, AppError> {
    // F2/F3/F4 CRITICAL FIX: Pessimistic locking strategy.
    // 1. Lock onboarding_state (serializes all finalize() calls on same session)
    // 2. Check state is still at step 7 or 8 (prevents TOCTOU on onboarding progression)
    // 3. Lock company row (prevents deletion during finalize)
    // 4. Proceed with complete_default_accounts_in_tx() (Story 36-19) with guaranteed exclusive access
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

    // Story 37-1 (AC-11, 37-1-CR-P1-3) — la finalisation reste à faire : les
    // dates du corps s'appliquent, et leur refus se rend ici, avant toute
    // écriture.
    let exercice = match exercice {
        Ok(e) => e,
        Err(refus) => {
            best_effort_rollback(tx).await;
            return Err(AppError::from(refus));
        }
    };

    // F3 CRITICAL FIX: Lock company row before complete_default_accounts_in_tx()
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

    // Story 36-19 (#1200, refs #987) — les réglages de facturation : créés, ou
    // **complétés** si un écran les a créés vides avant la fin de l'assistant
    // (`GET /api/v1/company/invoice-settings`), sans remplacer un compte déjà
    // choisi. Ordre des verrous dans cette transaction (revue P1 de
    // l'intégration de la v0.18.0, L2) : `companies` en partage (Story 37-1 ;
    // sans attente, la ligne est déjà tenue `FOR UPDATE` par `lock_company`) →
    // `accounts` (candidats `FOR UPDATE`) → `company_invoice_settings`
    // (`INSERT IGNORE`, puis la ligne `FOR UPDATE`) → `accounts` (désignés
    // `LOCK IN SHARE MODE` ; candidats relus, déjà tenus, et leur devise ; pour le
    // seul candidat au compte de produit par défaut, les entrées d'index
    // `idx_accounts_fx_gain` / `idx_accounts_fx_loss` en partage,
    // `accounts::fx_designations_of_in_tx`) → `bank_accounts`
    // (`LOCK IN SHARE MODE`). Un refus nomme chaque réglage obligatoire en défaut, sa
    // cause et son remède (`400 VALIDATION_ERROR`, code que la Story 36-17
    // reconnaît).
    let settings_outcome =
        match kesh_db::repositories::company_invoice_settings::complete_default_accounts_in_tx(
            &mut tx, company.id,
        )
        .await
        {
            Ok(outcome) => outcome,
            Err(kesh_db::errors::DbError::DefaultInvoiceAccountsUnavailable(items)) => {
                best_effort_rollback(tx).await;
                return Err(AppError::Validation(finalize_refusal_message(&items)));
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

    let seeded =
        audit_finalize_seed(&mut tx, current_user, &settings_outcome, &inserted_rates).await;
    if let Err(e) = seeded {
        best_effort_rollback(tx).await;
        return Err(e);
    }

    // Story 3.7 AC #13 — auto-create the first fiscal_year if none exists
    // (Path B), aux dates choisies dans l'assistant (Story 37-1, AC-11 ; par
    // défaut l'année civile du jour). L'INSERT atomique anti-TOCTOU
    // (`create_if_absent_in_tx`) garantit l'idempotence sous finalize concurrent.
    // Audit log inséré uniquement si la création a effectivement eu lieu — par
    // le dépôt lui-même : `finalize` n'écrit PAS sa propre entrée d'exercice
    // (Story 15-7a2, AC 7 ; son attribution par `::user` est la dette #431).
    let new_fy = NewFiscalYear {
        company_id: company.id,
        name: exercice.nom.clone(),
        start_date: exercice.debut,
        end_date: exercice.fin,
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

/// Le motif du refus de la finalisation (Story 36-19, #1200, D5, C-36-19-4) :
/// « La configuration ne peut pas se terminer. » puis une phrase par réglage
/// obligatoire en défaut — dans l'ordre reçu, débiteurs d'abord —, séparées par
/// une espace, chacune avec sa cause et son remède, puis « Réessayez ensuite. ».
///
/// En français seulement, comme les autres refus de cette route (limite #710,
/// écrite au manuel) ; rendu `400 VALIDATION_ERROR` par `AppError::Validation`.
fn finalize_refusal_message(items: &[UnavailableDefaultInvoiceAccount]) -> String {
    let phrases: Vec<String> = items.iter().map(finalize_refusal_phrase).collect();
    format!(
        "La configuration ne peut pas se terminer. {} Réessayez ensuite.",
        phrases.join(" ")
    )
}

/// Une phrase du motif de [`finalize_refusal_message`] : le réglage, la cause,
/// le remède. Libellés des rôles : `account-role-receivable` et
/// `account-role-default-revenue` du catalogue `fr-CH`.
fn finalize_refusal_phrase(item: &UnavailableDefaultInvoiceAccount) -> String {
    let (setting, role) = match item.setting {
        DefaultInvoiceSetting::Receivable => ("Compte débiteurs", "Créances clients"),
        DefaultInvoiceSetting::Revenue => ("Compte de produit par défaut", "Produit par défaut"),
    };
    match &item.cause {
        DefaultInvoiceAccountCause::NoRoleAccount => format!(
            "{setting} : le réglage est vide et aucun compte actif du plan comptable ne porte le \
             rôle « {role} » — désignez-le dans Administration → Facturation, ou attribuez ce rôle \
             à un compte dans le plan comptable."
        ),
        DefaultInvoiceAccountCause::DesignatedArchived { account_number } => format!(
            "{setting} : le compte désigné ({account_number}) est archivé — désignez-en un autre \
             dans Administration → Facturation, ou réactivez-le dans le plan comptable."
        ),
        // Revue P1, B6/E3 : un compte qui n'est pas de la société n'est pas dit
        // « archivé » (le remède « réactivez-le » serait impossible).
        DefaultInvoiceAccountCause::DesignatedOutsideCompany => format!(
            "{setting} : le compte désigné n'est pas un compte de cette société — désignez-en un \
             autre dans Administration → Facturation."
        ),
        DefaultInvoiceAccountCause::RoleAccountNotPostable { account_number } => format!(
            "{setting} : le compte {account_number}, qui porte le rôle « {role} », n'est pas \
             imputable — désignez un compte imputable dans Administration → Facturation, ou, dans \
             le plan comptable, retirez d'abord ce rôle au compte {account_number}, puis \
             attribuez-le à l'un de ses sous-comptes."
        ),
        // Revue P1, E4 : la garde retient aussi les comptes bancaires clos
        // (`first_active_bank_account_linked_to`, Story 28-4a), dont le lien ne
        // se change pas (`409 BANK_ACCOUNT_CLOSED`) tant qu'ils ne sont pas rouverts.
        DefaultInvoiceAccountCause::RoleAccountLinkedToBankAccount {
            account_number,
            bank_label,
        } => format!(
            "{setting} : le compte {account_number}, qui porte le rôle « {role} », est le compte \
             du compte bancaire « {bank_label} » — déliez ce compte bancaire dans Administration → \
             Comptes bancaires (s'il est clos, rouvrez-le d'abord), ou désignez un autre {} dans \
             Administration → Facturation.",
            setting.to_lowercase()
        ),
        // Story 37-1 (AC-4), intégration de la v0.18.0 dans l'Epic 37 : la
        // finalisation ne désigne pas un compte en devise, ni, au compte de produit
        // par défaut, un compte de gains ou de pertes de change — la règle du `PUT`
        // des réglages (`accounts::designation_refusal_in_tx`, revue P1 de
        // l'intégration, M1).
        DefaultInvoiceAccountCause::RoleAccountInForeignCurrency {
            account_number,
            currency,
        } => format!(
            "{setting} : le compte {account_number}, qui porte le rôle « {role} », est tenu en \
             {currency} — désignez un compte en francs suisses dans Administration → Facturation."
        ),
        // Revue P1 de l'intégration, L1 : une désignation propre nomme le compte en
        // devise qui la porte — l'utilisateur ne la trouverait pas dans le réglage
        // multi-devises de la société.
        DefaultInvoiceAccountCause::RoleAccountIsFxAccount {
            account_number,
            designation,
            holder_number,
        } => {
            let kind = match designation {
                kesh_db::errors::FxDesignation::Gain => "gains de change",
                kesh_db::errors::FxDesignation::Loss => "pertes de change",
                // Non atteint (M1) : le compte de conversion n'est pas écarté.
                kesh_db::errors::FxDesignation::Transit => "conversion",
            };
            let what = match holder_number {
                Some(holder) => format!(
                    "le compte de {kind} propre au compte en devise {holder} (plan comptable)"
                ),
                None => format!(
                    "le compte de {kind} de la société (Administration → Paramètres, section \
                     Comptabilité)"
                ),
            };
            format!(
                "{setting} : le compte {account_number}, qui porte le rôle « {role} », est {what} \
                 — un gain ou une perte de change ne va pas au chiffre d'affaires ; désignez un \
                 autre {} dans Administration → Facturation.",
                setting.to_lowercase()
            )
        }
    }
}

/// Trace ce que `finalize` a **réellement écrit** (Story 15-7a2, AC 5) :
/// `company_invoice_settings.created` si la ligne de réglages est neuve, ou
/// `company_invoice_settings.updated` (`trigger` = `onboarding`, `before`,
/// `after` au format de `settings_snapshot_json`) si une ligne existante a été
/// complétée (Story 36-19, AC5), rien si elle est restée inchangée ; puis un
/// `vat_rate.created` par taux inséré, dans l'ordre du seed.
async fn audit_finalize_seed(
    tx: &mut Tx<'_>,
    user: &CurrentUser,
    settings: &DefaultAccountsOutcome,
    inserted_rates: &[VatRate],
) -> Result<(), AppError> {
    if let DefaultAccountsOutcome::Completed { before, after } = settings {
        // Même action et même forme que le `PUT` des réglages (l'écran du journal
        // d'audit la connaît), attribuée par `from_current_user` comme `…created`.
        audit_log::insert_in_tx(
            tx,
            NewAuditLogEntry::from_current_user(
                user,
                "company_invoice_settings.updated",
                "company_invoice_settings",
                after.company_id,
                Some(json!({
                    "trigger": "onboarding",
                    "before": settings_snapshot_json(before),
                    "after": settings_snapshot_json(after),
                })),
            ),
        )
        .await?;
    }
    if let DefaultAccountsOutcome::Inserted(s) = settings {
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
             instance_language, email, phone, website, is_stub, books_locked_through, \
             multi_currency_enabled, fx_gain_account_id, fx_loss_account_id, fx_transit_account_id, \
             version, created_at, updated_at \
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
/// une company). Aucune société ⇒ insertion d'une société provisoire,
/// rattachement des principaux orphelins (Story 15-7b2) et `company.created` ;
/// sinon `company.updated {instance_language}` si la langue
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
            // Base SANS société : une installation déjà atteinte par #528 (une
            // remise à zéro antérieure à la 0.13.0 effaçait `companies`), ou
            // une base dont le premier démarrage n'a pas posé le stub. Société
            // provisoire insérée (`companies::insert_stub`, mêmes valeurs que
            // le bootstrap et la remise à zéro), puis **rattachement** des
            // principaux orphelins (Story 15-7b2, AC 4) : les utilisateurs
            // rejoignent la société créée, les clés d'API et (Story 32-5d) MCP
            // actives orphelines sont révoquées puis rattachées — jamais
            // réveillées.
            // `set_coordinates` lèvera le drapeau (`clear_stub_in_tx`).
            let id = companies::insert_stub(&mut **tx, lang).await?;
            let principals = companies::reattach_orphan_principals_in_tx(tx, Some(id)).await?;
            audit_log::insert_in_tx(
                tx,
                NewAuditLogEntry::from_current_user(
                    user,
                    "company.created",
                    "company",
                    id,
                    Some(json!({
                        "instance_language": lang,
                        "is_stub": true,
                        "users_repointed": principals.user_ids.len(),
                        "api_keys_revoked": principals.api_keys_revoked,
                        "api_keys_repointed": principals.api_keys_repointed,
                        "mcp_keys_revoked": principals.mcp_keys_revoked,
                        "mcp_keys_repointed": principals.mcp_keys_repointed,
                    })),
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

#[cfg(test)]
mod tests {
    use super::{BankAccountRequest, FinalizeRequest, PremierExercice, premier_exercice};
    use crate::errors::AppError;
    use chrono::NaiveDate;

    fn jour(a: i32, m: u32, j: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(a, m, j).unwrap()
    }

    fn corps(debut: Option<NaiveDate>, fin: Option<NaiveDate>) -> FinalizeRequest {
        FinalizeRequest {
            fiscal_year_start_date: debut,
            fiscal_year_end_date: fin,
        }
    }

    /// Story 37-1 (AC-11) — sans corps : l'année civile du jour, comme avant.
    #[test]
    fn premier_exercice_sans_dates_est_l_annee_civile_du_jour() {
        assert_eq!(
            premier_exercice(&FinalizeRequest::default(), jour(2026, 11, 16)).unwrap(),
            PremierExercice {
                nom: "Exercice 2026".into(),
                debut: jour(2026, 1, 1),
                fin: jour(2026, 12, 31),
            }
        );
    }

    /// Story 37-1 (AC-11) — un début seul : fin à la veille du même jour un an
    /// plus tard, nom à deux années ; cas limites du 29 février.
    #[test]
    fn premier_exercice_debut_seul_calcule_la_fin() {
        let cas = [
            (jour(2026, 4, 1), jour(2027, 3, 31), "Exercice 2026/2027"),
            (jour(2028, 2, 29), jour(2029, 2, 28), "Exercice 2028/2029"),
            (jour(2027, 3, 1), jour(2028, 2, 29), "Exercice 2027/2028"),
            (jour(2027, 1, 1), jour(2027, 12, 31), "Exercice 2027"),
        ];
        for (debut, fin, nom) in cas {
            let e = premier_exercice(&corps(Some(debut), None), jour(2026, 11, 16)).unwrap();
            assert_eq!(
                (e.debut, e.fin, e.nom.as_str()),
                (debut, fin, nom),
                "{debut}"
            );
        }
    }

    /// Story 37-1 (AC-11) — fin fournie : aucune borne de durée ; une fin
    /// seule garde le 1er janvier de l'année du jour.
    #[test]
    fn premier_exercice_fin_fournie_sans_borne_de_duree() {
        let e = premier_exercice(
            &corps(Some(jour(2026, 7, 1)), Some(jour(2027, 12, 31))),
            jour(2026, 11, 16),
        )
        .unwrap();
        assert_eq!((e.debut, e.fin), (jour(2026, 7, 1), jour(2027, 12, 31)));
        let e =
            premier_exercice(&corps(None, Some(jour(2026, 6, 30))), jour(2026, 11, 16)).unwrap();
        assert_eq!(
            (e.debut, e.fin, e.nom.as_str()),
            (jour(2026, 1, 1), jour(2026, 6, 30), "Exercice 2026")
        );
    }

    /// Story 37-1 (AC-11) — refus : fin ≤ début, dates hors de la plage de la
    /// base, fin calculée hors plage (début en 9999).
    #[test]
    fn premier_exercice_refuse_les_dates_invalides() {
        let aujourd_hui = jour(2026, 11, 16);
        for c in [
            corps(Some(jour(2026, 4, 1)), Some(jour(2026, 4, 1))),
            corps(Some(jour(2026, 4, 1)), Some(jour(2026, 3, 31))),
            corps(Some(jour(999, 1, 1)), None),
            corps(Some(jour(2026, 1, 1)), Some(jour(10000, 1, 1))),
            corps(Some(jour(9999, 2, 1)), None),
        ] {
            assert!(
                matches!(
                    premier_exercice(&c, aujourd_hui),
                    Err(AppError::Validation(_))
                ),
                "{c:?}"
            );
        }
    }

    /// Story 36-19 (revue P1, B5/E6 ; mutation M16) — le motif du refus de la
    /// finalisation, **mot pour mot**, pour chacune des cinq causes et pour un
    /// refus à deux causes **différentes** (débiteurs désigné archivé, produit
    /// vide sans rôle), débiteurs d'abord. Les continuations `\` de Rust sont
    /// ainsi vérifiées à l'exécution, et non seulement à l'œil.
    #[test]
    fn finalize_refusal_message_names_each_cause_word_for_word() {
        use super::finalize_refusal_message;
        use kesh_db::errors::DefaultInvoiceAccountCause as Cause;
        use kesh_db::errors::DefaultInvoiceSetting::{Receivable, Revenue};
        use kesh_db::errors::UnavailableDefaultInvoiceAccount as Item;

        let message = |items: Vec<(kesh_db::errors::DefaultInvoiceSetting, Cause)>| {
            finalize_refusal_message(
                &items
                    .into_iter()
                    .map(|(setting, cause)| Item { setting, cause })
                    .collect::<Vec<_>>(),
            )
        };
        let entoure =
            |p: &str| format!("La configuration ne peut pas se terminer. {p} Réessayez ensuite.");

        assert_eq!(
            message(vec![(
                Revenue,
                Cause::DesignatedArchived {
                    account_number: "3000".into()
                }
            )]),
            entoure(
                "Compte de produit par défaut : le compte désigné (3000) est archivé — désignez-en un autre dans Administration → Facturation, ou réactivez-le dans le plan comptable."
            )
        );
        assert_eq!(
            message(vec![(Receivable, Cause::DesignatedOutsideCompany)]),
            entoure(
                "Compte débiteurs : le compte désigné n'est pas un compte de cette société — désignez-en un autre dans Administration → Facturation."
            )
        );
        assert_eq!(
            message(vec![(
                Receivable,
                Cause::RoleAccountNotPostable {
                    account_number: "1100".into()
                }
            )]),
            entoure(
                "Compte débiteurs : le compte 1100, qui porte le rôle « Créances clients », n'est pas imputable — désignez un compte imputable dans Administration → Facturation, ou, dans le plan comptable, retirez d'abord ce rôle au compte 1100, puis attribuez-le à l'un de ses sous-comptes."
            )
        );
        assert_eq!(
            message(vec![(
                Revenue,
                Cause::RoleAccountNotPostable {
                    account_number: "3000".into()
                }
            )]),
            entoure(
                "Compte de produit par défaut : le compte 3000, qui porte le rôle « Produit par défaut », n'est pas imputable — désignez un compte imputable dans Administration → Facturation, ou, dans le plan comptable, retirez d'abord ce rôle au compte 3000, puis attribuez-le à l'un de ses sous-comptes."
            )
        );
        assert_eq!(
            message(vec![(
                Receivable,
                Cause::RoleAccountLinkedToBankAccount {
                    account_number: "1100".into(),
                    bank_label: "UBS — Courant".into(),
                }
            )]),
            entoure(
                "Compte débiteurs : le compte 1100, qui porte le rôle « Créances clients », est le compte du compte bancaire « UBS — Courant » — déliez ce compte bancaire dans Administration → Comptes bancaires (s'il est clos, rouvrez-le d'abord), ou désignez un autre compte débiteurs dans Administration → Facturation."
            )
        );
        // Intégration de la v0.18.0 dans l'Epic 37 (Story 37-1, AC-4) : compte de
        // rôle en devise ; compte de gains ou de pertes de change, de la société ou
        // propre (revue P1 de l'intégration, M1 et L1).
        assert_eq!(
            message(vec![(
                Receivable,
                Cause::RoleAccountInForeignCurrency {
                    account_number: "1100".into(),
                    currency: "EUR".into(),
                }
            )]),
            entoure(
                "Compte débiteurs : le compte 1100, qui porte le rôle « Créances clients », est tenu en EUR — désignez un compte en francs suisses dans Administration → Facturation."
            )
        );
        for (designation, holder, what) in [
            (
                kesh_db::errors::FxDesignation::Gain,
                None,
                "le compte de gains de change de la société (Administration → Paramètres, section Comptabilité)",
            ),
            (
                kesh_db::errors::FxDesignation::Loss,
                None,
                "le compte de pertes de change de la société (Administration → Paramètres, section Comptabilité)",
            ),
            (
                kesh_db::errors::FxDesignation::Gain,
                Some("1020"),
                "le compte de gains de change propre au compte en devise 1020 (plan comptable)",
            ),
            (
                kesh_db::errors::FxDesignation::Loss,
                Some("1021"),
                "le compte de pertes de change propre au compte en devise 1021 (plan comptable)",
            ),
        ] {
            assert_eq!(
                message(vec![(
                    Revenue,
                    Cause::RoleAccountIsFxAccount {
                        account_number: "3000".into(),
                        designation,
                        holder_number: holder.map(String::from),
                    }
                )]),
                entoure(&format!(
                    "Compte de produit par défaut : le compte 3000, qui porte le rôle « Produit par défaut », est {what} — un gain ou une perte de change ne va pas au chiffre d'affaires ; désignez un autre compte de produit par défaut dans Administration → Facturation."
                ))
            );
        }
        // Deux causes différentes, débiteurs d'abord, une espace entre elles.
        assert_eq!(
            message(vec![
                (
                    Receivable,
                    Cause::DesignatedArchived {
                        account_number: "1100".into()
                    }
                ),
                (Revenue, Cause::NoRoleAccount),
            ]),
            entoure(
                "Compte débiteurs : le compte désigné (1100) est archivé — désignez-en un autre dans Administration → Facturation, ou réactivez-le dans le plan comptable. Compte de produit par défaut : le réglage est vide et aucun compte actif du plan comptable ne porte le rôle « Produit par défaut » — désignez-le dans Administration → Facturation, ou attribuez ce rôle à un compte dans le plan comptable."
            )
        );
    }

    /// Story 34-1 (#966, AC-1, C-e341-34) — la double option du corps de
    /// l'étape bancaire : **absent** → `None` (inchangé), **`null`** →
    /// `Some(None)` (effacé), valeur → `Some(Some(_))`. Sans
    /// `double_option`, serde replierait `null` sur l'absence.
    #[test]
    fn bank_account_request_distinguishes_absent_null_and_value() {
        let absent: BankAccountRequest =
            serde_json::from_str(r#"{"bankName":"UBS","iban":"X","qrIban":null}"#).unwrap();
        assert_eq!(absent.description, None);
        assert_eq!(absent.journal_account_id, None);

        let nuls: BankAccountRequest = serde_json::from_str(
            r#"{"bankName":"UBS","iban":"X","description":null,"journalAccountId":null}"#,
        )
        .unwrap();
        assert_eq!(nuls.description, Some(None));
        assert_eq!(nuls.journal_account_id, Some(None));

        let valeurs: BankAccountRequest = serde_json::from_str(
            r#"{"bankName":"UBS","iban":"X","description":"Personnel","journalAccountId":24}"#,
        )
        .unwrap();
        assert_eq!(valeurs.description, Some(Some("Personnel".to_string())));
        assert_eq!(valeurs.journal_account_id, Some(Some(24)));
        assert_eq!(valeurs.qr_iban, None);
    }
}
