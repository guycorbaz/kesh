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
//! - `with-company-no-fy` : `with-company` mais sans `fiscal_year`
//!   (Issue #90 — Story 9-1 AC #34 garde-fou E2E « bouton Générer disabled »)
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
use axum::routing::{Router, get, post};
use kesh_db::test_fixtures::{
    mark_onboarding_complete, seed_accounting_company, seed_accounting_company_no_fy,
    seed_changeme_user_only, seed_contact_and_product, seed_stub_company_only, truncate_all,
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
        // Story 17-4e (DE-1) — injection d'un token de reset pour les E2E
        // recovery (l'email réel n'est pas vérifiable sans SMTP).
        .route("/password-reset-token", post(password_reset_token_handler))
        // Story 20-4 — capture des e-mails métier (round-trip Playwright).
        .route("/sent-emails", get(sent_emails_handler))
        // Story 37-2 (AC-3) — pilotage de la source factice des cours. Sur une
        // ligne : le registre des routes lit `.route("…", post(…))` ligne à ligne.
        .route("/exchange-rates/source", post(rate_source_handler))
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
    /// Issue #90 / Story 9-1 AC #34 : variante de `WithCompany` qui **omet
    /// le `fiscal_year`**. Garde-fou E2E pour la page `/reports` avec
    /// company sans exercice comptable (bouton « Générer » disabled +
    /// message i18n `reports-error-no-fiscal-year-available`).
    WithCompanyNoFy,
    /// **Story v011-5 AC #24** : DB vidée + 1 company stub seule (aucun user).
    /// Le frontend `/me` retournera 423 Locked → redirect `/setup` → submit
    /// → l'admin est créé via `POST /setup/admin` → flow onboarding self-service.
    SetupRequired,
}

#[derive(Debug, Deserialize)]
pub struct SeedRequest {
    pub preset: Preset,
}

const VALID_PRESETS: &str =
    "fresh, post-onboarding, with-company, with-data, with-company-no-fy, setup-required";

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

/// Installe les prompts standard du serveur MCP dans chaque société (Story
/// 32-10a-i, AC-i11) : **tout** préréglage crée une société (`fresh` et
/// `setup-required` une société provisoire, les autres une société complète),
/// et l'amorçage suit la création. Un échec rend `500`, comme les autres
/// étapes du préréglage.
async fn amorcer_prompts_standard(pool: &sqlx::MySqlPool) -> Result<(), AppError> {
    kesh_db::repositories::mcp_prompts::amorcer_standards(
        pool,
        kesh_db::entities::mcp_prompt_standards::STANDARDS,
    )
    .await
    .map(|_| ())
    .map_err(|e| AppError::Internal(format!("amorcer_standards: {e}")))
}

/// `POST /api/v1/_test/seed` — truncate puis re-seed selon le preset, puis
/// amorçage des prompts standard ([`amorcer_prompts_standard`], après le
/// préréglage : la société naît avec lui).
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
    clear_exchange_rate_tables(&state).await?;

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
        Preset::WithCompanyNoFy => {
            seed_accounting_company_no_fy(&state.pool)
                .await
                .map_err(|e| AppError::Internal(format!("seed_accounting_company_no_fy: {e}")))?;
            mark_onboarding_complete(&state.pool)
                .await
                .map_err(|e| AppError::Internal(format!("mark_onboarding_complete: {e}")))?;
            "with-company-no-fy"
        }
        // Story v011-5 AC #24 — DB vidée + 1 company stub seule (aucun user).
        // Le test setup_admin_e2e + Playwright setup.spec.ts utilisent ce preset
        // pour valider le flow `/setup` complet (423 → submit → cookies + redirect).
        Preset::SetupRequired => {
            seed_stub_company_only(&state.pool)
                .await
                .map_err(|e| AppError::Internal(format!("seed_stub_company_only: {e}")))?;
            "setup-required"
        }
    };
    amorcer_prompts_standard(&state.pool).await?;

    // Story v011-5 (BH2-3) — synchroniser le flag mémoire `users_exist` après
    // chaque seed pour éviter une divergence cache mémoire vs DB. Après
    // `truncate_all + seed`, `users_exist` doit refléter la nouvelle réalité :
    // `false` pour `setup-required` (preset sans user), `true` pour tous les
    // autres presets qui ré-INSERT un user `changeme` ou les users d'une company.
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("seed user_count refresh: {e}")))?;
    state
        .users_exist
        .store(user_count > 0, std::sync::atomic::Ordering::Release);

    // Story 17-4e (DE-2) — repartir avec des rate-limiters vierges à chaque
    // seed : le budget recovery (5 req / 15 min / IP, partagé forgot+reset)
    // rendrait sinon les re-runs E2E locaux flaky (état mémoire, pas DB).
    clear_transient_test_state(&state);

    Ok(Json(SeedResponse {
        preset: preset_label,
        ok: true,
    }))
}

/// Corps de `POST /api/v1/_test/password-reset-token` (Story 17-4e, DE-1).
#[derive(Debug, Deserialize)]
pub struct PasswordResetTokenRequest {
    pub username: String,
}

/// Réponse : le token de reset EN CLAIR (endpoint de test uniquement —
/// jamais monté hors `test_mode`, cf. doc module).
#[derive(Debug, Serialize)]
pub struct PasswordResetTokenResponse {
    pub token: String,
}

/// `POST /api/v1/_test/password-reset-token` — Story 17-4e (DE-1).
///
/// Crée un token de reset valide (TTL standard 30 min) pour `username` et
/// retourne le **clair**, pour que les specs Playwright recovery testent
/// `/reset-password?token=…` sans dépendre d'un vrai envoi SMTP (AC24
/// umbrella : « le token est injecté via seed/API de test »).
///
/// Pas d'anti-énumération ici : c'est un endpoint de test — un username
/// inconnu retourne un `404` franc pour diagnostiquer la spec.
async fn password_reset_token_handler(
    State(state): State<AppState>,
    Json(req): Json<PasswordResetTokenRequest>,
) -> Result<Json<PasswordResetTokenResponse>, AppError> {
    let user = kesh_db::repositories::users::find_by_username(&state.pool, &req.username)
        .await
        .map_err(AppError::Database)?
        // 404 franc via le mapping standard DbError::NotFound (endpoint de test,
        // pas d'anti-énumération).
        .ok_or(AppError::Database(kesh_db::errors::DbError::NotFound))?;

    let (token_clear, token_hash) = crate::auth::api_key::generate_reset_token();
    let expires_at = (chrono::Utc::now()
        + chrono::TimeDelta::minutes(crate::mail::PASSWORD_RESET_TTL_MINUTES))
    .naive_utc();
    kesh_db::repositories::password_reset_tokens::create(
        &state.pool,
        user.id,
        &token_hash,
        expires_at,
    )
    .await
    .map_err(AppError::Database)?;

    Ok(Json(PasswordResetTokenResponse { token: token_clear }))
}

/// `POST /api/v1/_test/reset` — alias de `seed { preset: "fresh" }`, amorçage
/// des prompts standard compris ([`amorcer_prompts_standard`]).
/// Également sérialisé par `seed_lock()` (code review P2).
async fn reset_handler(State(state): State<AppState>) -> Result<Json<SeedResponse>, AppError> {
    let _guard = seed_lock().lock().await;

    truncate_all(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("test_fixtures truncate_all: {e}")))?;
    clear_exchange_rate_tables(&state).await?;
    seed_changeme_user_only(&state.pool)
        .await
        .map_err(|e| AppError::Internal(format!("seed_changeme_user_only: {e}")))?;
    amorcer_prompts_standard(&state.pool).await?;
    // Story v011-5 (BH2-3) — sync users_exist post-reset (preset fresh inclut un user).
    state
        .users_exist
        .store(true, std::sync::atomic::Ordering::Release);
    // Story 17-4e Pass 1 (ECH) — même purge des limiters que seed_handler :
    // /_test/reset est un alias de seed{fresh}, il doit avoir la même
    // sémantique anti-flaky (DE-2).
    clear_transient_test_state(&state);
    Ok(Json(SeedResponse {
        preset: "fresh",
        ok: true,
    }))
}

/// Purge l'état transitoire en mémoire entre deux seeds E2E (Story 17-4e
/// rate-limiters, étendu Story 20-4) : budgets login/recovery/send-email et
/// buffer de capture d'e-mails. Partagé `seed_handler`/`reset_handler`.
fn clear_transient_test_state(state: &AppState) {
    state.rate_limiter.clear_all();
    state.rate_limiter_recovery.clear_all();
    state.rate_limiter_send_email.clear_all();
    if let Some(mock) = &state.test_mock_mailer {
        mock.clear();
    }
    // Story 37-2 (AC-7) — mémoire des 15 minutes, déclenchement de connexion,
    // source factice remise à son défaut.
    state
        .rate_fetch_memo
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
    *state
        .rate_login_trigger
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = None;
    if let Some(source) = &state.test_rate_source {
        source.reset();
    }
}

/// Story 37-2 (AC-7) — vide les deux tables système des cours
/// (`_kesh_fetched_exchange_rates`, `_kesh_exchange_rate_fetch`), qui
/// échappent à `truncate_all` : une valeur publiée posée par une spec ne
/// survit pas à la suivante. Partagé `seed_handler`/`reset_handler`.
async fn clear_exchange_rate_tables(state: &AppState) -> Result<(), AppError> {
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|e| AppError::Internal(format!("vidage des cours : {e}")))?;
    kesh_db::repositories::exchange_rates::clear_fetched_state(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("vidage des cours : {e}")))
}

/// Corps de `POST /api/v1/_test/exchange-rates/source` (Story 37-2, AC-3).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExchangeRatesSourceRequest {
    /// `month` ou `day`.
    pub kind: kesh_core::exchange_rates::RateKind,
    /// `default` (XML synthétique généré pour l'unité demandée), `xml` (le
    /// champ `xml`, servi tel quel), `empty` (racine vide : pas encore
    /// publié) ou `error` (le code classé du champ `error`).
    pub reply: String,
    /// XML servi tel quel (`reply: "xml"`).
    pub xml: Option<String>,
    /// Code classé (`reply: "error"`) : `timeout`, `connect`,
    /// `http_status:<n>`, `too_large`, `malformed`, `not_published`.
    pub error: Option<String>,
    /// Attente avant la réponse, en millisecondes (au plus 120 000).
    pub delay_ms: Option<u64>,
    /// Ne piloter que cette unité (`AAAA-MM-JJ`, 1er du mois pour `month`) ;
    /// absent : toutes les unités du genre. Lue en chaîne puis analysée (une
    /// date de pilotage n'est pas une date comptable reçue : hors de la garde
    /// `dates_recues_garde`).
    pub date: Option<String>,
}

/// Réponse de `POST /api/v1/_test/exchange-rates/source`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeRatesSourceResponse {
    /// Nombre d'appels reçus par la source factice depuis sa remise à zéro.
    pub calls: usize,
}

/// `POST /api/v1/_test/exchange-rates/source` — Story 37-2 (AC-3) : pilote la
/// réponse de la source factice par genre (ou par unité). 400 hors du mode
/// capture (source réelle) ou pour une réponse mal décrite. Rend le compteur
/// d'appels. Une remise à zéro (`_test/seed`, `_test/reset`) ramène la source
/// à son défaut.
async fn rate_source_handler(
    State(state): State<AppState>,
    Json(req): Json<ExchangeRatesSourceRequest>,
) -> Result<Json<ExchangeRatesSourceResponse>, AppError> {
    use crate::exchange_rates::{MockBehavior, MockReply, SourceError};
    use kesh_core::exchange_rates::RateUnit;

    let Some(source) = &state.test_rate_source else {
        return Err(AppError::Validation(
            "source des cours factice indisponible (backend hors KESH_TEST_MODE)".to_string(),
        ));
    };
    let reply = match (req.reply.as_str(), req.xml, req.error.as_deref()) {
        ("default", None, None) => MockReply::Default,
        ("empty", None, None) => MockReply::Empty,
        ("xml", Some(x), None) => MockReply::Xml(x),
        ("error", None, Some(code)) => MockReply::Error(
            SourceError::from_code(code)
                .ok_or_else(|| AppError::Validation(format!("code d'erreur inconnu : {code}")))?,
        ),
        _ => {
            return Err(AppError::Validation(
                "réponse mal décrite : `reply` default | empty | xml (+ `xml`) | error (+ `error`)"
                    .to_string(),
            ));
        }
    };
    let delay = match req.delay_ms {
        Some(ms) if ms > 120_000 => {
            return Err(AppError::Validation("`delayMs` au plus 120000".to_string()));
        }
        Some(ms) => Some(std::time::Duration::from_millis(ms)),
        None => None,
    };
    let behavior = MockBehavior { reply, delay };
    let date = req
        .date
        .as_deref()
        .map(|t| {
            chrono::NaiveDate::parse_from_str(t, "%Y-%m-%d")
                .map_err(|_| AppError::Validation(format!("`date` illisible : {t}")))
        })
        .transpose()?;
    match date {
        Some(date) => source.set_for(
            match req.kind {
                kesh_core::exchange_rates::RateKind::Month => RateUnit::month_of(date),
                kesh_core::exchange_rates::RateKind::Day => RateUnit::day(date),
            },
            behavior,
        ),
        None => source.set(req.kind, behavior),
    }
    Ok(Json(ExchangeRatesSourceResponse {
        calls: source.call_count(),
    }))
}

/// Un e-mail métier capturé, shape camelCase pour les specs Playwright
/// (Story 20-4 — mapping complet de `CapturedEmail`).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SentEmailJson {
    pub to: String,
    pub subject: String,
    pub body: String,
    pub from_display_name: Option<String>,
    pub reply_to: Option<String>,
    pub attachment_filename: Option<String>,
    pub attachment_content_type: Option<String>,
    pub attachment_size: usize,
}

/// Réponse de `GET /api/v1/_test/sent-emails`.
#[derive(Debug, Serialize)]
pub struct SentEmailsResponse {
    pub emails: Vec<SentEmailJson>,
}

/// `GET /api/v1/_test/sent-emails` — Story 20-4.
///
/// Renvoie les e-mails métier capturés par le `MockMailer` substitué au boot
/// (`KESH_TEST_MODE` + config SMTP factice, cf. `main.rs`). Si le boot n'est
/// PAS en mode capture (test-mode sans vars SMTP → NoopMailer), répond
/// 400 `VALIDATION_ERROR` avec un message explicite plutôt qu'un 200 vide
/// ambigu — la spec Playwright diagnostique immédiatement une recette de
/// lancement incomplète. (Déviation documentée vs spec [409] : réutilise
/// `AppError::Validation`, pas de variant dédié pour un endpoint de test.)
async fn sent_emails_handler(
    State(state): State<AppState>,
) -> Result<Json<SentEmailsResponse>, AppError> {
    let Some(mock) = &state.test_mock_mailer else {
        return Err(AppError::Validation(
            "test-mode sans SMTP factice — capture d'e-mails indisponible (démarrer le backend avec KESH_SMTP_* factices, cf. docs/testing.md)".to_string(),
        ));
    };
    let emails = mock
        .sent_emails()
        .into_iter()
        .map(|e| SentEmailJson {
            to: e.to,
            subject: e.subject,
            body: e.body,
            from_display_name: e.from_display_name,
            reply_to: e.reply_to,
            attachment_filename: e.attachment_filename,
            attachment_content_type: e.attachment_content_type,
            attachment_size: e.attachment_size,
        })
        .collect();
    Ok(Json(SentEmailsResponse { emails }))
}
