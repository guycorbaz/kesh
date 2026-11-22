//! Routes `POST /api/v1/stripe-downloads` et `GET
//! /api/v1/stripe-downloads/{jobId}` — Story 40-8b (Epic 40, phase 2, AC-1 à
//! AC-3).
//!
//! Télécharge par l'API Stripe les versements **automatiques payés** d'une
//! période (filtre sur leur date d'arrivée, jours de Zurich) d'un **compte
//! Stripe** dont l'administrateur a posé la clé restreinte dans
//! l'environnement ([`crate::provider_env`], [`crate::stripe::STRIPE`]), en
//! **tâche de fond** sur le socle commun ([`crate::provider_download`]) : le
//! `POST` lance la tâche et rend `202 { jobId }`, le `GET` en lit la
//! progression et l'issue (forme commune [`JobView`]).
//!
//! **Sessions seules**, Comptable et plus : une clé d'API est refusée sur les
//! deux routes, dans le gestionnaire, avant tout (`403
//! API_KEY_OPERATION_FORBIDDEN` : la clé de l'Administrateur sort de Kesh).
//!
//! **Ordre des refus du `POST`**, rien lancé : 403 (clé d'API) → `400
//! VALIDATION_ERROR` → `404 PROVIDER_ACCOUNT_NOT_FOUND` → `422
//! PROVIDER_API_NOT_CONFIGURED` → `400 PROVIDER_PERIOD_INVALID` → `409
//! PROVIDER_DOWNLOAD_RUNNING` (`details: { provider, jobId, from, to }`, comme
//! PayPal). La période se contrôle par la règle commune
//! ([`check_period`]) : `from_after_to`, `future` (début après le jour de
//! Zurich), `too_old` (au-delà de [`HISTORY_YEARS`] ans) ; une fin future est
//! ramenée au jour du jour ([`effective_to`]), et c'est la période lue que
//! disent le nom de l'import, la trace et le `409`. Aucun plafond de durée :
//! Stripe lit une période entière d'**une** fenêtre, par pages de 100.
//!
//! **Progression** : rien pendant la lecture de la liste des versements
//! (`steps.total` nul), puis une étape par versement automatique lu
//! ([`fetch_rows_tracked`], « versement 5/12 »). **Rythme et reprise** : ceux
//! du socle, sans variante ([`crate::provider_download::with_retry`],
//! `KESH_STRIPE_PARALLEL` versements en cours au plus).
//!
//! **La tâche** lit tout et contrôle tout **avant** toute écriture : les
//! lignes du client ([`crate::stripe::rows`]), puis la longueur des
//! identifiants ([`check_id_lengths`], la borne de l'import du CSV ; revue P1
//! de la 40-8a, L5), puis la ventilation de l'import du CSV
//! ([`groups_by_currency`]) et le cœur commun
//! ([`crate::routes::provider_imports`]) : mêmes doublons qu'à l'import du
//! rapport (identifiant de la transaction de solde, brut ou `:FEE`, et du
//! versement), une transaction de base, tout ou rien. Ce qui diffère d'un
//! import de fichier : le nom (« Stripe API <du>…<au> »), l'empreinte (les
//! identifiants, [`ImportSource::Download`]), **aucun** refus de fichier déjà
//! importé (relancer la même période rend « 0 insérée »), la trace
//! `stripe_download.created`, et le compte rendu, qui ajoute les versements
//! **manuels** de la période, non lus (`manualPayouts`, « à examiner »).

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use kesh_db::entities::bank_account::PROVIDER_STRIPE;
use kesh_import::stripe::StripeCsvError;

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;
use crate::provider_download::{
    JobHandle, JobView, Window, check_period, effective_to, period_windows, spawn_job,
};
use crate::routes::provider_imports::{ImportRequest, ImportSource, import_lines, load_balances};
use crate::routes::stripe_imports::{StripeImportResponse, check_id_lengths, groups_by_currency};
use crate::stripe::{HISTORY_YEARS, STRIPE, StripeClient, fetch_rows_tracked};

/// Corps du `POST`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DownloadRequest {
    account: String,
    /// Bornée à la plage des années de la base (`dates_recues`, Story 33-4).
    #[serde(deserialize_with = "crate::dates_recues::date")]
    from: NaiveDate,
    #[serde(deserialize_with = "crate::dates_recues::date")]
    to: NaiveDate,
}

/// Le compte rendu d'un téléchargement réussi (`result` de la tâche) : celui
/// de l'import du rapport CSV ([`StripeImportResponse`]), plus les versements
/// manuels de la période, que le téléchargement ne lit pas.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StripeDownloadResult {
    #[serde(flatten)]
    pub import: StripeImportResponse,
    /// Identifiants des versements **manuels** payés de la période, triés
    /// (« versement manuel à examiner » : ses transactions ne se lisent pas).
    pub manual_payouts: Vec<String>,
}

/// `POST /api/v1/stripe-downloads` — voir l'en-tête du module.
pub async fn create(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    body: Bytes,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    if current_user.api_key_id.is_some() {
        return Err(AppError::ApiKeyOperationForbidden);
    }
    let request: DownloadRequest = serde_json::from_slice(&body).map_err(|_| {
        AppError::Validation("corps attendu : { account, from, to } (dates AAAA-MM-JJ)".into())
    })?;
    let account = request.account.trim().to_string();
    if account.is_empty() {
        return Err(AppError::Validation("champ `account` manquant".into()));
    }

    load_balances(
        &state.pool,
        current_user.company_id,
        PROVIDER_STRIPE,
        &account,
    )
    .await?;
    let config = state.config.providers.get(STRIPE);
    let Some(series) = config.series_for(&account).cloned() else {
        return Err(AppError::ProviderApiNotConfigured {
            provider: PROVIDER_STRIPE,
            account,
        });
    };
    let now = state.horloge.maintenant();
    check_period(request.from, request.to, now, HISTORY_YEARS).map_err(|reason| {
        AppError::ProviderPeriodInvalid {
            provider: PROVIDER_STRIPE,
            reason,
        }
    })?;
    let to = effective_to(request.to, now);
    // Une seule fenêtre : Stripe ne borne pas la période d'une liste. Elle peut
    // manquer si la période commence à l'instant même (première seconde du
    // jour) : elle se lit alors vide.
    let window = period_windows(request.from, to, u32::MAX, now, HISTORY_YEARS)
        .into_iter()
        .next()
        .unwrap_or_else(|| Window::new(request.from, to, now));

    let downloads = state.provider_downloads.clone();
    // Le client se construit avant la tâche : un champ manquant ou un client
    // impossible se refuse ici, rien lancé.
    let gate = downloads.gate(PROVIDER_STRIPE, config.parallel);
    let client = StripeClient::new(
        &config,
        &series,
        gate,
        downloads.wait(),
        downloads.timeouts(),
    )?;
    let job = downloads
        .start(
            PROVIDER_STRIPE,
            current_user.company_id,
            &account,
            (request.from, to),
            None,
        )
        .map_err(|running| AppError::ProviderDownloadRunning {
            provider: PROVIDER_STRIPE,
            job_id: running.id,
            from: running.from,
            to: running.to,
        })?;
    let job_id = job.id;
    let run = Run {
        pool: state.pool.clone(),
        current_user,
        account,
        from: request.from,
        to,
        window,
        parallel: config.parallel,
        job: job.clone(),
    };
    spawn_job(
        job,
        PROVIDER_STRIPE,
        async move { run.execute(client).await },
    );

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({ "jobId": job_id })),
    ))
}

/// `GET /api/v1/stripe-downloads/{jobId}` — la tâche de la société de
/// l'appelant (sinon `404 NOT_FOUND`).
pub async fn show(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(job_id): Path<u64>,
) -> Result<Json<JobView>, AppError> {
    if current_user.api_key_id.is_some() {
        return Err(AppError::ApiKeyOperationForbidden);
    }
    state
        .provider_downloads
        .view(PROVIDER_STRIPE, job_id, current_user.company_id)
        .map(Json)
        .ok_or(AppError::Database(kesh_db::errors::DbError::NotFound))
}

/// Ce qu'une tâche emporte.
struct Run {
    pool: sqlx::MySqlPool,
    current_user: CurrentUser,
    account: String,
    from: NaiveDate,
    to: NaiveDate,
    window: Window,
    parallel: usize,
    job: JobHandle,
}

impl Run {
    /// La tâche : télécharger, contrôler, ventiler, importer.
    async fn execute(self, client: StripeClient) -> Result<serde_json::Value, AppError> {
        let fetched = fetch_rows_tracked(
            Arc::new(client),
            self.window,
            self.parallel,
            Some(self.job.clone()),
        )
        .await?;
        check_id_lengths(&fetched.rows).map_err(response_invalid)?;
        let (by_currency, payouts) = groups_by_currency(&fetched.rows);

        let balances = load_balances(
            &self.pool,
            self.current_user.company_id,
            PROVIDER_STRIPE,
            &self.account,
        )
        .await?;
        let filename = format!("Stripe API {}…{}", self.from, self.to);
        let mut audit_details = serde_json::Map::new();
        audit_details.insert("provider".into(), serde_json::json!(PROVIDER_STRIPE));
        audit_details.insert("provider_account".into(), serde_json::json!(self.account));
        audit_details.insert(
            "period".into(),
            serde_json::json!({ "from": self.from, "to": self.to }),
        );
        audit_details.insert("payouts".into(), serde_json::json!(payouts));
        audit_details.insert(
            "manual_payouts".into(),
            serde_json::json!(fetched.manual_payouts),
        );
        let request = ImportRequest {
            provider: PROVIDER_STRIPE,
            account: &self.account,
            filename: &filename,
            source: ImportSource::Download,
            retry_operation: "stripe_downloads::import",
            audit_action: "stripe_download.created",
            audit_details,
            by_currency,
        };
        let currencies = import_lines(&self.pool, &self.current_user, &balances, &request).await?;

        serde_json::to_value(StripeDownloadResult {
            import: StripeImportResponse {
                account: self.account,
                currencies,
                payouts,
            },
            manual_payouts: fetched.manual_payouts,
        })
        .map_err(|_| AppError::Internal("compte rendu illisible".into()))
    }
}

/// `PROVIDER_RESPONSE_INVALID` d'un refus des lignes lues (forme des erreurs
/// de ligne du rapport CSV : `details.lines`) ; rien n'est écrit.
fn response_invalid(e: StripeCsvError) -> AppError {
    let details = match e {
        StripeCsvError::Lines(lines) => serde_json::json!({ "lines": lines }),
        _ => serde_json::json!({}),
    };
    AppError::ProviderResponseInvalid {
        provider: PROVIDER_STRIPE,
        details,
    }
}
