//! Routes des rappels débiteurs (Story 21-5a, #231) — données & éligibilité.
//!
//! - `GET  /api/v1/dunning/reminders` (Comptable+) : factures à rappeler, groupées par contact.
//! - `PUT  /api/v1/invoices/{id}/dunning-pause`  (Comptable+) : suspend les rappels.
//! - `PUT  /api/v1/invoices/{id}/dunning-resume` (Comptable+) : reprend les rappels.
//! - `POST /api/v1/invoices/{id}/reminders/manual` (Comptable+) : enregistre un rappel papier.
//! - `POST /api/v1/invoices/{id}/reminders/{reminderId}/cancel` (Admin) : annule un rappel (soft).
//! - `GET  /api/v1/invoices/{id}/reminders` (tous rôles) : historique des rappels.
//!
//! Scoping company systématique (anti-IDOR : cross-tenant → 404, jamais 403). L'envoi
//! e-mail (unitaire/lot) est **21-5b** ; le frontend **21-6**.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{NaiveDateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::{InvoiceReminder, NewInvoiceReminder, ReminderChannel};
use kesh_db::errors::DbError;
use kesh_db::repositories::{
    audit_log, dunning_eligibility, dunning_levels, invoice_reminders, invoices,
};

use crate::AppState;
use crate::audit::AuditActor;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;

// ---------- DTOs de réponse ----------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderCandidateResponse {
    pub invoice_id: i64,
    pub invoice_number: Option<String>,
    pub due_date: chrono::NaiveDate,
    pub current_level: i16,
    pub next_level: Option<i16>,
    pub terminal: bool,
    pub last_reminder_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactGroup {
    pub contact_id: i64,
    pub contact_name: String,
    pub has_email: bool,
    pub invoices: Vec<ReminderCandidateResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderListResponse {
    pub groups: Vec<ContactGroup>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderResponse {
    pub id: i64,
    pub level_number: i16,
    pub fee_amount: Decimal,
    pub sent_at: NaiveDateTime,
    pub channel: String,
    pub sent_to: Option<String>,
    pub subject: String,
    pub body: String,
    pub note: Option<String>,
    pub cancelled_at: Option<NaiveDateTime>,
}

impl From<InvoiceReminder> for ReminderResponse {
    fn from(r: InvoiceReminder) -> Self {
        Self {
            id: r.id,
            level_number: r.level_number,
            fee_amount: r.fee_amount,
            sent_at: r.sent_at,
            channel: r.channel,
            sent_to: r.sent_to,
            subject: r.subject,
            body: r.body,
            note: r.note,
            cancelled_at: r.cancelled_at,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DunningPauseResponse {
    pub invoice_id: i64,
    pub dunning_paused_at: Option<NaiveDateTime>,
    pub dunning_paused_note: Option<String>,
    pub version: i32,
}

// ---------- DTOs de requête ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PauseBody {
    pub version: i32,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumeBody {
    pub version: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualReminderBody {
    pub level_number: i16,
    pub sent_at: NaiveDateTime,
    pub note: Option<String>,
}

fn validate_version(version: i32) -> Result<(), AppError> {
    if version < 0 {
        return Err(AppError::Validation("version invalide".to_string()));
    }
    Ok(())
}

/// Convertit une `DbError` de `set_dunning_pause` en `AppError` (la reprise sur une
/// facture non suspendue remonte `InvalidInput("notPaused")`).
fn map_pause_error(e: DbError) -> AppError {
    match e {
        DbError::InvalidInput(code) if code == "notPaused" => AppError::InvoiceNotPaused,
        other => other.into(),
    }
}

// ---------- Handlers ----------

/// Liste des factures à rappeler, groupées par contact (Comptable+).
pub async fn list_reminders(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<ReminderListResponse>, AppError> {
    let candidates =
        dunning_eligibility::list_reminder_candidates(&state.pool, current_user.company_id).await?;

    // Groupage par contact (les candidates sont déjà ordonnées par nom de contact).
    let mut groups: Vec<ContactGroup> = Vec::new();
    for c in candidates {
        let candidate = ReminderCandidateResponse {
            invoice_id: c.invoice_id,
            invoice_number: c.invoice_number,
            due_date: c.due_date,
            current_level: c.current_level,
            next_level: c.next_level,
            terminal: c.terminal,
            last_reminder_at: c.last_reminder_at,
        };
        match groups.last_mut() {
            Some(g) if g.contact_id == c.contact_id => g.invoices.push(candidate),
            _ => groups.push(ContactGroup {
                contact_id: c.contact_id,
                contact_name: c.contact_name,
                has_email: c.has_email,
                invoices: vec![candidate],
            }),
        }
    }

    Ok(Json(ReminderListResponse { groups }))
}

/// Suspend les rappels d'une facture (Comptable+).
pub async fn pause_dunning(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(body): Json<PauseBody>,
) -> Result<Json<DunningPauseResponse>, AppError> {
    validate_version(body.version)?;
    let invoice = invoices::set_dunning_pause(
        &state.pool,
        current_user.user_id,
        id,
        current_user.company_id,
        body.version,
        true,
        body.note,
    )
    .await
    .map_err(map_pause_error)?;
    Ok(Json(DunningPauseResponse {
        invoice_id: invoice.id,
        dunning_paused_at: invoice.dunning_paused_at,
        dunning_paused_note: invoice.dunning_paused_note,
        version: invoice.version,
    }))
}

/// Reprend les rappels d'une facture suspendue (Comptable+).
pub async fn resume_dunning(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(body): Json<ResumeBody>,
) -> Result<Json<DunningPauseResponse>, AppError> {
    validate_version(body.version)?;
    let invoice = invoices::set_dunning_pause(
        &state.pool,
        current_user.user_id,
        id,
        current_user.company_id,
        body.version,
        false,
        None,
    )
    .await
    .map_err(map_pause_error)?;
    Ok(Json(DunningPauseResponse {
        invoice_id: invoice.id,
        dunning_paused_at: invoice.dunning_paused_at,
        dunning_paused_note: invoice.dunning_paused_note,
        version: invoice.version,
    }))
}

/// Enregistre un rappel manuel (papier) — le cycle avance sans e-mail (Comptable+).
pub async fn record_manual_reminder(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(body): Json<ManualReminderBody>,
) -> Result<(StatusCode, Json<ReminderResponse>), AppError> {
    if body.level_number < 1 {
        return Err(AppError::Validation(
            "niveau de rappel invalide".to_string(),
        ));
    }
    // Garde : une date d'envoi ne peut pas être dans le futur (gèlerait le cycle).
    if body.sent_at > Utc::now().naive_utc() {
        return Err(AppError::ReminderDateInFuture);
    }

    // Gardes d'éligibilité sur la facture (scopée company).
    let (invoice, _lines) =
        invoices::find_by_id_with_lines(&state.pool, current_user.company_id, id)
            .await?
            .ok_or(DbError::NotFound)?;
    if invoice.status != "validated" {
        return Err(AppError::InvoiceNotValidated);
    }
    if invoice.paid_at.is_some() {
        return Err(AppError::InvoiceAlreadyPaid);
    }

    // Snapshot du frais du niveau visé (config courante).
    let level = dunning_levels::find_by_level_number(
        &state.pool,
        current_user.company_id,
        body.level_number,
    )
    .await?
    .ok_or(AppError::DunningLevelNotFound)?;

    let subject = format!("Rappel manuel — niveau {}", body.level_number);
    let reminder_body = body.note.clone().unwrap_or_default();

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;

    let created = invoice_reminders::insert_in_tx(
        &mut tx,
        &NewInvoiceReminder {
            company_id: current_user.company_id,
            invoice_id: id,
            level_number: body.level_number,
            fee_amount: level.fee_amount,
            sent_at: body.sent_at,
            channel: ReminderChannel::Manual,
            sent_to: None,
            subject,
            body: reminder_body,
            note: body.note,
            actor_user_id: Some(current_user.user_id),
        },
    )
    .await?;

    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::from_current_user(
            &current_user,
            "invoice.reminder_sent",
            "invoice",
            id,
            Some(serde_json::json!({
                "reminderId": created.id,
                "levelNumber": created.level_number,
                "channel": "manual",
            })),
        ),
    )
    .await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit tx: {e}")))?;

    Ok((StatusCode::CREATED, Json(created.into())))
}

/// Annule (soft) un rappel envoyé par erreur (Admin). Idempotent.
pub async fn cancel_reminder(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path((id, reminder_id)): Path<(i64, i64)>,
) -> Result<Json<ReminderResponse>, AppError> {
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;

    let reminder =
        invoice_reminders::find_by_id_for_company(&mut tx, current_user.company_id, reminder_id)
            .await?
            .filter(|r| r.invoice_id == id)
            .ok_or(DbError::NotFound)?;

    let newly_cancelled =
        invoice_reminders::cancel_in_tx(&mut tx, current_user.company_id, reminder_id).await?;

    if newly_cancelled {
        audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::from_current_user(
                &current_user,
                "invoice.reminder_cancelled",
                "invoice",
                id,
                Some(serde_json::json!({
                    "reminderId": reminder_id,
                    "levelNumber": reminder.level_number,
                })),
            ),
        )
        .await?;
    }

    // Re-lecture pour refléter `cancelled_at` (posé cette tx ou déjà présent).
    let updated =
        invoice_reminders::find_by_id_for_company(&mut tx, current_user.company_id, reminder_id)
            .await?
            .ok_or(DbError::NotFound)?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit tx: {e}")))?;

    Ok(Json(updated.into()))
}

/// Historique des rappels d'une facture (tous rôles authentifiés).
pub async fn list_reminder_history(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<ReminderResponse>>, AppError> {
    let reminders =
        invoice_reminders::list_for_invoice(&state.pool, current_user.company_id, id).await?;
    Ok(Json(reminders.into_iter().map(Into::into).collect()))
}
