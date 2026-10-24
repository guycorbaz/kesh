//! Story 31-1c (#503) — routes de la sauvegarde automatique et des sauvegardes
//! du serveur (Administrateur ; refusées aux clés d'API par la couche
//! `require_not_pat` du bloc `admin_routes`).
//!
//! - `GET  /api/v1/admin/backup-schedule` — réglage, état, `nextDueAt`,
//!   `overdue` (calculés par le serveur : le navigateur ne compare aucune date
//!   à sa propre horloge) ;
//! - `PUT  /api/v1/admin/backup-schedule` — réglage (verrou optimiste, audit
//!   `admin.backup_schedule_updated`) ;
//! - `POST /api/v1/admin/backup-schedule/run` — « Sauvegarder maintenant » :
//!   `auto_backup::run_once(…, Trigger::Manual)` dans une tâche attendue (un
//!   client parti n'interrompt pas l'écriture) ; `Skipped` → 409
//!   `BACKUP_IN_PROGRESS` ; audit `admin.backup_run` ;
//! - `GET  /api/v1/admin/backups` — `{ importMaxMib, items }` ;
//! - `GET  /api/v1/admin/backups/{kind}/{name}` — téléchargement, qui porte
//!   en plus `ensure_not_pat` (défense en profondeur là où sort le secret) ;
//!   audit best-effort `admin.backup_downloaded`.
//!
//! Le chemin du dossier de sauvegarde n'est jamais exposé (journal seulement).
//! Ces handlers ne touchent pas `AppState::backup_lock` : c'est `run_once` qui
//! le prend.

use std::path::PathBuf;
use std::sync::Arc;

use axum::Extension;
use axum::Json;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use kesh_db::entities::AUDIT_ENTITY_ID_NONE;
use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::repositories::backup_schedule::{self, BackupSchedule, BackupScheduleSettings};

use crate::AppState;
use crate::audit::AuditActor;
use crate::auto_backup::{
    self, BackupKind, RealBackupEnv, RunOutcome, RunStatus, Trigger, is_overdue, next_due_at,
};
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;
use crate::routes::api_keys::ensure_not_pat;

/// Une date UTC de l'application, rendue avec son `Z`.
fn utc(t: Option<NaiveDateTime>) -> Option<chrono::DateTime<Utc>> {
    t.map(|t| t.and_utc())
}

/// Réglage et état de la sauvegarde automatique.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupScheduleResponse {
    pub enabled: bool,
    pub interval_hours: i32,
    pub retention_count: i32,
    pub version: i32,
    pub last_attempt_at: Option<chrono::DateTime<Utc>>,
    pub last_success_at: Option<chrono::DateTime<Utc>>,
    pub last_success_file: Option<String>,
    pub last_success_bytes: Option<i64>,
    pub last_status: Option<String>,
    pub last_files_not_included: Option<i32>,
    pub last_complete_file: Option<String>,
    /// Prochaine sauvegarde prévue (même règle qu'`is_due`) ; `null` si
    /// désactivée.
    pub next_due_at: Option<chrono::DateTime<Utc>>,
    /// Dernière réussite datant d'au moins deux intervalles (ou datée du
    /// futur).
    pub overdue: bool,
}

impl BackupScheduleResponse {
    fn new(s: BackupSchedule, now: NaiveDateTime) -> Self {
        Self {
            next_due_at: utc(next_due_at(&s, now)),
            overdue: is_overdue(&s, now),
            enabled: s.enabled,
            interval_hours: s.interval_hours,
            retention_count: s.retention_count,
            version: s.version,
            last_attempt_at: utc(s.last_attempt_at),
            last_success_at: utc(s.last_success_at),
            last_success_file: s.last_success_file,
            last_success_bytes: s.last_success_bytes,
            last_status: s.last_status,
            last_files_not_included: s.last_files_not_included,
            last_complete_file: s.last_complete_file,
        }
    }
}

async fn etat(state: &AppState) -> Result<BackupScheduleResponse, AppError> {
    let s = backup_schedule::get(&state.pool).await?;
    Ok(BackupScheduleResponse::new(s, Utc::now().naive_utc()))
}

/// `GET /api/v1/admin/backup-schedule`.
pub async fn get_backup_schedule(
    State(state): State<AppState>,
) -> Result<Json<BackupScheduleResponse>, AppError> {
    Ok(Json(etat(&state).await?))
}

/// Corps de `PUT /api/v1/admin/backup-schedule`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBackupScheduleRequest {
    pub enabled: bool,
    pub interval_hours: i32,
    pub retention_count: i32,
    pub version: i32,
}

/// `PUT /api/v1/admin/backup-schedule` — 400 `VALIDATION_ERROR` hors domaine
/// (rien d'écrit, aucun audit) ; 409 `OPTIMISTIC_LOCK_CONFLICT` sur une version
/// périmée.
pub async fn update_backup_schedule(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<UpdateBackupScheduleRequest>,
) -> Result<Json<BackupScheduleResponse>, AppError> {
    let settings = BackupScheduleSettings {
        enabled: req.enabled,
        interval_hours: req.interval_hours,
        retention_count: req.retention_count,
    };
    if !settings.is_valid() {
        return Err(AppError::Validation(
            "Réglage de sauvegarde hors domaine : fréquence 6, 12, 24 ou 168 heures, \
             de 1 à 60 sauvegardes conservées."
                .into(),
        ));
    }
    let s =
        backup_schedule::update_settings(&state.pool, req.version, current_user.user_id, settings)
            .await?;
    Ok(Json(BackupScheduleResponse::new(s, Utc::now().naive_utc())))
}

/// `POST /api/v1/admin/backup-schedule/run` — « Sauvegarder maintenant ».
/// Rend l'état après la tentative (200, quel que soit son statut : l'échec se
/// lit dans `lastStatus`), ou 409 `BACKUP_IN_PROGRESS` si le verrou était pris.
pub async fn run_backup_now(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<BackupScheduleResponse>, AppError> {
    let etat_tache = state.clone();
    let tache = tokio::spawn(async move {
        auto_backup::run_once(
            &etat_tache,
            Arc::new(RealBackupEnv),
            Utc::now().naive_utc(),
            Trigger::Manual,
        )
        .await
    });
    let issue = tache
        .await
        .map_err(|e| AppError::Internal(format!("tâche de sauvegarde : {e}")))??;
    let statut = match issue {
        RunOutcome::Skipped => return Err(AppError::BackupInProgress),
        RunOutcome::Done(statut) => statut,
    };
    let reponse = etat(&state).await?;
    emit_audit(
        &state,
        NewAuditLogEntry::from_current_user(
            &current_user,
            "admin.backup_run",
            "installation",
            AUDIT_ENTITY_ID_NONE,
            Some(serde_json::json!({
                "status": RunStatus::as_backup_status(statut).as_str(),
                "file": match statut {
                    RunStatus::Success(_) => reponse.last_success_file.clone(),
                    RunStatus::Failure(_) => None,
                },
            })),
        ),
    )
    .await;
    Ok(Json(reponse))
}

/// Une ligne de `GET /api/v1/admin/backups`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerBackupItem {
    pub name: String,
    pub kind: &'static str,
    pub size_bytes: u64,
    pub modified_at: Option<chrono::DateTime<Utc>>,
}

/// Réponse de `GET /api/v1/admin/backups`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerBackupsResponse {
    /// Taille d'import de l'installation (`KESH_ADMIN_IMPORT_MAX_MB`), une
    /// fois par réponse : une ligne plus grosse ne peut pas être restaurée par
    /// l'écran (C-31-1-52).
    pub import_max_mib: u32,
    pub items: Vec<ServerBackupItem>,
}

async fn lister(state: &AppState) -> Result<Vec<auto_backup::ServerBackup>, AppError> {
    let dossier = PathBuf::from(&state.config.admin_backup_dir);
    tokio::task::spawn_blocking(move || auto_backup::lister_les_sauvegardes(&dossier))
        .await
        .map_err(|e| AppError::Internal(format!("liste des sauvegardes : {e}")))?
        .map_err(|e| AppError::Internal(format!("liste des sauvegardes : {e}")))
}

/// `GET /api/v1/admin/backups`.
pub async fn list_backups(
    State(state): State<AppState>,
) -> Result<Json<ServerBackupsResponse>, AppError> {
    let items = lister(&state)
        .await?
        .into_iter()
        .map(|b| ServerBackupItem {
            name: b.name,
            kind: b.kind.as_str(),
            size_bytes: b.size,
            modified_at: b.modified.map(chrono::DateTime::<Utc>::from),
        })
        .collect();
    Ok(Json(ServerBackupsResponse {
        import_max_mib: state.config.admin_import_max_mib,
        items,
    }))
}

/// `GET /api/v1/admin/backups/{kind}/{name}` — le nom est validé par
/// l'expression de son type **puis** retrouvé dans la liste du dossier
/// (fichier régulier) : il n'est jamais concaténé tel quel à un chemin. 404
/// sinon.
pub async fn download_backup(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path((kind, name)): Path<(String, String)>,
) -> Result<Response, AppError> {
    ensure_not_pat(&current_user)?;
    let introuvable = || AppError::Database(kesh_db::errors::DbError::NotFound);
    let kind = BackupKind::parse(&kind).ok_or_else(introuvable)?;
    if !kind.nom_valide(&name) {
        return Err(introuvable());
    }
    let sauvegarde = lister(&state)
        .await?
        .into_iter()
        .find(|b| b.kind == kind && b.name == name)
        .ok_or_else(introuvable)?;
    // Disparu, devenu un lien ou plus un fichier régulier depuis la liste :
    // 404, comme un nom absent (C-31-1-89).
    let chemin = sauvegarde.path.clone();
    let fichier = tokio::task::spawn_blocking(move || auto_backup::ouvrir_la_sauvegarde(&chemin))
        .await
        .map_err(|e| AppError::Internal(format!("ouverture de la sauvegarde : {e}")))?
        .map_err(|e| AppError::Internal(format!("ouverture de la sauvegarde : {e}")))?
        .map(tokio::fs::File::from_std)
        .ok_or_else(introuvable)?;
    emit_audit(
        &state,
        NewAuditLogEntry::from_current_user(
            &current_user,
            "admin.backup_downloaded",
            "installation",
            AUDIT_ENTITY_ID_NONE,
            Some(serde_json::json!({
                "name": sauvegarde.name,
                "kind": kind.as_str(),
                "size": sauvegarde.size,
            })),
        ),
    )
    .await;
    let disposition = crate::util::build_content_disposition(&sauvegarde.name, "fr-CH")?;
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_DISPOSITION, disposition)
        .header(header::CONTENT_LENGTH, sauvegarde.size)
        .body(Body::from_stream(tokio_util::io::ReaderStream::new(
            fichier,
        )))
        .map_err(|e| AppError::Internal(format!("response build: {e}")))
        .map(IntoResponse::into_response)
}

/// Audit best-effort d'un geste de l'administrateur (un échec d'écriture est
/// journalisé et n'empêche pas la réponse). L'entrée est construite au site
/// d'appel, l'action écrite en toutes lettres (garde `audit_label_registry`).
async fn emit_audit(state: &AppState, entree: NewAuditLogEntry) {
    let action = entree.action.clone();
    let resultat = async {
        let mut tx = state
            .pool
            .begin()
            .await
            .map_err(kesh_db::errors::map_db_error)?;
        kesh_db::repositories::audit_log::insert_in_tx(&mut tx, entree).await?;
        tx.commit().await.map_err(kesh_db::errors::map_db_error)?;
        Ok::<(), kesh_db::errors::DbError>(())
    }
    .await;
    if let Err(e) = resultat {
        tracing::warn!(error = ?e, action, "audit non écrit — non bloquant");
    }
}
