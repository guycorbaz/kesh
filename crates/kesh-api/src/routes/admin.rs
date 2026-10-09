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
/// 3. ouverture de la transaction + **verrou d'installation**
///    (`_kesh_version id=1 FOR UPDATE`, pattern 17-1) sérialisant backup +
///    restore (acquis **avant** le backup, AC13) ;
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
    //    Antérieur à la sauvegarde : un échec de lecture du schéma ne promet
    //    aucune copie (Story 15-13b, #576 — `avant_sauvegarde`).
    check_schema_compat(&state.pool, &parsed)
        .await
        .map_err(avant_sauvegarde)?;

    // 3 + 4 + 5. Backup pré-import + restore transactionnel. La sérialisation
    //    des imports destructeurs concurrents est assurée par un verrou
    //    `_kesh_version id=1 FOR UPDATE` pris **en tête de la transaction de
    //    restore** (cf. `run_backup_and_restore`) — pattern Story 17-1,
    //    auto-relâché au COMMIT/rollback (robuste aux panics, une seule
    //    connexion ; review Pass 1 : remplace un `GET_LOCK` qui fuyait sur panic
    //    et mobilisait une 2e connexion du pool).
    let (backup_created, tables_restored, rows_restored) =
        run_backup_and_restore(&state, &parsed, &current_user).await?;

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
    // Ouvre la transaction de restore et acquiert **d'abord** le verrou
    // d'installation (pattern Story 17-1), AVANT le backup pré-import (AC13 :
    // « acquérir d'abord un verrou… tenu sur toute la durée backup+restore »).
    // Un 2e import bloque ici jusqu'au COMMIT/rollback du 1er. `_kesh_version`
    // n'est jamais supprimée (table système), la row id=1 existe toujours
    // (migration). Auto-relâché avec la transaction (robuste aux panics).
    let mut tx = state.pool.begin().await.map_err(|e| {
        AppError::AdminPreImportBackupFailed(format!("ouverture transaction : {e}"))
    })?;
    sqlx::query("SELECT id FROM _kesh_version WHERE id = 1 FOR UPDATE")
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppError::AdminPreImportBackupFailed(format!("verrou installation : {e}")))?
        .ok_or_else(|| {
            // Row id=1 absente = installation incohérente (le boot l'aurait
            // refusée via check_downgrade_protection::RowMissing). Sans elle, le
            // FOR UPDATE ne verrouille rien → refuser l'import plutôt que de
            // risquer une course entre imports concurrents (review Pass 2).
            AppError::AdminPreImportBackupFailed(
                "verrou installation : row _kesh_version id=1 absente (installation incohérente)"
                    .into(),
            )
        })?;

    // 4. Backup automatique de l'état courant (cœur sans audit, O-4) → disque.
    //    Jamais d'import sans backup réussi (DC5). Pris **sous le verrou**
    //    (review Pass 4) : aucun autre import ne peut committer pendant cette
    //    lecture (ils bloquent au FOR UPDATE ci-dessus) ⇒ pas de torn-read
    //    inter-imports. `build_keshbackup` lit via le pool (connexions
    //    séparées) ; le SELECT non-bloquant sur `_kesh_version` ne deadlocke
    //    pas avec le FOR UPDATE détenu par cette transaction. Tout échec
    //    jusqu'à l'écriture réussie du fichier est `AdminPreImportBackupFailed`
    //    (Story 15-13b, #576) : aucune sauvegarde n'existe encore.
    let (backup_bytes, _meta) = build_keshbackup(&state.pool)
        .await
        .map_err(avant_sauvegarde)?;
    let backup_created =
        write_pre_import_backup(&state.config.admin_backup_dir, &backup_bytes).await?;

    // 4-bis (Story 24-4c, #380) : RELEVER LA BORNE DU VERROU DE PÉRIODE
    // AVANT LE RESTORE — c'est le seul instant où elle est encore lisible.
    //
    // ⛔ `companies` figure dans `TABLES_TO_TRUNCATE` : l'étape 5 la VIDE puis
    // la ré-insère depuis l'archive, et `books_locked_through` y voyage tout
    // seul. Après le restore, l'ancienne valeur n'existe **nulle part** — la
    // lire à l'endroit où l'entrée d'audit s'écrit rapporterait la NOUVELLE
    // valeur comme étant l'ancienne, c'est-à-dire une trace qui ne trace rien.
    //
    // ⚠️ Restaurer une archive antérieure à la pose du verrou fait donc reculer
    // la borne. **Ce n'est pas un défaut à interdire** : un `.keshbackup`
    // restaure l'installation ENTIÈRE, et il est cohérent que le verrou suive
    // les écritures — refuser produirait un verrou décorrélé des livres. Ce qui
    // serait grave, c'est que la restauration devienne un DÉVERROUILLAGE
    // SILENCIEUX. D'où la trace ci-dessous.
    let books_lock_before: Option<chrono::NaiveDate> =
        sqlx::query_scalar("SELECT books_locked_through FROM companies ORDER BY id LIMIT 1")
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| {
                AppError::AdminFullImportFailed(format!("lecture du verrou de période : {e}"))
            })?
            .flatten();

    // 5. Restore transactionnel (DELETE+INSERT, FK_CHECKS gérés dans kesh-db).
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

    // 5-bis. Rejeu des backfills de données (Story 16-1c, #281). **Après** la
    //    garde de comptage — inutile de rejouer par-dessus un restore déjà
    //    incohérent, qui va être annulé — et **avant** l'audit, dont le détail
    //    porte le rapport. Tourne avec `FOREIGN_KEY_CHECKS = 1` :
    //    `restore_tables_in_tx` rétablit systématiquement le flag avant de rendre
    //    la main, y compris sur erreur.
    //
    //    Sans ce rejeu, restaurer un backup antérieur à une migration de backfill
    //    rouvrait DÉFINITIVEMENT le bug qu'elle fermait : `_sqlx_migrations`
    //    n'étant pas restaurée, la migration reste marquée appliquée.
    let backfills_replayed =
        kesh_db::post_restore::replay_post_restore_backfills(&mut tx, &parsed.tables)
            .await
            .map_err(|e| AppError::AdminFullImportFailed(format!("rejeu des backfills : {e}")))?;

    // 5-ter. Backfill Rust de la canonique du numéro de client (Story 22-1,
    //    D6) — le pendant du 5-bis pour le SEUL backfill qui ne peut pas être
    //    du SQL (NFKC + invisibles). Un backup antérieur à la story arrive
    //    avec `client_number_canonical` vide ; un backup en COLLISION est
    //    refusé ici en 400, rapport nominatif à l'appui — un backup en
    //    collision ne s'installe pas, il se répare d'abord. Idempotent : sur
    //    un backup à jour, une requête, zéro écriture.
    kesh_db::backfill::backfill_client_number_canonical(&mut tx)
        .await
        .map_err(|e| match e {
            kesh_db::backfill::BackfillError::Refused(report) => {
                // 400 nominatif : collisions OU canoniques trop longues sont un
                // problème de DONNÉES du backup, réparable par l'exploitant —
                // pas une panne du serveur.
                AppError::ImportClientNumberCollision {
                    report: report.to_string(),
                }
            }
            kesh_db::backfill::BackfillError::Db(db) => {
                AppError::AdminFullImportFailed(format!("backfill client_number_canonical : {db}"))
            }
        })?;

    // Audit in-tx, user_id = MIN(admin) **du dataset restauré** (O-1). PAS
    // current_user (peut ne pas exister dans la source).
    //
    // ⚠️ Le motif était la FK `audit_log.user_id → users`, RETIRÉE par la Story
    // 25-1a (#376) — `user_id` est un pointeur logique depuis que la piste
    // survit au remplacement de `users`. Le choix reste néanmoins le bon :
    // attribuer l'import à un compte absent de l'installation restaurée
    // produirait une entrée que personne ne peut relier à quiconque.
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

    // Story 24-4c (#380) : la borne a-t-elle RECULÉ ? Si oui, la restauration
    // vaut déverrouillage et doit se voir.
    //
    // ⛔ L'action est `books.restored`, PAS `books.unlocked`. Ce dernier a **un
    // seul producteur** — le déverrouillage délibéré par un Admin — et l'entrée
    // écrite ici serait signée par `MIN(id)` des admins **du dataset restauré**,
    // un administrateur qui n'a rien déverrouillé (il n'existe aucun acteur
    // « système » : `ActorType` ne connaît que `User` et `ApiKey`). Les
    // confondre rendrait le filtre d'audit inutilisable pour le réviseur qui
    // cherche QUI a déverrouillé.
    let books_lock_after: Option<chrono::NaiveDate> =
        sqlx::query_scalar("SELECT books_locked_through FROM companies ORDER BY id LIMIT 1")
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| {
                AppError::AdminFullImportFailed(format!("relecture du verrou de période : {e}"))
            })?
            .flatten();

    // ⚠️ « Reculer » couvre AUSSI le passage à `NULL` (verrou retiré), qui est
    // le recul maximal. `None > Some(_)` étant faux pour `Option<NaiveDate>`,
    // la comparaison se fait à la main plutôt que par `<`.
    let a_recule = match (books_lock_before, books_lock_after) {
        (Some(_), None) => true,
        (Some(avant), Some(apres)) => apres < avant,
        (None, _) => false,
    };
    if a_recule {
        kesh_db::repositories::audit_log::insert_in_tx(
            &mut tx,
            kesh_db::entities::NewAuditLogEntry::user(
                audit_uid,
                "books.restored".to_string(),
                "company".to_string(),
                0,
                Some(serde_json::json!({
                    "before": books_lock_before,
                    "after": books_lock_after,
                    "motif": "restauration de sauvegarde",
                    "triggered_by_user": current_user.user_id,
                })),
            ),
        )
        .await
        .map_err(|e| {
            AppError::AdminFullImportFailed(format!("audit du verrou de période : {e}"))
        })?;
    }

    let details = serde_json::json!({
        "source_kesh_version": parsed.manifest.kesh_version,
        "source_instance_id": parsed.manifest.instance_id,
        "triggered_by_user": current_user.user_id,
        "tables_restored": tables_restored,
        "rows_restored": rows_restored,
        // Story 16-1c — rapport de rejeu. Le corps de réponse HTTP reste
        // INCHANGÉ (D-C7) : cette information de diagnostic vit dans l'audit et
        // le journal serveur, que l'exploitant d'un restore consulte de toute
        // façon depuis la machine.
        //
        // `outcome` porte le CODE STABLE de `ReplayOutcome::code()`, et non un
        // `format!("{:?}")` de l'enum : l'audit est une archive relue longtemps
        // après, un renommage de variant y ferait dériver silencieusement le
        // contenu d'enregistrements déjà écrits.
        "backfills_replayed": backfills_replayed
            .iter()
            .map(|r| {
                serde_json::json!({
                    "version": r.version,
                    "label": r.label,
                    "outcome": r.outcome.code(),
                    "missing_sentinels": r.outcome.missing_sentinels(),
                    "rows_affected": r.rows_affected,
                })
            })
            .collect::<Vec<_>>(),
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

/// Story 15-13b (#576) — convertit, **à l'appel**, un échec né hors de ce
/// module **avant** que la sauvegarde pré-import soit sur disque
/// (`check_schema_compat`, `build_keshbackup`) : `AdminFullImportFailed` et
/// `AdminFullExportFailed` deviennent `AdminPreImportBackupFailed`, détail
/// conservé ; toute autre variante est rendue telle quelle
/// (`ImportSchemaMismatch` reste un 400). Forme d'appel prescrite :
/// `….await.map_err(avant_sauvegarde)?`, gardée lexicalement par le test
/// `avant_sauvegarde_branchee_aux_appels_de_l_import`.
fn avant_sauvegarde(err: AppError) -> AppError {
    match err {
        AppError::AdminFullImportFailed(detail) | AppError::AdminFullExportFailed(detail) => {
            AppError::AdminPreImportBackupFailed(detail)
        }
        autre => autre,
    }
}

/// Écrit la sauvegarde de sécurité pré-import sur disque (filet avant le
/// restore destructeur). Le chemin est **journalisé côté serveur uniquement**
/// (jamais exposé en réponse). Story 15-13b (#552) : le dossier, quand Kesh le
/// crée, l'est en `0700` — **chaque niveau créé**, parents compris
/// (`DirBuilder` récursif) ; un dossier existant (montage de l'hôte) n'est pas
/// modifié. Le fichier est écrit par [`write_backup_file`] : mode `0600`,
/// sous `<nom>.partial` puis renommé, jamais par-dessus un fichier existant.
/// Tout échec rend `AdminPreImportBackupFailed` : aucun import sans
/// sauvegarde réussie (DC5).
async fn write_pre_import_backup(dir: &str, bytes: &[u8]) -> Result<bool, AppError> {
    let mut builder = tokio::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    builder.mode(0o700);
    builder.create(dir).await.map_err(|e| {
        AppError::AdminPreImportBackupFailed(format!("création répertoire backup '{dir}' : {e}"))
    })?;
    // Nom unique même pour deux sauvegardes dans la même milliseconde : pid +
    // compteur atomique process-global, comme `stream_via_tempfile` (review
    // Pass 2). (La sauvegarde est prise SOUS le verrou `FOR UPDATE` de
    // `run_backup_and_restore` — un ancien commentaire disait « avant ».)
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let path = std::path::Path::new(dir).join(format!(
        "kesh-pre-import-{}-{}-{}.keshbackup",
        Utc::now().format("%Y%m%dT%H%M%S%3f"),
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    write_backup_file(&path, bytes).await?;
    tracing::info!(
        path = %path.display(),
        bytes = bytes.len(),
        "backup pré-import écrit (filet de sécurité avant restore)"
    );
    Ok(true)
}

/// Story 15-13b (#552) — écrit `bytes` sous `path` sans jamais exposer un
/// fichier incomplet sous ce nom ni écraser un fichier existant :
///
/// 1. crée `<path>.partial` (même dossier, donc même système de fichiers) en
///    `create_new` et, sous Unix, mode `0600` — un `.partial` déjà présent
///    fait échouer l'appel **sans rien supprimer** (on ne supprime que ce
///    qu'on a créé) ;
/// 2. écrit, puis `sync_all` ;
/// 3. vérifie que `path` n'existe pas (`try_exists` en erreur vaut échec,
///    jamais « absent ») — `rename` remplacerait une cible existante ;
/// 4. renomme `<path>.partial` en `path` (atomique sur un même système de
///    fichiers).
///
/// Si 2, 3 ou 4 échouent, le `.partial` créé est supprimé au mieux (un échec
/// de suppression est journalisé en `warn!` avec son chemin, sans masquer
/// l'erreur d'origine). Toute erreur : `AdminPreImportBackupFailed`, avec un
/// détail par étape qui nomme le chemin (journal seulement).
///
/// Angles morts écrits (non traités) :
/// - la fenêtre entre la vérification 3 et le renommage 4 n'est pas fermée
///   (le nom est unique par construction) ;
/// - **annulation** : si le futur est abandonné pendant 2 à 4 (client qui
///   coupe la connexion), ni la suppression du `.partial` ni le `warn!`
///   n'ont lieu — le `.partial` (0600) reste, sans jamais porter le nom
///   d'une sauvegarde ; le manuel dit qu'il peut être supprimé ;
/// - **durabilité** : le dossier n'est pas synchronisé après le renommage —
///   après une coupure de courant, l'entrée du dossier peut manquer alors
///   que le contenu était synchronisé ;
/// - `try_exists` **suit** les liens symboliques : un lien pendant au nom
///   final est vu absent, et `rename` remplace le lien (pas sa cible) ; sans
///   portée (nom unique, dossier `0700` créé par Kesh).
async fn write_backup_file(path: &std::path::Path, bytes: &[u8]) -> Result<(), AppError> {
    use tokio::io::AsyncWriteExt;

    let mut partial_os = path.as_os_str().to_owned();
    partial_os.push(".partial");
    let partial = std::path::PathBuf::from(partial_os);

    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options.open(&partial).await.map_err(|e| {
        AppError::AdminPreImportBackupFailed(format!(
            "création fichier partiel '{}' : {e}",
            partial.display()
        ))
    })?;

    let resultat: Result<(), AppError> = async {
        file.write_all(bytes).await.map_err(|e| {
            AppError::AdminPreImportBackupFailed(format!(
                "écriture backup '{}' : {e}",
                partial.display()
            ))
        })?;
        file.sync_all().await.map_err(|e| {
            AppError::AdminPreImportBackupFailed(format!(
                "synchronisation backup '{}' : {e}",
                partial.display()
            ))
        })?;
        drop(file);
        match tokio::fs::try_exists(path).await {
            Ok(false) => {}
            Ok(true) => {
                return Err(AppError::AdminPreImportBackupFailed(format!(
                    "nom final déjà présent '{}'",
                    path.display()
                )));
            }
            Err(e) => {
                return Err(AppError::AdminPreImportBackupFailed(format!(
                    "vérification nom final '{}' : {e}",
                    path.display()
                )));
            }
        }
        tokio::fs::rename(&partial, path).await.map_err(|e| {
            AppError::AdminPreImportBackupFailed(format!(
                "renommage backup '{}' → '{}' : {e}",
                partial.display(),
                path.display()
            ))
        })
    }
    .await;

    if resultat.is_err()
        && let Err(e) = tokio::fs::remove_file(&partial).await
    {
        tracing::warn!(
            path = %partial.display(),
            "fichier partiel non supprimé après un échec d'écriture de la sauvegarde \
             pré-import — ce n'est PAS une sauvegarde valide, il peut être supprimé : {e}"
        );
    }
    resultat
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

#[cfg(test)]
mod tests {
    //! Story 15-13b (#552, #576) — sauvegarde pré-import : mode `0600`,
    //! écriture par `.partial`, refus d'écraser, conversion des échecs
    //! antérieurs à la sauvegarde, et branchement de cette conversion.

    use super::*;

    /// Test 11 — `write_pre_import_backup` sur un sous-dossier absent : dossier
    /// créé en `0700`, fichier en `0600`, contenu identique, aucun `.partial`
    /// restant. Assertions de montage : un fichier et un dossier témoins créés
    /// par `std::fs` ne sont PAS en `0600`/`0700` — sinon l'umask rend le test
    /// non discriminant.
    #[cfg(unix)]
    #[tokio::test]
    async fn write_pre_import_backup_en_0600() {
        use std::os::unix::fs::PermissionsExt;
        let racine = tempfile::tempdir().expect("tempdir");
        let dir = racine.path().join("sauvegardes");
        let ecrit = write_pre_import_backup(dir.to_str().unwrap(), b"contenu-de-test")
            .await
            .expect("écriture de la sauvegarde");
        assert!(ecrit);

        let mode = |p: &std::path::Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        let temoin_fichier = dir.join("temoin.txt");
        std::fs::write(&temoin_fichier, b"t").unwrap();
        let temoin_dossier = racine.path().join("temoin-dossier");
        std::fs::create_dir(&temoin_dossier).unwrap();
        assert_ne!(
            mode(&temoin_fichier),
            0o600,
            "umask de l'environnement trop restrictif (077 ?) : le test ne peut pas distinguer le mode posé par le code"
        );
        assert_ne!(
            mode(&temoin_dossier),
            0o700,
            "umask de l'environnement trop restrictif (077 ?) : le test ne peut pas distinguer le mode posé par le code"
        );

        assert_eq!(mode(&dir), 0o700, "dossier de sauvegarde créé par Kesh");
        let entrees: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| *p != temoin_fichier)
            .collect();
        assert_eq!(
            entrees.len(),
            1,
            "une seule sauvegarde attendue : {entrees:?}"
        );
        let sauvegarde = &entrees[0];
        let nom = sauvegarde.file_name().unwrap().to_str().unwrap();
        assert!(
            nom.starts_with("kesh-pre-import-") && nom.ends_with(".keshbackup"),
            "nom inattendu : {nom}"
        );
        assert_eq!(mode(sauvegarde), 0o600, "sauvegarde en 0600");
        assert_eq!(std::fs::read(sauvegarde).unwrap(), b"contenu-de-test");
    }

    /// Revue P1 (E3, A-4) — `write_pre_import_backup` sur deux niveaux absents :
    /// **chaque** niveau créé est en `0700`, le parent intermédiaire compris
    /// (`DirBuilder` récursif) ; et un dossier **existant** n'est pas modifié
    /// (`0755` reste `0755`, cas du `./backup` créé par Docker). Assertion de
    /// montage : un dossier témoin créé par `std::fs` n'est pas en `0700`.
    #[cfg(unix)]
    #[tokio::test]
    async fn write_pre_import_backup_dossiers_crees_et_existants() {
        use std::os::unix::fs::PermissionsExt;
        let mode = |p: &std::path::Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        let racine = tempfile::tempdir().expect("tempdir");
        let temoin = racine.path().join("temoin-dossier");
        std::fs::create_dir(&temoin).unwrap();
        assert_ne!(
            mode(&temoin),
            0o700,
            "umask de l'environnement trop restrictif (077 ?) : le test ne peut pas distinguer le mode posé par le code"
        );

        let parent = racine.path().join("a");
        let feuille = parent.join("b");
        assert!(
            write_pre_import_backup(feuille.to_str().unwrap(), b"x")
                .await
                .expect("écriture sous deux niveaux absents")
        );
        assert_eq!(mode(&parent), 0o700, "parent intermédiaire créé par Kesh");
        assert_eq!(mode(&feuille), 0o700, "dossier de sauvegarde créé par Kesh");

        let existant = racine.path().join("existant");
        std::fs::create_dir(&existant).unwrap();
        std::fs::set_permissions(&existant, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(
            write_pre_import_backup(existant.to_str().unwrap(), b"y")
                .await
                .expect("écriture dans un dossier existant")
        );
        assert_eq!(
            mode(&existant),
            0o755,
            "un dossier existant n'est pas modifié"
        );
    }

    /// Test 12 (a) — un nom final déjà occupé fait échouer l'écriture en
    /// `AdminPreImportBackupFailed` ; le fichier existant garde son contenu et
    /// aucun `.partial` ne reste.
    #[tokio::test]
    async fn write_backup_file_n_ecrase_pas() {
        let racine = tempfile::tempdir().expect("tempdir");
        let path = racine.path().join("kesh-pre-import-x.keshbackup");
        std::fs::write(&path, b"existant").unwrap();
        let err = write_backup_file(&path, b"nouveau")
            .await
            .expect_err("le nom final existe : échec attendu");
        assert!(
            matches!(err, AppError::AdminPreImportBackupFailed(_)),
            "variante inattendue : {err:?}"
        );
        assert_eq!(std::fs::read(&path).unwrap(), b"existant");
        assert!(
            !racine
                .path()
                .join("kesh-pre-import-x.keshbackup.partial")
                .exists(),
            "le .partial créé par l'appel doit être supprimé"
        );
    }

    /// Test 12 (b) — un `.partial` préexistant (écriture interrompue) fait
    /// échouer l'appel ; le nom final n'apparaît pas, et ce `.partial`, qui
    /// n'est pas celui de l'appel, garde son contenu (on ne supprime que ce
    /// qu'on a créé).
    #[tokio::test]
    async fn write_backup_file_passe_par_un_partiel() {
        let racine = tempfile::tempdir().expect("tempdir");
        let path = racine.path().join("kesh-pre-import-y.keshbackup");
        let partial = racine.path().join("kesh-pre-import-y.keshbackup.partial");
        std::fs::write(&partial, b"interrompu").unwrap();
        let err = write_backup_file(&path, b"complet")
            .await
            .expect_err("un .partial préexistant doit faire échouer l'appel");
        assert!(
            matches!(err, AppError::AdminPreImportBackupFailed(_)),
            "variante inattendue : {err:?}"
        );
        assert!(!path.exists(), "le nom final ne doit pas apparaître");
        assert_eq!(std::fs::read(&partial).unwrap(), b"interrompu");
    }

    /// Test 14 — `avant_sauvegarde` convertit les deux variantes nées avant la
    /// sauvegarde, détail conservé, et rend les autres telles quelles.
    #[test]
    fn avant_sauvegarde_convertit_les_echecs_anterieurs() {
        for err in [
            AppError::AdminFullImportFailed("d1".into()),
            AppError::AdminFullExportFailed("d1".into()),
        ] {
            match avant_sauvegarde(err) {
                AppError::AdminPreImportBackupFailed(d) => assert_eq!(d, "d1"),
                autre => panic!("conversion attendue, trouvé {autre:?}"),
            }
        }
        match avant_sauvegarde(AppError::ImportSchemaMismatch {
            table: "t".into(),
            unknown_columns: vec![],
            missing_required_columns: vec!["c".into()],
        }) {
            AppError::ImportSchemaMismatch { table, .. } => assert_eq!(table, "t"),
            autre => panic!("ImportSchemaMismatch doit rester un 400, trouvé {autre:?}"),
        }
        match avant_sauvegarde(AppError::InvalidBackupStructure("s".into())) {
            AppError::InvalidBackupStructure(d) => assert_eq!(d, "s"),
            autre => panic!("InvalidBackupStructure doit rester tel quel, trouvé {autre:?}"),
        }
    }

    /// Test 20 — garde LEXICALE (C-15-13-28) : `avant_sauvegarde` est appliquée
    /// aux appels de `check_schema_compat` et `build_keshbackup` de l'import
    /// (`full_import`, `run_backup_and_restore`), et PAS à celui de l'export
    /// (`full_export`). Lit ce fichier, tronqué à la première ligne
    /// `#[cfg(test)]`, commentaires `//` écartés ; une instruction court
    /// jusqu'au `;` suivant (rustfmt peut la couper). Assertion de montage :
    /// un appel de `check_schema_compat`, deux de `build_keshbackup`.
    #[test]
    fn avant_sauvegarde_branchee_aux_appels_de_l_import() {
        let code = code_de_production();
        let fonction_englobante = |pos: usize| -> String {
            code[..pos]
                .lines()
                .rev()
                .find_map(|l| {
                    l.strip_prefix("pub async fn ")
                        .or_else(|| l.strip_prefix("async fn "))
                        .map(|reste| reste.split('(').next().unwrap_or("").to_string())
                })
                .unwrap_or_default()
        };
        let mut appels: Vec<(&str, String, String)> = Vec::new();
        for ident in ["check_schema_compat", "build_keshbackup"] {
            let motif = format!("{ident}(");
            for (pos, _) in code.match_indices(&motif) {
                let avant = &code[..pos];
                if avant
                    .chars()
                    .last()
                    .is_some_and(|c| c.is_alphanumeric() || c == '_')
                    || avant.ends_with("fn ")
                {
                    continue;
                }
                let fin = code[pos..].find(';').map_or(code.len(), |i| pos + i);
                appels.push((ident, fonction_englobante(pos), code[pos..fin].to_string()));
            }
        }
        let compte = |ident: &str| appels.iter().filter(|(i, _, _)| *i == ident).count();
        assert_eq!(
            (compte("check_schema_compat"), compte("build_keshbackup")),
            (1, 2),
            "assertion de montage : un appel de check_schema_compat et deux de build_keshbackup attendus, trouvé {appels:#?}"
        );
        for (ident, fonction, instruction) in &appels {
            let convertie = instruction.contains(".map_err(avant_sauvegarde)");
            match fonction.as_str() {
                "full_import" | "run_backup_and_restore" => assert!(
                    convertie,
                    "{fonction} : l'appel de {ident} doit porter `.map_err(avant_sauvegarde)` : {instruction}"
                ),
                "full_export" => assert!(
                    !convertie,
                    "full_export : un échec d'export reste un échec d'export : {instruction}"
                ),
                autre => {
                    panic!("appel de {ident} dans une fonction non triée `{autre}` : {instruction}")
                }
            }
        }
    }
    /// Code de production de ce fichier (avant la première ligne
    /// `#[cfg(test)]`), lignes de commentaire `//` écartées — base des gardes
    /// lexicales ci-dessous et du test 20.
    fn code_de_production() -> String {
        include_str!("admin.rs")
            .lines()
            .take_while(|l| l.trim() != "#[cfg(test)]")
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Revue P1 (E5) — garde LEXICALE des conversions directes : dans
    /// `run_backup_and_restore`, tout ce qui précède l'appel de
    /// `write_pre_import_backup` (ouverture de transaction, verrou `FOR
    /// UPDATE`, ligne `_kesh_version` absente, `build_keshbackup`) ne construit
    /// ni `AdminFullImportFailed` ni `AdminFullExportFailed`, et chaque `?` y
    /// est couvert par une conversion : autant de `?` que de
    /// `AdminPreImportBackupFailed` construites plus de `.map_err(avant_sauvegarde)`.
    /// Assertion de montage : trois constructions directes, une conversion par
    /// `avant_sauvegarde`. Un nouveau site d'échec dans ce segment fait rougir
    /// le test : il est à trier, puis le décompte à mettre à jour.
    #[test]
    fn echecs_avant_sauvegarde_tous_convertis() {
        let code = code_de_production();
        let debut = code
            .find("async fn run_backup_and_restore(")
            .expect("run_backup_and_restore introuvable");
        let fin = debut
            + code[debut..]
                .find("write_pre_import_backup(")
                .expect("appel de write_pre_import_backup introuvable");
        let segment = &code[debut..fin];
        for interdit in ["AdminFullImportFailed", "AdminFullExportFailed"] {
            assert!(
                !segment.contains(interdit),
                "{interdit} construite avant la sauvegarde : le message promettrait une sauvegarde absente"
            );
        }
        let directes = segment.matches("AdminPreImportBackupFailed").count();
        let par_avant_sauvegarde = segment.matches(".map_err(avant_sauvegarde)").count();
        assert_eq!(
            (directes, par_avant_sauvegarde),
            (3, 1),
            "assertion de montage : trois constructions directes et une conversion attendues"
        );
        assert_eq!(
            segment.matches('?').count(),
            directes + par_avant_sauvegarde,
            "un `?` avant la sauvegarde n'est pas couvert par une conversion"
        );
    }

    /// Revue P1 (B1 = E2 = A-1) — aucune ligne de code de production de ce
    /// fichier ne porte deux espaces consécutives après son indentation. Le
    /// formatage n'en produit jamais hors commentaires : une telle suite ne
    /// peut venir que d'un littéral de chaîne dont la continuation `\` s'est
    /// perdue (l'indentation de la ligne suivante entre alors dans le texte
    /// journalisé). Le test ne dépend d'aucun message. Assertion de montage :
    /// le prédicat mord sur une ligne fabriquée.
    #[test]
    fn aucun_blanc_parasite_dans_le_code() {
        let parasite = |l: &str| l.trim_start().contains("  ");
        assert!(
            parasite("        \"de la sauvegarde              pré-import\""),
            "assertion de montage : le prédicat doit reconnaître une suite d'espaces"
        );
        let code = code_de_production();
        let fautives: Vec<&str> = code.lines().filter(|l| parasite(l)).collect();
        assert!(
            fautives.is_empty(),
            "suite d'espaces dans le code (continuation `\\` perdue ?) : {fautives:#?}"
        );
    }
}
