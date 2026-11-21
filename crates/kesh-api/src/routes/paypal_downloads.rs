//! Routes `POST /api/v1/paypal-downloads` et `GET
//! /api/v1/paypal-downloads/{jobId}` — Story 38-12a (Epic 38, phase 2, AC-7,
//! AC-8).
//!
//! Télécharge par l'API PayPal les transactions d'une période d'un **compte
//! PayPal** dont l'administrateur a posé les identifiants dans l'environnement
//! ([`crate::provider_env`]), en **tâche de fond** : le `POST` lance la tâche et
//! rend `202 { jobId }`, le `GET` en lit la progression et l'issue (forme
//! commune [`crate::provider_download::JobView`]).
//!
//! **Sessions seules**, Comptable et plus : une clé d'API est refusée sur les
//! deux routes, dans le gestionnaire, avant tout (`403
//! API_KEY_OPERATION_FORBIDDEN` : un secret de l'Administrateur sort de Kesh).
//!
//! **Ordre des refus du `POST`**, rien lancé : 403 (clé d'API) → `400
//! VALIDATION_ERROR` → `404 PROVIDER_ACCOUNT_NOT_FOUND` → `422
//! PROVIDER_API_NOT_CONFIGURED` → `400 PROVIDER_PERIOD_INVALID` → `409
//! PROVIDER_DOWNLOAD_RUNNING` (`details.jobId`, pour reprendre le suivi).
//!
//! **La tâche** lit toutes les fenêtres et les convertit **avant** toute
//! écriture ; les lignes passent par la ventilation de 38-11b puis par le cœur
//! commun ([`crate::routes::provider_imports`]) : mêmes doublons qu'à l'import
//! du CSV (par identifiant de transaction, brut ou `:FEE`), une transaction de
//! base, tout ou rien. Ce qui diffère d'un import de fichier : le nom (« PayPal
//! API <du>…<au> », `<au>` étant la fin **lue** : une `to` future est ramenée
//! au jour du jour, et la trace dit la même période), l'empreinte (les identifiants), aucun refus de fichier
//! déjà importé, la trace `paypal_download.created`. Un identifiant présent deux
//! fois dans le téléchargement (pages décalées pendant la lecture) est gardé une
//! fois s'il porte les mêmes devise, brut et frais ; sinon la tâche échoue en
//! `PROVIDER_RESPONSE_INVALID`. Une page décalée peut aussi faire **manquer** une
//! transaction de la fenêtre du jour : relancer la période plus tard la
//! rattrape, sans doublon.

use std::collections::HashMap;
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;

use kesh_db::entities::bank_account::PROVIDER_PAYPAL;
use kesh_import::MAX_CSV_LINE_ERRORS;
use kesh_import::paypal::{PaypalCsvError, PaypalLineError, PaypalRow};
use kesh_import::paypal_api::{PaypalApiTx, to_paypal_row};

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;
use crate::paypal_api::{self, HISTORY_YEARS, MAX_DAYS, PaypalClient};
use crate::provider_download::{
    JobHandle, JobView, Window, check_period, effective_to, period_windows, spawn_job,
};
use crate::provider_env::PAYPAL;
use crate::routes::paypal_imports::{
    PaypalImportResponse, check_transaction_id_lengths, provider_lines, ventilate,
};
use crate::routes::provider_imports::{ImportRequest, ImportSource, import_lines, load_balances};

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

/// `POST /api/v1/paypal-downloads` — voir l'en-tête du module.
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
        PROVIDER_PAYPAL,
        &account,
    )
    .await?;
    let config = state.config.providers.get(PAYPAL);
    let Some(series) = config.series_for(&account).cloned() else {
        return Err(AppError::ProviderApiNotConfigured {
            provider: PROVIDER_PAYPAL,
            account,
        });
    };
    let now = state.horloge.maintenant();
    check_period(request.from, request.to, now, HISTORY_YEARS).map_err(|reason| {
        AppError::ProviderPeriodInvalid {
            provider: PROVIDER_PAYPAL,
            reason,
        }
    })?;
    // Une `to` future n'est pas refusée : la période lue s'arrête au jour du
    // jour, et c'est elle que disent le nom de l'import et la trace (revue P1,
    // F1) ; la première fenêtre commence au plancher de l'historique (F11).
    let to = effective_to(request.to, now);
    let wins = period_windows(request.from, to, MAX_DAYS, now, HISTORY_YEARS);

    let downloads = state.provider_downloads.clone();
    // Le client se construit avant la tâche : un champ manquant ou un client
    // impossible se refuse ici, rien lancé (revue P1, F6).
    let gate = downloads.gate(PROVIDER_PAYPAL, config.parallel);
    let client = PaypalClient::new(
        &config,
        &series,
        gate,
        downloads.wait(),
        downloads.timeouts(),
    )?;
    let job = downloads
        .start(
            PROVIDER_PAYPAL,
            current_user.company_id,
            &account,
            Some(wins.len() as u32),
        )
        .map_err(|job_id| AppError::ProviderDownloadRunning {
            provider: PROVIDER_PAYPAL,
            job_id,
        })?;
    let job_id = job.id;
    let run = Run {
        pool: state.pool.clone(),
        current_user,
        account,
        from: request.from,
        to,
        windows: wins,
        parallel: config.parallel,
        now,
        job: job.clone(),
    };
    spawn_job(
        job,
        PROVIDER_PAYPAL,
        async move { run.execute(client).await },
    );

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({ "jobId": job_id })),
    ))
}

/// `GET /api/v1/paypal-downloads/{jobId}` — la tâche de la société de
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
        .view(PROVIDER_PAYPAL, job_id, current_user.company_id)
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
    windows: Vec<Window>,
    parallel: usize,
    now: DateTime<Utc>,
    job: JobHandle,
}

impl Run {
    /// La tâche : télécharger, convertir, ventiler, importer.
    async fn execute(self, client: PaypalClient) -> Result<serde_json::Value, AppError> {
        let window_count = self.windows.len();
        let fetched = paypal_api::download(
            Arc::new(client),
            self.windows,
            self.parallel,
            self.now,
            self.job.clone(),
        )
        .await?;
        let rows = rows_of(&fetched.transactions)?;
        check_transaction_id_lengths(&rows).map_err(|e| match e {
            PaypalCsvError::Lines(lines) => lines_invalid(lines),
            _ => AppError::ProviderResponseInvalid {
                provider: PROVIDER_PAYPAL,
                details: serde_json::json!({}),
            },
        })?;
        let ventilation = ventilate(&rows);

        let balances = load_balances(
            &self.pool,
            self.current_user.company_id,
            PROVIDER_PAYPAL,
            &self.account,
        )
        .await?;
        let filename = format!("PayPal API {}…{}", self.from, self.to);
        let mut audit_details = serde_json::Map::new();
        audit_details.insert("provider".into(), serde_json::json!(PROVIDER_PAYPAL));
        audit_details.insert("provider_account".into(), serde_json::json!(self.account));
        audit_details.insert(
            "period".into(),
            serde_json::json!({ "from": self.from, "to": self.to }),
        );
        audit_details.insert("windows".into(), serde_json::json!(window_count));
        audit_details.insert("skipped".into(), serde_json::json!(ventilation.skipped));
        let request = ImportRequest {
            provider: PROVIDER_PAYPAL,
            account: &self.account,
            filename: &filename,
            source: ImportSource::Download,
            retry_operation: "paypal_downloads::import",
            audit_action: "paypal_download.created",
            audit_details,
            by_currency: provider_lines(&ventilation),
        };
        let currencies = import_lines(&self.pool, &self.current_user, &balances, &request).await?;

        let response = PaypalImportResponse {
            account: self.account,
            currencies,
            skipped: ventilation.skipped,
        };
        let mut result = serde_json::to_value(response)
            .map_err(|_| AppError::Internal("compte rendu illisible".into()))?;
        if let Some(map) = result.as_object_mut() {
            map.insert(
                "lastRefreshed".into(),
                serde_json::json!(fetched.last_refreshed),
            );
        }
        Ok(result)
    }
}

/// `PROVIDER_RESPONSE_INVALID` des erreurs de ligne (forme de 38-11a).
fn lines_invalid(lines: Vec<PaypalLineError>) -> AppError {
    AppError::ProviderResponseInvalid {
        provider: PROVIDER_PAYPAL,
        details: serde_json::json!({ "lines": lines }),
    }
}

/// Convertit les transactions téléchargées (rang à partir de 1, heure de
/// Zurich) et écarte les répétitions identiques d'un identifiant — Story
/// 38-12a (AC-5, AC-7). Erreurs de ligne au plus [`MAX_CSV_LINE_ERRORS`] ; un
/// identifiant répété avec d'autres devise, brut ou frais → refus nommant
/// l'identifiant. Rien n'est écrit en cas de refus.
pub fn rows_of(transactions: &[PaypalApiTx]) -> Result<Vec<PaypalRow>, AppError> {
    let local = |t: chrono::DateTime<chrono::FixedOffset>| {
        kesh_core::horloge::heure_suisse(t.with_timezone(&Utc))
    };
    let mut rows: Vec<PaypalRow> = Vec::with_capacity(transactions.len());
    let mut errors: Vec<PaypalLineError> = Vec::new();
    for (i, tx) in transactions.iter().enumerate() {
        match to_paypal_row(tx, i + 1, &local) {
            Ok(row) => rows.push(row),
            Err(e) => {
                for err in e {
                    if errors.len() < MAX_CSV_LINE_ERRORS {
                        errors.push(err);
                    }
                }
            }
        }
    }
    if !errors.is_empty() {
        return Err(lines_invalid(errors));
    }
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut kept: Vec<PaypalRow> = Vec::with_capacity(rows.len());
    for row in rows {
        match seen.get(&row.transaction_id) {
            Some(&k) => {
                let first = &kept[k];
                if first.currency != row.currency
                    || first.gross != row.gross
                    || first.fee != row.fee
                {
                    return Err(AppError::ProviderResponseInvalid {
                        provider: PROVIDER_PAYPAL,
                        details: serde_json::json!({
                            "reason": "duplicate_transaction",
                            "transactionId": row.transaction_id,
                        }),
                    });
                }
            }
            None => {
                seen.insert(row.transaction_id.clone(), kept.len());
                kept.push(row);
            }
        }
    }
    Ok(kept)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kesh_import::paypal_api::{PAYPAL_CSV_LABELS_WITHOUT_CODE, PAYPAL_EVENT_TYPES};

    /// Libellés du classement de la revue PayPal (38-11c2, AC-4 : ventes,
    /// conversions, retraits), recopiés de sa fiche : la 38-11c2 se développe
    /// en parallèle et ses constantes ne sont pas encore sur cette branche. ⚠️ À
    /// la fusion, remplacer cette liste par ses constantes.
    const CLASSIFY_LABELS_38_11C2: &[&str] = &[
        "Website Payment",
        "Subscription Payment",
        "Express Checkout Payment",
        "Mobile Payment",
        "Shopping Cart Payment Received",
        "Payment Received",
        "General Currency Conversion",
        "User Initiated Currency Conversion",
        "General Withdrawal",
        "Withdraw Funds to Bank Account",
        "User Initiated Withdrawal",
    ];

    /// AC-5 — chaque libellé que lit la phase 1 est l'image d'au moins un code
    /// de la table, ou figure à l'inventaire fermé des libellés sans code (avec
    /// sa raison) : sans quoi une vente téléchargée tomberait « à examiner » et
    /// une paire de retenue ne serait plus neutralisée, en silence.
    #[test]
    fn la_table_couvre_les_libelles_de_la_phase_1() {
        let images: Vec<&str> = PAYPAL_EVENT_TYPES.iter().map(|(_, l)| *l).collect();
        let sans_code: Vec<&str> = PAYPAL_CSV_LABELS_WITHOUT_CODE
            .iter()
            .map(|(l, _)| *l)
            .collect();
        let lus = crate::routes::paypal_imports::PAYPAL_NEUTRAL_PAIR_TYPES
            .iter()
            .chain(CLASSIFY_LABELS_38_11C2);
        let mut non_resolus = Vec::new();
        for label in lus {
            if !images.contains(label) && !sans_code.contains(label) {
                non_resolus.push(*label);
            }
        }
        assert!(
            non_resolus.is_empty(),
            "libellés sans code ni raison : {non_resolus:?}"
        );
        // L'inventaire ne garde que des libellés lus par la phase 1.
        for label in &sans_code {
            assert!(
                crate::routes::paypal_imports::PAYPAL_NEUTRAL_PAIR_TYPES.contains(label)
                    || CLASSIFY_LABELS_38_11C2.contains(label),
                "{label} : inventaire périmé"
            );
        }
    }

    fn tx(id: &str, gross: &str, fee: Option<&str>, date: &str) -> PaypalApiTx {
        let fee = fee
            .map(|f| format!(r#","fee_amount":{{"currency_code":"EUR","value":"{f}"}}"#))
            .unwrap_or_default();
        serde_json::from_str(&format!(
            r#"{{"transaction_info":{{"transaction_id":"{id}","transaction_event_code":"T0007",
            "transaction_initiation_date":"{date}","transaction_status":"S",
            "transaction_amount":{{"currency_code":"EUR","value":"{gross}"}}{fee}}}}}"#
        ))
        .unwrap()
    }

    #[test]
    fn rang_heure_de_zurich_et_repetition_identique() {
        let txs = [
            tx("0TESTA", "10.00", Some("-0.50"), "2024-03-30T23:30:00Z"),
            tx("0TESTB", "5.00", None, "2024-07-01T12:00:00Z"),
            tx("0TESTA", "10.00", Some("-0.50"), "2024-03-30T23:30:00Z"),
        ];
        let rows = rows_of(&txs).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!((rows[0].line, rows[1].line), (1, 2));
        // 23:30 UTC le 30 mars 2024 = 00:30 le 31 à Zurich (+01:00, avant le
        // passage à l'heure d'été de 02:00).
        assert_eq!(rows[0].date, NaiveDate::from_ymd_opt(2024, 3, 31).unwrap());
        assert_eq!(rows[0].time_zone.as_deref(), Some("+01:00"));
        assert_eq!(rows[1].time_zone.as_deref(), Some("+02:00"));
    }

    #[test]
    fn repetition_differente_et_erreurs_de_ligne_refusent() {
        let txs = [
            tx("0TESTA", "10.00", Some("-0.50"), "2024-03-01T10:00:00Z"),
            tx("0TESTA", "10.00", Some("-0.60"), "2024-03-01T10:00:00Z"),
        ];
        match rows_of(&txs) {
            Err(AppError::ProviderResponseInvalid { details, .. }) => {
                assert_eq!(details["transactionId"], "0TESTA");
            }
            other => panic!("refus attendu : {other:?}"),
        }
        let txs = [tx("0TEST_A", "10.00", None, "2024-03-01T10:00:00Z")];
        match rows_of(&txs) {
            Err(AppError::ProviderResponseInvalid { details, .. }) => {
                assert_eq!(details["lines"][0]["code"], "TRANSACTION_ID");
                assert_eq!(details["lines"][0]["line"], 1);
            }
            other => panic!("refus attendu : {other:?}"),
        }
    }
}
