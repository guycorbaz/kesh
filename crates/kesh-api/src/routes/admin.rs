//! Story 17-3a — Endpoints d'administration d'installation.
//!
//! `GET /api/v1/admin/full-export` — export complet `.keshbackup` (Admin strict
//! + interdit aux PAT). La sous-story 17-3c ajoutera `POST .../full-import`.

use axum::{
    Extension, Json,
    body::Body,
    extract::{Multipart, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::Utc;

use kesh_db::entities::AUDIT_ENTITY_ID_NONE;
use kesh_db::entities::audit_log::NewAuditLogEntry;

use crate::AppState;
use crate::admin_backup::export::build_keshbackup;
use crate::admin_backup::import::{ParsedBackup, check_schema_compat, parse_and_verify};
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

/// POST /api/v1/admin/full-import — import complet d'installation (.keshbackup).
///
/// Monté dans `admin_routes` (RBAC `require_admin_role`) avec un
/// `DefaultBodyLimit` propre (`KESH_ADMIN_IMPORT_MAX_MB`). **Interdit aux clés
/// PAT** (AC2) : opération d'infra destructrice, jamais via intégration API.
///
/// Séquence (DC4/DC5/DC6) :
/// 1. anti-PAT + lecture multipart (`file`) ;
/// 2. **validation avant tout DELETE** : structure ZIP + intégrité SHA-256
///    ([`parse_and_verify`]), compat version 409 ([`check_import_version_compat`]),
///    compat colonnes bidirectionnelle 400 ([`check_schema_compat`]) ;
/// 3. **verrou d'installation** (`GET_LOCK`) sérialisant backup + restore ;
/// 4. **backup automatique pré-import** (cœur `build_keshbackup` sans audit) →
///    disque (`KESH_ADMIN_BACKUP_DIR`) — jamais d'import sans backup réussi ;
/// 5. **restore transactionnel** (`DELETE`+`INSERT`, FK_CHECKS=0) + audit
///    in-tx (`user_id = MIN(admin)` source, O-1) + DC11 onboarding + COMMIT.
///
/// ⚠️ Le `.keshbackup` est un **secret** (hash de mots de passe, refresh
/// tokens). L'import remplace les `refresh_tokens` ⇒ sessions destination
/// invalidées (`sessionInvalidated: true`).
pub async fn full_import(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    multipart: Multipart,
) -> Result<Response, AppError> {
    // AC2 — anti-PAT.
    ensure_not_pat(&current_user)?;

    // 1. Lire l'upload multipart (champ `file`).
    let bytes = read_upload(multipart).await?;

    // 2a. Structure + intégrité SHA-256 (avant tout DELETE).
    let parsed = parse_and_verify(&bytes)?;

    // 2b. Compat version (DC4) : 409 si le backup exige plus récent que nous.
    match kesh_db::version::check_import_version_compat(
        &parsed.manifest.kesh_version_min_required,
        env!("CARGO_PKG_VERSION"),
    ) {
        Ok(()) => {}
        Err(kesh_db::version::VersionError::DowngradeRefused { .. }) => {
            return Err(AppError::ImportVersionIncompatible {
                source_min_required: parsed.manifest.kesh_version_min_required.clone(),
                binary_version: env!("CARGO_PKG_VERSION").to_string(),
            });
        }
        Err(e) => {
            // SemVer illisible dans le manifeste → backup corrompu (400).
            return Err(AppError::InvalidBackupStructure(format!(
                "version du manifeste illisible : {e}"
            )));
        }
    }

    // 2c. Compat colonnes bidirectionnelle (AC12c) → 400 IMPORT_SCHEMA_MISMATCH.
    check_schema_compat(&state.pool, &parsed).await?;

    // 3. Verrou d'installation : sérialise les imports concurrents, tenu sur
    //    tout le backup + restore (relâché à la sortie quel que soit le chemin).
    let mut lock_conn = state.pool.acquire().await.map_err(|e| {
        AppError::AdminFullImportFailed(format!("acquisition connexion verrou : {e}"))
    })?;
    let got: Option<i64> = sqlx::query_scalar("SELECT GET_LOCK('kesh_full_import', 10)")
        .fetch_one(&mut *lock_conn)
        .await
        .map_err(|e| AppError::AdminFullImportFailed(format!("GET_LOCK : {e}")))?;
    if got != Some(1) {
        return Err(AppError::AdminFullImportFailed(
            "un autre import est déjà en cours".into(),
        ));
    }

    // 4 + 5. Backup pré-import + restore transactionnel.
    let outcome = run_backup_and_restore(&state, &parsed, &current_user).await;

    // Relâcher le verrou dans tous les cas (puis fermer la connexion).
    let _ = sqlx::query("DO RELEASE_LOCK('kesh_full_import')")
        .execute(&mut *lock_conn)
        .await;
    drop(lock_conn);

    let (backup_created, tables_restored, rows_restored) = outcome?;

    let body = serde_json::json!({
        "backupCreated": backup_created,
        "tablesRestored": tables_restored,
        "rowsRestored": rows_restored,
        "sourceVersion": parsed.manifest.kesh_version,
        "sessionInvalidated": true,
    });
    Ok((StatusCode::OK, Json(body)).into_response())
}

/// Lit le champ multipart `file` (le `.keshbackup`). Rejette un champ dupliqué
/// ou absent. Les autres champs sont ignorés.
async fn read_upload(mut multipart: Multipart) -> Result<Vec<u8>, AppError> {
    let mut file_bytes: Option<Vec<u8>> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("multipart illisible : {e}")))?
    {
        if field.name() == Some("file") {
            if file_bytes.is_some() {
                return Err(AppError::Validation(
                    "Champ 'file' dupliqué dans le multipart".into(),
                ));
            }
            let b = field
                .bytes()
                .await
                .map_err(|e| AppError::Validation(format!("lecture du fichier : {e}")))?;
            file_bytes = Some(b.to_vec());
        }
    }
    file_bytes.ok_or_else(|| AppError::Validation("Champ 'file' manquant dans le multipart".into()))
}

/// Backup pré-import (sans audit) + restore transactionnel + audit in-tx.
/// Retourne `(backup_created, tables_restored, rows_restored)`.
async fn run_backup_and_restore(
    state: &AppState,
    parsed: &ParsedBackup,
    current_user: &CurrentUser,
) -> Result<(bool, usize, usize), AppError> {
    // 4. Backup automatique de l'état courant (cœur sans audit, O-4) → disque.
    //    Jamais d'import sans backup réussi (DC5).
    let (backup_bytes, _meta) = build_keshbackup(&state.pool).await?;
    let backup_created =
        write_pre_import_backup(&state.config.admin_backup_dir, &backup_bytes).await?;

    // 5. Restore transactionnel (DELETE+INSERT, FK_CHECKS gérés dans kesh-db).
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::AdminFullImportFailed(format!("ouverture transaction : {e}")))?;

    let (tables_restored, rows_restored) =
        kesh_db::backup::restore_tables_in_tx(&mut tx, &parsed.tables)
            .await
            .map_err(|e| AppError::AdminFullImportFailed(format!("restore : {e}")))?;

    // Garde de cohérence : autant de lignes insérées que lues du backup, en
    // **excluant `onboarding_state`** (non restaurée, DC11). Un écart signale
    // qu'un INSERT a silencieusement perdu des lignes → rollback diagnostique.
    let onboarding_rows = parsed
        .tables
        .get("onboarding_state")
        .map(|t| t.rows.len())
        .unwrap_or(0);
    let expected_rows = parsed.total_rows - onboarding_rows;
    if rows_restored != expected_rows {
        return Err(AppError::AdminFullImportFailed(format!(
            "incohérence restore : {rows_restored} lignes insérées, {expected_rows} attendues"
        )));
    }

    // Audit in-tx, user_id = MIN(admin) **du dataset restauré** (O-1, FK
    // audit_log.user_id → users). PAS current_user (peut ne pas exister dans
    // la source).
    let min_admin: Option<i64> =
        sqlx::query_scalar("SELECT MIN(id) FROM users WHERE role = 'Admin'")
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| AppError::AdminFullImportFailed(format!("lecture admin source : {e}")))?;
    let audit_uid = min_admin.ok_or_else(|| {
        AppError::AdminFullImportFailed(
            "le backup source ne contient aucun compte Admin — import refusé".into(),
        )
    })?;

    let details = serde_json::json!({
        "source_kesh_version": parsed.manifest.kesh_version,
        "source_instance_id": parsed.manifest.instance_id,
        "triggered_by_user": current_user.user_id,
        "tables_restored": tables_restored,
        "rows_restored": rows_restored,
    });
    kesh_db::repositories::audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            audit_uid,
            "admin.full_import",
            "installation",
            AUDIT_ENTITY_ID_NONE,
            Some(details),
        ),
    )
    .await
    .map_err(|e| AppError::AdminFullImportFailed(format!("audit import : {e}")))?;

    // DC11 — forcer onboarding « done » si dataset onboardable (anti catch-22).
    kesh_db::backup::force_onboarding_done_if_eligible(&mut tx)
        .await
        .map_err(|e| AppError::AdminFullImportFailed(format!("onboarding post-restore : {e}")))?;

    tx.commit()
        .await
        .map_err(|e| AppError::AdminFullImportFailed(format!("commit : {e}")))?;

    Ok((backup_created, tables_restored, rows_restored))
}

/// Écrit le backup pré-import sur disque (filet de sécurité rollback). Le
/// chemin est **loggé serveur uniquement** (jamais exposé en réponse — l'admin
/// n'a pas d'accès disque sans SSH). Échec d'écriture → 500 (jamais d'import
/// sans backup réussi).
async fn write_pre_import_backup(dir: &str, bytes: &[u8]) -> Result<bool, AppError> {
    tokio::fs::create_dir_all(dir).await.map_err(|e| {
        AppError::AdminFullImportFailed(format!("création répertoire backup '{dir}' : {e}"))
    })?;
    let path = std::path::Path::new(dir).join(format!(
        "kesh-pre-import-{}.keshbackup",
        Utc::now().format("%Y%m%dT%H%M%S%3f")
    ));
    tokio::fs::write(&path, bytes).await.map_err(|e| {
        AppError::AdminFullImportFailed(format!("écriture backup '{}' : {e}", path.display()))
    })?;
    tracing::info!(
        path = %path.display(),
        bytes = bytes.len(),
        "backup pré-import écrit (filet de sécurité avant restore)"
    );
    Ok(true)
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
