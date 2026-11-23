//! Routes de la **revue** et de l'**acceptation** PayPal — Story 38-11c2
//! (AC-6, AC-7), gestionnaires minces sur le moteur commun
//! ([`super::provider_accept`]) avec le paramétrage PayPal
//! ([`kesh_reconciliation::paypal::SPEC`]).
//!
//! - `GET /api/v1/paypal/review?account=<libellé>&fiscalYearId=<id>` —
//!   Comptable et plus, clés d'API admises en lecture ; lecture seule, une
//!   transaction de lecture ; `404 PROVIDER_ACCOUNT_NOT_FOUND`, `404
//!   FISCAL_YEAR_NOT_FOUND`.
//! - `POST /api/v1/paypal/accept` — Comptable et plus, clés d'API
//!   `read-write` ; corps `{ account, proposals: [ {type, …} ] }`, réponse `{
//!   accepted, failed }` (*Pattern batch*).
//!
//! ⛔ Le gestionnaire d'acceptation s'appelle [`accept_proposals`], **jamais**
//! `post_accept` : l'exemption `RETRY_WITH_AUTORISE` du registre des routes
//! désigne une fonction par son seul nom. Il rejoue sur interblocage par
//! l'enveloppe `crate::retry::retry_app_on_deadlock` (opération
//! `paypal::accept`), non par `retry_with`.

use axum::extract::{Query, State};
use axum::{Extension, Json};
use kesh_reconciliation::paypal::SPEC;

use super::provider_accept::{
    self, AcceptBodyExtractor, BatchOutcome, ProviderAcceptResponse, ReviewQuery, ReviewResponse,
};
use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;

/// `GET /api/v1/paypal/review` — voir l'en-tête du module.
pub async fn review(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ReviewQuery>,
) -> Result<Json<ReviewResponse>, AppError> {
    provider_accept::review(&state.pool, &current_user, &SPEC, query)
        .await
        .map(Json)
}

/// `POST /api/v1/paypal/accept` — voir l'en-tête du module.
pub async fn accept_proposals(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    AcceptBodyExtractor(body): AcceptBodyExtractor,
) -> Result<Json<ProviderAcceptResponse>, AppError> {
    let prepared =
        provider_accept::prepare_accept(&state.pool, &current_user, &SPEC, body, &[]).await?;
    if prepared.is_empty() {
        return Ok(Json(prepared.finish(BatchOutcome::default())));
    }
    let outcome = crate::retry::retry_app_on_deadlock("paypal::accept", || {
        provider_accept::accept_once(&state.pool, &SPEC, &prepared)
    })
    .await?;
    Ok(Json(prepared.finish(outcome)))
}
