//! Story 17-3a — Endpoints d'administration d'installation.
//!
//! `GET /api/v1/admin/full-export` — export complet `.keshbackup` (Admin strict
//! + interdit aux PAT). La sous-story 17-3c ajoutera `POST .../full-import`.

use axum::{
    Extension,
    body::Body,
    extract::State,
    http::{StatusCode, header},
    response::Response,
};
use chrono::Utc;

use kesh_db::entities::AUDIT_ENTITY_ID_NONE;
use kesh_db::entities::audit_log::NewAuditLogEntry;

use crate::AppState;
use crate::admin_backup::export::build_keshbackup;
use crate::audit::AuditActor;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;
use crate::routes::api_keys::ensure_not_pat;

/// GET /api/v1/admin/full-export — export complet d'installation.
///
/// Monté dans `admin_routes` (RBAC `require_admin_role` appliqué par le
/// sub-router). Interdit aux clés PAT (AC2) : le backup contient des secrets
/// (hash de mots de passe, refresh tokens) qui ne doivent jamais transiter par
/// une intégration API.
pub async fn full_export(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Response, AppError> {
    // AC2 — anti-PAT (opération d'infra interdite aux clés API).
    ensure_not_pat(&current_user)?;

    let (bytes, meta) = build_keshbackup(&state.pool).await?;

    // Audit best-effort (handler-level, pas dans build_keshbackup — réutilisé
    // sans audit par le backup pré-import 17-3c). Un échec d'INSERT n'empêche
    // pas le téléchargement.
    emit_full_export_audit(&state, &current_user, &meta).await;

    let filename = format!(
        "kesh-installation-{}.keshbackup",
        Utc::now().date_naive().format("%Y-%m-%d")
    );
    let content_disposition = crate::util::build_content_disposition(&filename, "fr-CH")?;

    // DC8 — in-memory sous plafond, au-delà spill fichier temporaire + stream.
    let threshold = (state.config.admin_export_inmem_mib as usize) * 1024 * 1024;
    let body = if bytes.len() > threshold {
        stream_via_tempfile(bytes).await?
    } else {
        Body::from(bytes)
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_DISPOSITION, content_disposition)
        .body(body)
        .map_err(|e| AppError::Internal(format!("response build: {e}")))
}

/// Écrit le `.keshbackup` dans un fichier temporaire puis le streame.
///
/// Le fichier est unlink immédiatement après ouverture en lecture (Unix : le
/// descripteur reste valide, le fichier est auto-nettoyé à la fin du stream).
async fn stream_via_tempfile(bytes: Vec<u8>) -> Result<Body, AppError> {
    use std::sync::atomic::{AtomicU64, Ordering};
    use tokio::io::AsyncWriteExt;
    use tokio_util::io::ReaderStream;

    // Unicité du nom même pour des exports concurrents (même PID, même
    // horodatage) : compteur atomique process-global (review 17-3a Pass 1).
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "kesh-export-{}-{}-{}.keshbackup.tmp",
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or(0),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ));

    let mut f = tokio::fs::File::create(&path)
        .await
        .map_err(|e| AppError::AdminFullExportFailed(format!("temp create: {e}")))?;
    f.write_all(&bytes)
        .await
        .map_err(|e| AppError::AdminFullExportFailed(format!("temp write: {e}")))?;
    f.flush()
        .await
        .map_err(|e| AppError::AdminFullExportFailed(format!("temp flush: {e}")))?;
    drop(f);

    let file = match tokio::fs::File::open(&path).await {
        Ok(file) => file,
        Err(e) => {
            // Nettoyage si l'ouverture échoue (sinon fuite du fichier temp).
            let _ = tokio::fs::remove_file(&path).await;
            return Err(AppError::AdminFullExportFailed(format!("temp open: {e}")));
        }
    };
    // Unlink — sous Unix le fd ouvert garde le contenu accessible (auto-clean
    // à la fin du stream). Échec loggé (FS read-only, non-Unix) plutôt qu'avalé.
    if let Err(e) = tokio::fs::remove_file(&path).await {
        tracing::warn!(error = ?e, path = ?path, "tempfile unlink failed (non-blocking)");
    }

    Ok(Body::from_stream(ReaderStream::new(file)))
}

/// Émet `audit_log` `action='admin.full_export'`, `entity_type='installation'`
/// (best-effort, snake_case `details_json` pour les JSON paths SQL).
async fn emit_full_export_audit(
    state: &AppState,
    current_user: &CurrentUser,
    meta: &crate::admin_backup::export::KeshBackupMeta,
) {
    let result = async {
        let mut tx = state
            .pool
            .begin()
            .await
            .map_err(kesh_db::errors::map_db_error)?;
        kesh_db::repositories::audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::from_current_user(
                current_user,
                "admin.full_export",
                "installation",
                AUDIT_ENTITY_ID_NONE,
                Some(serde_json::json!({
                    "file_size": meta.byte_size,
                    "table_count": meta.table_count,
                    "total_rows": meta.total_rows,
                    "kesh_version": env!("CARGO_PKG_VERSION"),
                })),
            ),
        )
        .await?;
        tx.commit().await.map_err(kesh_db::errors::map_db_error)?;
        Ok::<(), kesh_db::errors::DbError>(())
    }
    .await;

    if let Err(e) = result {
        tracing::warn!(
            error = ?e,
            user_id = current_user.user_id,
            "audit insert failed (admin.full_export) — non-blocking"
        );
    }
}
