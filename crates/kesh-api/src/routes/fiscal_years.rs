//! Routes API pour les exercices comptables (Story 3.7).
//!
//! Endpoints :
//! - `GET    /api/v1/fiscal-years`        — liste scopée company (DESC).
//! - `GET    /api/v1/fiscal-years/{id}`   — détail scopé company.
//! - `GET    /api/v1/fiscal-years/{id}/closing-checks` — contrôles préalables à
//!   la clôture, lecture seule, tout rôle (Story 29-1a, #232).
//! - `POST   /api/v1/fiscal-years`        — création (Comptable+).
//! - `PUT    /api/v1/fiscal-years/{id}`   — renommage uniquement (Comptable+).
//! - `POST   /api/v1/fiscal-years/{id}/close` — transition Open→Closed (Comptable+).
//! - `POST   /api/v1/fiscal-years/{id}/reopen` — transition Closed→Open (Admin, motif, audit — Story 14-2).
//! - `PUT    /api/v1/fiscal-years/{id}/rate-kinds` — genres des cours d'ouverture et
//!   de clôture d'un exercice multi-devises (Comptable+, Story 37-2).
//!
//! Toutes les routes mutatrices forcent `company_id = current_user.company_id`
//! ; tout `companyId` injecté dans le payload est ignoré (multi-tenant
//! defense en profondeur — Story 6.2 + Pass 2 HP2-M9).
//!
//! Pas de `DELETE` : conformément au CO art. 957-964 (10 ans de conservation),
//! aucun handler `delete_fiscal_year` n'est exposé. Une requête
//! `DELETE /api/v1/fiscal-years/{id}` retournera donc `405 Method Not Allowed`
//! (axum) — voir AC #23.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

use kesh_db::entities::{FiscalYear, FiscalYearStatus, NewFiscalYear, RateKind};
use kesh_db::errors::DbError;
use kesh_db::repositories::closing_checks::{self, ClosingChecks};
use kesh_db::repositories::fiscal_years::{
    self, FY_NAME_DUPLICATE_KEY, FY_NAME_EMPTY_KEY, FY_NAME_TOO_LONG_KEY, FY_OVERLAP_KEY,
    FY_REOPEN_ALREADY_OPEN_KEY, FY_REOPEN_LIFO_BLOCKED_KEY, REOPEN_MOTIF_MAX,
};
use kesh_db::snapshot::ReadSnapshot;

use crate::AppState;
use crate::dates_recues::borner;
use crate::errors::{AppError, t};
use crate::middleware::auth::CurrentUser;
use crate::routes::api_keys::ensure_not_pat;
use crate::services::acces::ContexteAcces;
use crate::services::exercices;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FiscalYearResponse {
    pub id: i64,
    pub company_id: i64,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    /// `"Open"` ou `"Closed"` (PascalCase, cohérent avec l'enum DB).
    pub status: FiscalYearStatus,
    /// Exercice multi-devises (Story 37-1, AC-2). Toujours présent.
    pub multi_currency: bool,
    /// Genre du cours d'ouverture, `"day"` ou `"month"` (Story 37-1, AC-2 ;
    /// sens : doc de `FiscalYear`). Toujours présent.
    pub opening_rate_kind: RateKind,
    /// Genre du cours de clôture, `"day"` ou `"month"`. Toujours présent.
    pub closing_rate_kind: RateKind,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<FiscalYear> for FiscalYearResponse {
    fn from(fy: FiscalYear) -> Self {
        Self {
            id: fy.id,
            company_id: fy.company_id,
            name: fy.name,
            start_date: fy.start_date,
            end_date: fy.end_date,
            status: fy.status,
            multi_currency: fy.multi_currency,
            opening_rate_kind: fy.opening_rate_kind,
            closing_rate_kind: fy.closing_rate_kind,
            created_at: fy.created_at,
            updated_at: fy.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFiscalYearRequest {
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFiscalYearRequest {
    pub name: String,
}

/// Story 14-2 — body de `POST /api/v1/fiscal-years/{id}/reopen`.
///
/// Le `motif` est **obligatoire** (validé non vide et ≤ [`REOPEN_MOTIF_MAX`] au
/// handler, décision D4) et tracé dans l'audit `fiscal_year.reopened`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReopenFiscalYearRequest {
    pub motif: String,
}

// ---------------------------------------------------------------------------
// Mapping erreurs DB → AppError
// ---------------------------------------------------------------------------

/// Mapping des erreurs `create()` (Pass 2 HP2-M4 — namespaced Invariant keys).
fn map_create_error(err: DbError) -> AppError {
    match err {
        DbError::Invariant(ref s) if s == FY_OVERLAP_KEY => AppError::Validation(t(
            "error-fiscal-year-overlap",
            "Cet exercice chevauche un exercice existant",
        )),
        DbError::Invariant(ref s) if s == FY_NAME_DUPLICATE_KEY => AppError::Validation(t(
            "error-fiscal-year-name-duplicate",
            "Un exercice avec ce nom existe déjà",
        )),
        DbError::Invariant(ref s) if s == FY_NAME_EMPTY_KEY => AppError::Validation(t(
            "error-fiscal-year-name-empty",
            "Le nom de l'exercice est obligatoire",
        )),
        // Story 3.7 Code Review Pass 1 F3 — pré-validation longueur (VARCHAR(50)).
        DbError::Invariant(ref s) if s == FY_NAME_TOO_LONG_KEY => AppError::Validation(t(
            "error-fiscal-year-name-too-long",
            "Le nom de l'exercice est trop long (50 caractères maximum)",
        )),
        DbError::CheckConstraintViolation(_) => AppError::Validation(t(
            "error-fiscal-year-dates-invalid",
            "Les dates de l'exercice sont invalides (la date de fin doit être strictement postérieure à la date de début)",
        )),
        DbError::UniqueConstraintViolation(_) => AppError::Validation(t(
            "error-fiscal-year-conflict",
            "Conflit d'exercice (nom ou date de début déjà utilisé)",
        )),
        DbError::ForeignKeyViolation(m) => {
            // Pass 2 HP2-M6 : log + erreur interne — JWT garantit que company_id existe.
            tracing::error!(
                "FK violation in fiscal_years::create — JWT scope should prevent this: {m}"
            );
            AppError::Internal(format!(
                "FK violation impossible (JWT scope) — investigate: {m}"
            ))
        }
        // Story 15-12a (#543, C-15-12a-1) — un exercice postérieur est clos :
        // même code et mêmes `details` que le mapping global, message propre
        // à la création.
        DbError::LaterFiscalYearClosed {
            fiscal_year_id,
            fiscal_year_name,
        } => AppError::FiscalYearBeforeClosedYear {
            fiscal_year_id,
            fiscal_year_name,
        },
        // Story 37-1 (D-CR7-2) — un exercice postérieur est mono-devise, dans
        // une société multi-devises : code propre, mêmes `details`.
        DbError::LaterFiscalYearMonoCurrency {
            fiscal_year_id,
            fiscal_year_name,
        } => AppError::FiscalYearBeforeMonoCurrencyYear {
            fiscal_year_id,
            fiscal_year_name,
        },
        // Tout le reste (Sqlx, ConnectionUnavailable, NotFound, ...) → mapping standard.
        other => AppError::from(other),
    }
}

fn map_update_error(err: DbError) -> AppError {
    match err {
        DbError::Invariant(ref s) if s == FY_NAME_EMPTY_KEY => AppError::Validation(t(
            "error-fiscal-year-name-empty",
            "Le nom de l'exercice est obligatoire",
        )),
        DbError::Invariant(ref s) if s == FY_NAME_DUPLICATE_KEY => AppError::Validation(t(
            "error-fiscal-year-name-duplicate",
            "Un exercice avec ce nom existe déjà",
        )),
        // Story 3.7 Code Review Pass 1 F3 — pré-validation longueur (VARCHAR(50)).
        DbError::Invariant(ref s) if s == FY_NAME_TOO_LONG_KEY => AppError::Validation(t(
            "error-fiscal-year-name-too-long",
            "Le nom de l'exercice est trop long (50 caractères maximum)",
        )),
        other => AppError::from(other),
    }
}

/// Mapping des erreurs `reopen()` (Story 14-2, décision D7 — miroir
/// `map_create_error`).
///
/// Les deux conflits de réouverture partagent le **code machine** 409
/// `ILLEGAL_STATE_TRANSITION` mais portent des **messages utilisateur distincts
/// et localisés** (via `AppError::IllegalState(t(...))`). Ils sont émis par le
/// repo en `DbError::Invariant(<KEY>)` namespacés — jamais en
/// `DbError::IllegalStateTransition` (dont le `Display` log-only écraserait le
/// message). Toute autre `Invariant` (dont `FY_REOPEN_UNEXPECTED_KEY`) retombe
/// vers le mapping global (→ 500, défensif).
fn map_reopen_error(err: DbError) -> AppError {
    match err {
        DbError::Invariant(ref s) if s == FY_REOPEN_ALREADY_OPEN_KEY => AppError::IllegalState(t(
            "error-fiscal-year-already-open",
            "Cet exercice est déjà ouvert.",
        )),
        DbError::Invariant(ref s) if s == FY_REOPEN_LIFO_BLOCKED_KEY => AppError::IllegalState(t(
            "error-fiscal-year-reopen-blocked",
            "Réouverture impossible : un exercice postérieur est clôturé ; rouvrez d’abord les exercices postérieurs clôturés, en commençant par le plus récent.",
        )),
        other => AppError::from(other),
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /api/v1/fiscal-years` — liste les exercices de la company courante,
/// triés `start_date DESC` — par le service `exercices::lister` (Story 32-9a).
pub async fn list_fiscal_years(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<FiscalYearResponse>>, AppError> {
    let acces = ContexteAcces::depuis_api(&current_user);
    let list = exercices::lister(&acces, &state.lecture()).await?;
    Ok(Json(
        list.into_iter().map(FiscalYearResponse::from).collect(),
    ))
}

/// `GET /api/v1/fiscal-years/{id}` — détail d'un exercice scopé company.
///
/// Retourne 404 si l'exercice appartient à une autre company (anti-énumération).
pub async fn get_fiscal_year(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<FiscalYearResponse>, AppError> {
    let fy = fiscal_years::find_by_id_in_company(&state.pool, current_user.company_id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;
    Ok(Json(FiscalYearResponse::from(fy)))
}

/// `GET /api/v1/fiscal-years/{id}/closing-checks` — les contrôles préalables à
/// la clôture de l'exercice (Story 29-1a, #232) : neuf contrôles en lecture
/// seule (K8, Story v017-4a ; K9, Story 37-5), dans l'ordre de
/// `closing_checks` — tout rôle, clés d'API comprises.
///
/// Toutes les lectures passent par **un** instantané
/// ([`kesh_db::snapshot::ReadSnapshot`]) : elles voient la base au même
/// instant, sans verrou. Le 404 (exercice absent ou d'une autre société,
/// anti-énumération) est décidé **dans** l'instantané, par le `NotFound` de
/// `for_fiscal_year`, sans lecture préalable sur le pool. Aucun audit (une
/// lecture), aucun rejeu (aucune écriture).
pub async fn get_closing_checks(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<ClosingChecks>, AppError> {
    let mut snapshot = ReadSnapshot::begin(&state.pool).await?;
    // Sur l'erreur, l'instantané abandonné est annulé par son `Drop`.
    let checks =
        closing_checks::for_fiscal_year(snapshot.conn(), current_user.company_id, id).await?;
    snapshot.finish().await?;
    Ok(Json(checks))
}

/// `POST /api/v1/fiscal-years` — crée un exercice (Comptable+).
///
/// Le `company_id` est forcé depuis le JWT — tout champ `companyId` du payload
/// est ignoré (multi-tenant defense en profondeur).
pub async fn create_fiscal_year(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateFiscalYearRequest>,
) -> Result<(StatusCode, Json<FiscalYearResponse>), AppError> {
    borner(req.start_date, "startDate")?;
    borner(req.end_date, "endDate")?;
    // Pré-validation client-side back-up : nom non vide.
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Validation(t(
            "error-fiscal-year-name-empty",
            "Le nom de l'exercice est obligatoire",
        )));
    }

    let new = NewFiscalYear {
        company_id: current_user.company_id,
        name,
        start_date: req.start_date,
        end_date: req.end_date,
    };

    // Story 15-12a (AC 6) — rejouée sur interblocage : la création verrouille
    // ses antérieurs (pré-contrôles) puis ses postérieurs (garde de
    // l'invariant I) ; elle peut former un cycle avec une clôture ou une
    // contre-passation. Transaction unique, sûre à relancer.
    let fy = kesh_db::retry::retry_on_deadlock("fiscal_years::create", || {
        let new = new.clone();
        let pool = state.pool.clone();
        async move { fiscal_years::create(&pool, current_user.user_id, new).await }
    })
    .await
    .map_err(map_create_error)?;

    Ok((StatusCode::CREATED, Json(FiscalYearResponse::from(fy))))
}

/// `PUT /api/v1/fiscal-years/{id}` — renomme un exercice (Comptable+).
///
/// Seul le `name` est mutable. Les dates et le statut ne peuvent pas être
/// modifiés via cette route. Renommage autorisé même si `status='Closed'`
/// (le CO protège les montants/dates, pas un libellé descriptif).
///
/// Rejouée sur interblocage (Story 34-7, #945) : le renommage tient l'exercice,
/// puis demande son homonyme par la lecture verrouillante de doublon ; la
/// clôture tient ses antérieurs puis l'exercice, et la validation d'une facture
/// fournisseur le sien puis les autres exercices de son année. Victime, il
/// rendait 500 là où le pré-contrôle refuse en 400. Transaction unique, audit
/// compris, sûre à relancer ; un refus métier sort en `DbError::Invariant`, qui
/// ne se rejoue pas (preuve : `rejeu_interblocage_e2e.rs`).
pub async fn update_fiscal_year(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateFiscalYearRequest>,
) -> Result<Json<FiscalYearResponse>, AppError> {
    // Multi-tenant scoping : 404 si l'exercice n'existe pas dans cette company.
    let _existing = fiscal_years::find_by_id_in_company(&state.pool, current_user.company_id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;

    let fy = kesh_db::retry::retry_on_deadlock("fiscal_years::update_name", || {
        let name = req.name.clone();
        let pool = state.pool.clone();
        async move {
            fiscal_years::update_name(
                &pool,
                current_user.user_id,
                current_user.company_id,
                id,
                name,
            )
            .await
        }
    })
    .await
    .map_err(map_update_error)?;

    Ok(Json(FiscalYearResponse::from(fy)))
}

/// Story 37-2 (AC-5) — corps de `PUT /api/v1/fiscal-years/{id}/rate-kinds`.
/// Les deux genres sont obligatoires (`"day"` | `"month"`) : un corps
/// illisible, un champ absent ou une valeur hors liste est refusé par
/// l'extracteur (400 `VALIDATION_ERROR`) avant le handler.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateKindsRequest {
    /// Genre du cours d'ouverture.
    pub opening_rate_kind: RateKind,
    /// Genre du cours de clôture.
    pub closing_rate_kind: RateKind,
    /// U4 : appliquer le genre de clôture changé à l'ouverture de l'exercice
    /// suivant. Absent = `false`.
    #[serde(default)]
    pub apply_to_next_opening: bool,
}

crate::extractors::impl_validation_extractor!(
    RateKindsRequestExtractor,
    RateKindsRequest,
    "`openingRateKind` et `closingRateKind` \"day\" | \"month\", `applyToNextOpening` booléen facultatif"
);

/// Ce que la réponse de `PUT …/rate-kinds` dit de l'exercice suivant (U4).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NextFiscalYearRateKinds {
    /// L'exercice suivant.
    pub id: i64,
    /// Son genre d'ouverture, après.
    pub opening_rate_kind: RateKind,
    /// Vrai s'il a pris le genre de clôture réglé.
    pub applied: bool,
    /// `"closed"` s'il n'a pas été touché parce que clos.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<&'static str>,
}

/// Réponse de `PUT …/rate-kinds`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateKindsResponse {
    /// L'exercice réglé.
    pub fiscal_year: FiscalYearResponse,
    /// L'exercice suivant, quand le genre de clôture change, qu'il existe et
    /// que `applyToNextOpening` est vrai ; `null` sinon.
    pub next_fiscal_year: Option<NextFiscalYearRateKinds>,
}

/// `PUT /api/v1/fiscal-years/{id}/rate-kinds` — règle les genres des cours
/// d'ouverture et de clôture d'un exercice multi-devises ouvert (Story 37-2,
/// AC-5, U4), Administrateur ou Comptable.
///
/// Refus, dans l'ordre : 400 de l'extracteur → 404 (exercice d'une autre
/// société) → 409 `MULTI_CURRENCY_DISABLED` (`details {fiscalYearId}`) → 400
/// `FISCAL_YEAR_CLOSED`, mode et statut relus sous le `FOR UPDATE` de
/// l'exercice. No-op → 200 sans audit. Sinon une ligne d'audit
/// `fiscal_year.rate_kinds_updated` sur l'exercice réglé, `{ before, after,
/// nextFiscalYear: { id, before, after } | null, copied }`. Aucune écriture
/// comptable n'est modifiée ; les unités neuves reçoivent la copie des cours
/// déjà récupérés.
///
/// Rejouée sur interblocage, comme les autres routes d'exercice : elle tient
/// l'exercice (et le suivant) `FOR UPDATE`, puis prend `companies` en partage
/// par la clé étrangère de la copie — cycle possible avec une récupération ou
/// une autre route qui copie. Transaction unique, audit compris.
pub async fn update_rate_kinds(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    RateKindsRequestExtractor(req): RateKindsRequestExtractor,
) -> Result<Json<RateKindsResponse>, AppError> {
    use crate::audit::AuditActor;
    use kesh_db::entities::audit_log::NewAuditLogEntry;
    use serde_json::json;

    let user = &current_user;
    let req = &req;
    let maj = kesh_db::retry::retry_on_deadlock("fiscal_years::update_rate_kinds", || {
        let pool = state.pool.clone();
        async move {
            let mut tx = pool.begin().await.map_err(kesh_db::errors::map_db_error)?;
            let maj = fiscal_years::update_rate_kinds_in_tx(
                &mut tx,
                user.company_id,
                id,
                req.opening_rate_kind,
                req.closing_rate_kind,
                req.apply_to_next_opening,
            )
            .await?;
            if !maj.changed() {
                tx.rollback().await.map_err(kesh_db::errors::map_db_error)?;
                return Ok(maj);
            }
            let genres = |fy: &FiscalYear| {
                json!({
                    "openingRateKind": fy.opening_rate_kind.as_str(),
                    "closingRateKind": fy.closing_rate_kind.as_str(),
                })
            };
            let suivant = maj.next.as_ref().filter(|n| n.applied).map(|n| {
                json!({
                    "id": n.after.id,
                    "before": { "openingRateKind": n.before.opening_rate_kind.as_str() },
                    "after": { "openingRateKind": n.after.opening_rate_kind.as_str() },
                })
            });
            kesh_db::repositories::audit_log::insert_in_tx(
                &mut tx,
                NewAuditLogEntry::from_current_user(
                    user,
                    "fiscal_year.rate_kinds_updated",
                    "fiscal_year",
                    id,
                    Some(json!({
                        "before": genres(&maj.before),
                        "after": genres(&maj.after),
                        "nextFiscalYear": suivant,
                        "copied": maj.copied,
                    })),
                ),
            )
            .await?;
            tx.commit().await.map_err(kesh_db::errors::map_db_error)?;
            Ok(maj)
        }
    })
    .await?;

    let next_fiscal_year = maj.next.map(|n| NextFiscalYearRateKinds {
        id: n.after.id,
        opening_rate_kind: n.after.opening_rate_kind,
        applied: n.applied,
        reason: (!n.applied).then_some("closed"),
    });
    Ok(Json(RateKindsResponse {
        fiscal_year: FiscalYearResponse::from(maj.after),
        next_fiscal_year,
    }))
}

/// `POST /api/v1/fiscal-years/{id}/close` — transition Open → Closed (Comptable+).
///
/// Renvoie 409 `ILLEGAL_STATE_TRANSITION` si l'exercice est déjà clos
/// (mappé automatiquement via `AppError::Database(DbError::IllegalStateTransition)`),
/// et 409 `EARLIER_FISCAL_YEAR_OPEN` si un exercice antérieur est encore ouvert
/// (Story 15-12a, #543 — `details` nomment le plus ancien, à clôturer d'abord).
///
/// Rejouée sur interblocage (Story 15-12a, AC 6) : la clôture verrouille ses
/// antérieurs puis l'exercice, et forme un cycle avec une contre-passation ou
/// une annulation d'une écriture de l'exercice (preuve :
/// `rejeu_interblocage_e2e.rs`).
pub async fn close_fiscal_year(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<FiscalYearResponse>, AppError> {
    let _existing = fiscal_years::find_by_id_in_company(&state.pool, current_user.company_id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;

    let fy = kesh_db::retry::retry_on_deadlock("fiscal_years::close", || {
        fiscal_years::close(
            &state.pool,
            current_user.user_id,
            current_user.company_id,
            id,
        )
    })
    .await?;
    Ok(Json(FiscalYearResponse::from(fy)))
}

/// `POST /api/v1/fiscal-years/{id}/reopen` — transition Closed → Open (Story 14-2).
///
/// **Admin uniquement** (route montée dans `admin_routes`, scellé par
/// `require_admin_role` — aucun check de rôle dans ce handler) et **interdite
/// aux clés PAT** (`ensure_not_pat`, décision D5 : opération privilégiée
/// réglementaire, jamais une automatisation). Le motif est **obligatoire** et
/// **borné** ; il est tracé dans l'audit `fiscal_year.reopened`.
///
/// Réponses : 200 `FiscalYearResponse` (Open) ; 400 si motif vide / trop long ;
/// 404 si l'exercice n'existe pas dans cette company (anti-énumération) ; 409
/// `ILLEGAL_STATE_TRANSITION` (message distinct, D7) si déjà ouvert ou garde
/// LIFO violée ; 403 pour Comptable/Consultation/PAT ; 401 sans auth.
pub async fn reopen_fiscal_year(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<ReopenFiscalYearRequest>,
) -> Result<Json<FiscalYearResponse>, AppError> {
    // D5 — anti-PAT : la réouverture n'est pas une opération d'automatisation.
    ensure_not_pat(&current_user)?;

    // Multi-tenant scoping D'ABORD : 404 si l'exercice n'existe pas dans cette
    // company (anti-énumération, miroir `close_fiscal_year` — check d'existence
    // avant toute validation de body, cohérent avec les handlers-sœurs).
    let _existing = fiscal_years::find_by_id_in_company(&state.pool, current_user.company_id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;

    // Validation du motif au handler (D4). `trim()` puis `chars().count()`
    // (scalaires Unicode) — le frontend miroite cette borne sur `[...trimmed]`.
    let motif = req.motif.trim().to_string();
    if motif.is_empty() {
        return Err(AppError::Validation(t(
            "error-fiscal-year-reopen-motif-empty",
            "Le motif de réouverture est obligatoire.",
        )));
    }
    if motif.chars().count() > REOPEN_MOTIF_MAX {
        return Err(AppError::Validation(t(
            "error-fiscal-year-reopen-motif-too-long",
            "Le motif de réouverture est trop long (500 caractères maximum).",
        )));
    }

    // Story 33-1a (#926, C-e331-2) — rejouée sur interblocage : la réouverture tient
    // l'exercice puis demande ses postérieurs, alors que la validation d'une facture
    // fournisseur demande désormais les autres exercices de l'année. Transaction
    // unique, audit compris, sûre à relancer ; un refus métier sort en
    // `DbError::Invariant`, qui ne se rejoue pas.
    let fy = kesh_db::retry::retry_on_deadlock("fiscal_years::reopen", || {
        let motif = motif.clone();
        let pool = state.pool.clone();
        async move {
            fiscal_years::reopen(
                &pool,
                current_user.user_id,
                current_user.company_id,
                id,
                motif,
            )
            .await
        }
    })
    .await
    .map_err(map_reopen_error)?;

    Ok(Json(FiscalYearResponse::from(fy)))
}
