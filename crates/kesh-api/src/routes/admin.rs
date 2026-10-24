//! Story 17-3a — Endpoints d'administration d'installation.
//!
//! `GET /api/v1/admin/full-export` — export complet `.keshbackup` (Admin strict
//! + interdit aux PAT) ; `POST /api/v1/admin/full-import` — import (17-3c).
//!
//! Story 31-1a (#503) — format 2 : les pièces du dossier des documents voyagent
//! dans la sauvegarde. L'export n'assemble plus l'archive en mémoire (il
//! déborde dans `KESH_ADMIN_BACKUP_DIR`) ; l'import reçoit l'envoi sur disque,
//! écrit les pièces **avant** de remplacer la base, et s'exécute dans une
//! tâche que l'abandon du client n'interrompt pas.

use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};

use axum::{
    Extension, Json,
    body::Body,
    extract::{Multipart, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::Utc;

use kesh_db::backup::InstallationExport;
use kesh_db::entities::AUDIT_ENTITY_ID_NONE;
use kesh_db::entities::audit_log::NewAuditLogEntry;

use crate::AppState;
use crate::admin_backup::export::{BackupWriteError, build_keshbackup, write_keshbackup};
use crate::admin_backup::import::{
    ParsedBackup, ReadLimits, check_schema_compat, compter_les_pieces_manquantes,
    ecrire_les_pieces, parse_and_verify,
};
use crate::audit::AuditActor;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;
use crate::routes::api_keys::ensure_not_pat;

/// Nombre de pièces emportées par l'export (AC5).
const HEADER_FILES: &str = "x-kesh-backup-files";
/// Nombre de pièces référencées que l'export n'emporte pas (AC5).
const HEADER_FILES_NOT_INCLUDED: &str = "x-kesh-backup-files-not-included";
/// Posé quand l'archive dépasse `KESH_ADMIN_IMPORT_MAX_MB` (valeur : ce
/// plafond, en Mio) — AC12.
const HEADER_EXCEEDS_IMPORT_LIMIT: &str = "x-kesh-backup-exceeds-import-limit";

/// GET /api/v1/admin/full-export — export complet d'installation.
///
/// Monté dans `admin_routes` (RBAC `require_admin_role` appliqué par le
/// sub-router). Interdit aux clés PAT (AC2) : le backup contient des secrets
/// (hash de mots de passe, refresh tokens) qui ne doivent jamais transiter par
/// une intégration API.
///
/// Story 31-1a (AC4, AC5, AC12) : l'archive s'écrit dans un
/// `SpooledTempFile` de seuil `KESH_ADMIN_EXPORT_INMEM_MB` (en mémoire jusqu'au
/// seuil, sur disque au-delà), qui déborde **dans `KESH_ADMIN_BACKUP_DIR`**
/// (créé s'il manque) — monté, contrairement au `/tmp` du conteneur. La réponse
/// porte les comptes de pièces, et prévient si l'archive dépasse le plafond
/// d'import.
pub async fn full_export(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Response, AppError> {
    // AC2 — anti-PAT (opération d'infra interdite aux clés API).
    ensure_not_pat(&current_user)?;

    let backup_dir = state.config.admin_backup_dir.clone();
    ensure_backup_dir(&backup_dir)
        .await
        .map_err(AppError::AdminFullExportFailed)?;
    let seuil = (state.config.admin_export_inmem_mib as usize) * 1024 * 1024;
    // Story 31-1c (AC27) — une seule sauvegarde à la fois. Garde POSSÉDÉ, pris
    // avant la lecture de la base et DÉPLACÉ dans la fermeture : il vit dans
    // le `spawn_blocking` d'assemblage et n'est rendu qu'à la fin de
    // l'assemblage, quel que soit le sort du client (C-31-1-55) ; il est rendu
    // avant la diffusion (un téléchargement lent ne bloque rien).
    let garde = state.backup_lock.clone().lock_owned().await;
    let remplir = move |export: InstallationExport, documents: PathBuf| {
        let _garde = garde;
        let temporaire = tempfile::spooled_tempfile_in(seuil, &backup_dir);
        write_keshbackup(&export, &documents, temporaire)
    };
    let (archive, meta) = build_keshbackup(&state.pool, documents_dir(&state), remplir)
        .await
        .map_err(AppError::from)?;

    // Audit best-effort (handler-level, pas dans build_keshbackup — réutilisé
    // sans audit par le backup pré-import 17-3c). Un échec d'INSERT n'empêche
    // pas le téléchargement.
    emit_full_export_audit(&state, &current_user, &meta).await;

    let filename = format!(
        "kesh-installation-{}.keshbackup",
        Utc::now().date_naive().format("%Y-%m-%d")
    );
    let content_disposition = crate::util::build_content_disposition(&filename, "fr-CH")?;
    let body = corps_de_l_archive(archive)?;

    let plafond_mib = state.config.admin_import_max_mib;
    let mut reponse = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_DISPOSITION, content_disposition)
        .header(HEADER_FILES, meta.file_count)
        .header(HEADER_FILES_NOT_INCLUDED, meta.files_not_included);
    if meta.byte_size > u64::from(plafond_mib) * 1024 * 1024 {
        reponse = reponse.header(HEADER_EXCEEDS_IMPORT_LIMIT, plafond_mib);
    }
    reponse
        .body(body)
        .map_err(|e| AppError::Internal(format!("response build: {e}")))
}

/// Le dossier des documents de l'installation (`KESH_DOCUMENTS_DIR`).
fn documents_dir(state: &AppState) -> PathBuf {
    PathBuf::from(&state.config.documents_dir)
}

/// Corps de réponse de l'archive : les octets s'ils sont restés en mémoire,
/// sinon le fichier anonyme du dossier de sauvegarde, rembobiné et diffusé
/// (il disparaît avec son descripteur, à la fin de la diffusion).
fn corps_de_l_archive(archive: tempfile::SpooledTempFile) -> Result<Body, AppError> {
    match archive.into_inner() {
        tempfile::SpooledData::InMemory(cursor) => Ok(Body::from(cursor.into_inner())),
        tempfile::SpooledData::OnDisk(mut fichier) => {
            fichier
                .seek(SeekFrom::Start(0))
                .map_err(|e| AppError::AdminFullExportFailed(format!("temporaire : {e}")))?;
            let fichier = tokio::fs::File::from_std(fichier);
            Ok(Body::from_stream(tokio_util::io::ReaderStream::new(
                fichier,
            )))
        }
    }
}

/// Story 31-1a (AC4, AC11) — crée au besoin le dossier de sauvegarde
/// (`KESH_ADMIN_BACKUP_DIR`), qui reçoit aussi les temporaires de l'export et
/// de l'import. Story 15-13b (#552) : quand Kesh le crée, **chaque niveau
/// créé** l'est en `0700`, parents compris (`DirBuilder` récursif) ; un dossier
/// existant (montage de l'hôte) n'est pas modifié. Rend le détail de l'échec
/// (journal seulement) : l'appelant choisit la variante d'erreur.
async fn ensure_backup_dir(dir: &str) -> Result<(), String> {
    let mut builder = tokio::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    builder.mode(0o700);
    builder
        .create(dir)
        .await
        .map_err(|e| format!("création répertoire backup '{dir}' : {e}"))
}

/// POST /api/v1/admin/full-import — import complet d'installation (.keshbackup).
///
/// Monté dans `admin_routes` (RBAC `require_admin_role`) avec un
/// `DefaultBodyLimit` propre (`KESH_ADMIN_IMPORT_MAX_MB`). **Interdit aux clés
/// PAT** (AC2) : opération d'infra destructrice, jamais via intégration API.
///
/// Séquence (DC4/DC5/DC6, Story 31-1a) :
/// 1. anti-PAT ; **réception sur disque** (AC11) : le champ `file` est copié
///    par morceaux dans un temporaire anonyme de `KESH_ADMIN_BACKUP_DIR` (créé
///    s'il manque) ;
/// 2. tout le reste dans une **tâche `tokio::spawn` attendue** (AC8b) : si le
///    client abandonne, la tâche va au bout — jamais une restauration
///    interrompue au milieu ;
/// 3. **validation avant tout effet** : structure, intégrité et cohérence des
///    pièces ([`parse_and_verify`], en `spawn_blocking`), compat version 409,
///    compat colonnes 400 ([`check_schema_compat`]) ;
/// 4. verrou d'installation (`_kesh_version id=1 FOR UPDATE`, pattern 17-1) ;
///    **sauvegarde de sécurité** (cœur `build_keshbackup`, pièces comprises)
///    → disque — jamais d'import sans sauvegarde réussie ;
/// 5. **écriture des pièces** de l'archive sous `KESH_DOCUMENTS_DIR`, avant le
///    remplacement des tables (AC8) : un refus d'écriture laisse la base
///    intacte ;
/// 6. **restore transactionnel** (`DELETE`+`INSERT`, FK_CHECKS=0) + audit
///    in-tx (`user_id = MIN(admin)` source, O-1) + DC11 onboarding + COMMIT ;
///    la réponse compte les pièces restaurées et manquantes (AC9).
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

    // 1. Réception sur disque (AC11). Rien n'a commencé : un dossier
    //    inutilisable est un échec d'avant la sauvegarde.
    ensure_backup_dir(&state.config.admin_backup_dir)
        .await
        .map_err(AppError::AdminPreImportBackupFailed)?;
    let recu = read_upload(multipart, &state.config.admin_backup_dir).await?;

    // 2. Le reste dans une tâche attendue (AC8b) : l'abandon du futur du
    //    handler par hyper (client parti) n'interrompt pas la tâche.
    let tache = tokio::spawn(async move {
        // 3a. Structure + intégrité + cohérence (avant tout effet), hors du
        //     fil de l'exécuteur (lecture disque, SHA-256 de plusieurs Gio).
        let limites = ReadLimits::production(state.config.admin_import_max_mib);
        let parsed = tokio::task::spawn_blocking(move || parse_and_verify(recu, &limites))
            .await
            .map_err(|e| AppError::Internal(format!("vérification de l'archive : {e}")))??;

        // Story 31-1c (AC27) — une seule sauvegarde à la fois : garde POSSÉDÉ,
        // pris dans la tâche (un client parti ne le relâche pas), APRÈS la
        // réception et la vérification (un envoi lent ne bloque pas les
        // sauvegardes), tenu jusqu'à la fin de la restauration, et toujours
        // AVANT le `FOR UPDATE` de `_kesh_version` (`run_backup_and_restore`).
        let _garde = state.backup_lock.clone().lock_owned().await;

        // 3b. Compat version (DC4) : 409 si le backup exige plus récent que nous.
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

        // 3c. Compat colonnes bidirectionnelle (AC12c) → 400 IMPORT_SCHEMA_MISMATCH.
        //     Antérieur à la sauvegarde : un échec de lecture du schéma ne promet
        //     aucune copie (Story 15-13b, #576 — `avant_sauvegarde`).
        check_schema_compat(&state.pool, &parsed)
            .await
            .map_err(avant_sauvegarde)?;

        // 4 + 5 + 6. Sauvegarde pré-import, pièces, restore transactionnel. La
        //    sérialisation des imports destructeurs concurrents est assurée par
        //    un verrou `_kesh_version id=1 FOR UPDATE` pris **en tête de la
        //    transaction de restore** (cf. `run_backup_and_restore`) — pattern
        //    Story 17-1, auto-relâché au COMMIT/rollback.
        let source_version = parsed.manifest.kesh_version.clone();
        // Story 31-1c (AC28, F9) — sous le verrou, avant la sauvegarde de
        // sécurité : les `kesh-pre-import-*.keshbackup.partial` laissés par un
        // arrêt brutal sont supprimés (sinon jamais : exclus de la liste du
        // serveur). Un échec est journalisé et ne refuse pas l'import.
        let dossier = PathBuf::from(&state.config.admin_backup_dir);
        if let Err(e) = tokio::task::spawn_blocking(move || {
            crate::auto_backup::supprimer_les_partiels(&dossier, "kesh-pre-import-")
        })
        .await
        {
            tracing::warn!("nettoyage des sauvegardes pré-import partielles : {e}");
        }
        let issue = run_backup_and_restore(&state, parsed, &current_user).await?;

        let body = serde_json::json!({
            "backupCreated": issue.backup_created,
            "tablesRestored": issue.tables_restored,
            "rowsRestored": issue.rows_restored,
            "sourceVersion": source_version,
            "sessionInvalidated": true,
            "filesRestored": issue.files_restored,
            "filesMissing": issue.files_missing,
        });
        Ok::<Response, AppError>((StatusCode::OK, Json(body)).into_response())
    });
    match tache.await {
        Ok(reponse) => reponse,
        Err(e) => {
            // Panique dans la tâche : non testée (angle mort écrit, AC8b).
            tracing::error!(error = ?e, "tâche d'import interrompue");
            Err(AppError::Internal(format!("tâche d'import : {e}")))
        }
    }
}

/// Lit le champ multipart `file` (le `.keshbackup`) **par morceaux** dans un
/// temporaire **anonyme** de `dir` (Story 31-1a, AC11) : sans nom, il
/// disparaît avec son descripteur, même après un `kill -9` — aucun résidu de
/// plusieurs Gio. Écrit par `tokio::fs`, puis rendu en `std::fs::File` pour la
/// vérification synchrone. Rejette un champ dupliqué ou absent ; les autres
/// champs sont ignorés.
async fn read_upload(mut multipart: Multipart, dir: &str) -> Result<std::fs::File, AppError> {
    use tokio::io::AsyncWriteExt;

    let mut recu: Option<tokio::fs::File> = None;
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("multipart illisible : {e}")))?
    {
        if field.name() != Some("file") {
            continue;
        }
        if recu.is_some() {
            return Err(AppError::Validation(
                "Champ 'file' dupliqué dans le multipart".into(),
            ));
        }
        let temporaire = tempfile::tempfile_in(dir).map_err(|e| {
            AppError::AdminPreImportBackupFailed(format!(
                "temporaire de réception dans '{dir}' : {e}"
            ))
        })?;
        let mut fichier = tokio::fs::File::from_std(temporaire);
        while let Some(morceau) = field
            .chunk()
            .await
            .map_err(|e| AppError::Validation(format!("lecture du fichier : {e}")))?
        {
            fichier.write_all(&morceau).await.map_err(|e| {
                AppError::AdminPreImportBackupFailed(format!("réception du fichier : {e}"))
            })?;
        }
        fichier.flush().await.map_err(|e| {
            AppError::AdminPreImportBackupFailed(format!("réception du fichier : {e}"))
        })?;
        recu = Some(fichier);
    }
    let fichier =
        recu.ok_or_else(|| AppError::Validation("Champ 'file' manquant dans le multipart".into()))?;
    Ok(fichier.into_std().await)
}

/// Ce que rend [`run_backup_and_restore`].
struct ImportOutcome {
    backup_created: bool,
    tables_restored: usize,
    rows_restored: usize,
    files_restored: usize,
    files_missing: usize,
}

/// Backup pré-import (sans audit) + écriture des pièces + restore
/// transactionnel + audit in-tx. Reçoit le backup **par valeur** : l'archive
/// en est déplacée dans la tâche d'écriture des pièces.
async fn run_backup_and_restore(
    state: &AppState,
    mut parsed: ParsedBackup<std::fs::File>,
    current_user: &CurrentUser,
) -> Result<ImportOutcome, AppError> {
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
    //    lecture (ils bloquent au FOR UPDATE ci-dessus). Depuis la Story 31-1a,
    //    elle emporte aussi les pièces du dossier des documents.
    let backup_created = write_pre_import_backup(state).await?;

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

    // 4-ter (Story 31-1a, AC8) — LES PIÈCES AVANT LA BASE. Écrites sous
    //    `KESH_DOCUMENTS_DIR` avant le `DELETE` des tables : un refus d'écriture
    //    (dossier non inscriptible, disque plein) refuse la restauration, base
    //    intacte. Les pièces déjà écrites restent — nommées par leur contenu,
    //    elles sont inoffensives. En `spawn_blocking` : `std::fs`, `fsync`.
    let archive = parsed.archive.take().ok_or_else(|| {
        AppError::AdminFullImportFailed("archive du backup déjà consommée".into())
    })?;
    let pieces = parsed.manifest.files.clone();
    let documents = documents_dir(state);
    let files_restored = tokio::task::spawn_blocking(move || {
        let mut archive = archive;
        ecrire_les_pieces(&mut archive, &pieces, &documents)
    })
    .await
    .map_err(|e| AppError::AdminFullImportFailed(format!("écriture des pièces : {e}")))?
    .map_err(|e| AppError::AdminFullImportFailed(format!("écriture des pièces : {e}")))?;

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

    // 5-quater. Réparation de l'installation restaurée (Story 15-7b3, #528) —
    //    une archive prise sur une installation atteinte ramène des
    //    utilisateurs et des clés d'API désignant une société effacée. Même
    //    règle qu'au démarrage (`repair_installation_in_tx`), **sans** la
    //    suppression des sociétés provisoires superflues (#542), que fait le
    //    démarrage suivant : une archive est l'état choisi par l'administrateur.
    //    Signée par `audit_uid`, l'acteur d'`admin.full_import`, pour la même
    //    raison ; `triggered_by_user` au détail. **Avant** `books.restored` et
    //    `admin.full_import` : leur `company_id` (sous-SELECT sur `users`)
    //    désigne ainsi la société vivante. Une erreur annule l'import entier,
    //    comme toute autre — la règle non bloquante ne vaut qu'au démarrage.
    //    Aucun verrou neuf : `companies`, `users` et `api_keys` viennent d'être
    //    vidées puis réinsérées par cette transaction.
    kesh_db::repositories::companies::repair_installation_in_tx(
        &mut tx,
        kesh_db::repositories::companies::RepairTrigger::Restore {
            actor_user_id: audit_uid,
            triggered_by_user: current_user.user_id,
        },
    )
    .await
    .map_err(|e| AppError::AdminFullImportFailed(format!("réparation de l'installation : {e}")))?;

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

    // Story 31-1a (AC9) — pièces référencées par la base RESTAURÉE qui ne sont
    // pas lisibles et intactes dans le dossier des documents, après l'écriture
    // (chaque pièce présente est hachée : `spawn_blocking`).
    let references = parsed.referenced_files();
    let documents = documents_dir(state);
    let files_missing =
        tokio::task::spawn_blocking(move || compter_les_pieces_manquantes(&references, &documents))
            .await
            .map_err(|e| AppError::AdminFullImportFailed(format!("contrôle des pièces : {e}")))?;

    let details = serde_json::json!({
        "source_kesh_version": parsed.manifest.kesh_version,
        "source_instance_id": parsed.manifest.instance_id,
        "source_format_version": parsed.manifest.format_version,
        "triggered_by_user": current_user.user_id,
        "tables_restored": tables_restored,
        "rows_restored": rows_restored,
        "files_restored": files_restored,
        "files_missing": files_missing,
        // Story 16-1c — rapport de rejeu. Cette information de diagnostic vit
        // dans l'audit et le journal serveur, que l'exploitant d'un restore
        // consulte de toute façon depuis la machine.
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

    Ok(ImportOutcome {
        backup_created,
        tables_restored,
        rows_restored,
        files_restored,
        files_missing,
    })
}

/// Story 15-13b (#576) — convertit, **à l'appel**, un échec né hors de ce
/// module **avant** que la sauvegarde pré-import soit sur disque
/// (`check_schema_compat`, `build_keshbackup`) : `AdminFullImportFailed` et
/// `AdminFullExportFailed` deviennent `AdminPreImportBackupFailed`, détail
/// conservé ; toute autre variante est rendue telle quelle
/// (`ImportSchemaMismatch` reste un 400). Story 31-1a : accepte aussi
/// l'erreur propre de `build_keshbackup` (`BackupError`), convertie d'abord en
/// `AdminFullExportFailed`. Forme d'appel prescrite :
/// `….await.map_err(avant_sauvegarde)?`, gardée lexicalement par le test
/// `avant_sauvegarde_branchee_aux_appels_de_l_import`.
fn avant_sauvegarde(err: impl Into<AppError>) -> AppError {
    match err.into() {
        AppError::AdminFullImportFailed(detail) | AppError::AdminFullExportFailed(detail) => {
            AppError::AdminPreImportBackupFailed(detail)
        }
        autre => autre,
    }
}

/// Écrit la sauvegarde de sécurité pré-import sur disque (filet avant le
/// restore destructeur), **pièces comprises** depuis la Story 31-1a. Le chemin
/// est **journalisé côté serveur uniquement** (jamais exposé en réponse). Le
/// dossier est créé au besoin ([`ensure_backup_dir`], `0700`). Le fichier est
/// écrit par [`write_backup_file`] — mode `0600`, sous `<nom>.partial` puis
/// renommé, jamais par-dessus un fichier existant — dans le `spawn_blocking`
/// de `build_keshbackup`. Tout échec rend `AdminPreImportBackupFailed` : aucun
/// import sans sauvegarde réussie (DC5).
async fn write_pre_import_backup(state: &AppState) -> Result<bool, AppError> {
    let dir = &state.config.admin_backup_dir;
    ensure_backup_dir(dir)
        .await
        .map_err(AppError::AdminPreImportBackupFailed)?;
    // Nom unique même pour deux sauvegardes dans la même milliseconde : pid +
    // compteur atomique process-global (review Pass 2). (La sauvegarde est
    // prise SOUS le verrou `FOR UPDATE` de `run_backup_and_restore`.)
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let path = Path::new(dir).join(format!(
        "kesh-pre-import-{}-{}-{}.keshbackup",
        Utc::now().format("%Y%m%dT%H%M%S%3f"),
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    let chemin = path.clone();
    let remplir = move |export: InstallationExport, documents: PathBuf| {
        write_backup_file(&chemin, |fichier| {
            write_keshbackup(&export, &documents, fichier).map(|(_, meta)| meta)
        })
    };
    let meta = build_keshbackup(&state.pool, documents_dir(state), remplir)
        .await
        .map_err(avant_sauvegarde)?;
    tracing::info!(
        path = %path.display(),
        bytes = meta.byte_size,
        files = meta.file_count,
        files_not_included = meta.files_not_included,
        "backup pré-import écrit (filet de sécurité avant restore)"
    );
    Ok(true)
}

/// Story 15-13b (#552), synchrone depuis la Story 31-1a (AC4) — écrit sous
/// `path`, par la fermeture `remplir`, sans jamais exposer un fichier
/// incomplet sous ce nom ni écraser un fichier existant :
///
/// 1. crée `<path>.partial` (même dossier, donc même système de fichiers) en
///    `create_new` et, sous Unix, mode `0600` — un `.partial` déjà présent
///    fait échouer l'appel **sans rien supprimer** (on ne supprime que ce
///    qu'on a créé) ;
/// 2. passe à `remplir` un **clone** du descripteur (`try_clone`), qu'il
///    possède (il entre dans un `spawn_blocking`) ;
/// 3. `sync_all` sur l'original (sur btrfs ou un partage réseau, `ENOSPC` et
///    `EDQUOT` surgissent souvent ici) ;
/// 4. vérifie que `path` n'existe pas (`try_exists` en erreur vaut échec,
///    jamais « absent ») — `rename` remplacerait une cible existante ;
/// 5. renomme `<path>.partial` en `path` (atomique sur un même système de
///    fichiers).
///
/// Si 2 à 5 échouent, le `.partial` créé est supprimé au mieux (un échec de
/// suppression est journalisé en `warn!` avec son chemin, sans masquer
/// l'erreur d'origine). L'erreur est typée ([`BackupWriteError`], `io::Error`
/// conservée) ; l'adaptation en `AppError` revient à l'appelant.
///
/// Angles morts écrits (non traités) :
/// - la fenêtre entre la vérification 4 et le renommage 5 n'est pas fermée
///   (le nom est unique par construction) ;
/// - **durabilité** : le dossier n'est pas synchronisé après le renommage —
///   après une coupure de courant, l'entrée du dossier peut manquer alors
///   que le contenu était synchronisé ;
/// - `try_exists` **suit** les liens symboliques : un lien pendant au nom
///   final est vu absent, et `rename` remplace le lien (pas sa cible) ; sans
///   portée (nom unique, dossier `0700` créé par Kesh).
pub(crate) fn write_backup_file<T>(
    path: &Path,
    remplir: impl FnOnce(std::fs::File) -> Result<T, BackupWriteError>,
) -> Result<T, BackupWriteError> {
    let mut partial_os = path.as_os_str().to_owned();
    partial_os.push(".partial");
    let partial = PathBuf::from(partial_os);

    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let file = options.open(&partial).map_err(|e| {
        BackupWriteError::new(
            format!("création fichier partiel '{}'", partial.display()),
            e,
        )
    })?;

    let resultat = (|| -> Result<T, BackupWriteError> {
        let clone = file.try_clone().map_err(|e| {
            BackupWriteError::new(format!("descripteur '{}'", partial.display()), e)
        })?;
        let valeur = remplir(clone)?;
        file.sync_all().map_err(|e| {
            BackupWriteError::new(format!("synchronisation backup '{}'", partial.display()), e)
        })?;
        match path.try_exists() {
            Ok(false) => {}
            Ok(true) => {
                return Err(BackupWriteError::new(
                    format!("nom final déjà présent '{}'", path.display()),
                    std::io::Error::from(std::io::ErrorKind::AlreadyExists),
                ));
            }
            Err(e) => {
                return Err(BackupWriteError::new(
                    format!("vérification nom final '{}'", path.display()),
                    e,
                ));
            }
        }
        std::fs::rename(&partial, path).map_err(|e| {
            BackupWriteError::new(
                format!(
                    "renommage backup '{}' → '{}'",
                    partial.display(),
                    path.display()
                ),
                e,
            )
        })?;
        Ok(valeur)
    })();

    if resultat.is_err()
        && let Err(e) = std::fs::remove_file(&partial)
    {
        tracing::warn!(
            path = %partial.display(),
            "fichier partiel non supprimé après un échec d'écriture de la sauvegarde \
             — ce n'est PAS une sauvegarde valide, il peut être supprimé : {e}"
        );
    }
    resultat
}

/// Émet `audit_log` `action='admin.full_export'`, `entity_type='installation'`
/// (best-effort, snake_case `details_json` pour les JSON paths SQL). Story
/// 31-1a (AC5) : `file_count`, `files_bytes`, `files_not_included`.
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
                    "file_count": meta.file_count,
                    "files_bytes": meta.files_bytes,
                    "files_not_included": meta.files_not_included,
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
    //! Story 31-1a (#503) : `write_backup_file` synchrone, dossier de
    //! sauvegarde créé par `ensure_backup_dir`, import dans une tâche détachée.

    use super::*;
    use std::io::Write;

    /// Fermeture de remplissage de test : écrit `contenu`.
    fn ecrire(
        contenu: &'static [u8],
    ) -> impl FnOnce(std::fs::File) -> Result<(), BackupWriteError> {
        move |mut f| {
            f.write_all(contenu)
                .map_err(|e| BackupWriteError::new("test", e))
        }
    }

    /// Test 11 — `write_backup_file` : fichier en `0600`, contenu identique,
    /// aucun `.partial` restant. Assertion de montage : un fichier témoin
    /// créé par `std::fs` n'est PAS en `0600` — sinon l'umask rend le test
    /// non discriminant.
    #[cfg(unix)]
    #[test]
    fn write_backup_file_en_0600() {
        use std::os::unix::fs::PermissionsExt;
        let racine = tempfile::tempdir().expect("tempdir");
        let mode = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        let temoin = racine.path().join("temoin.txt");
        std::fs::write(&temoin, b"t").unwrap();
        assert_ne!(
            mode(&temoin),
            0o600,
            "umask de l'environnement trop restrictif (077 ?) : le test ne peut pas distinguer le mode posé par le code"
        );
        let path = racine.path().join("kesh-pre-import-x.keshbackup");
        write_backup_file(&path, ecrire(b"contenu-de-test")).expect("écriture");
        assert_eq!(mode(&path), 0o600, "sauvegarde en 0600");
        assert_eq!(std::fs::read(&path).unwrap(), b"contenu-de-test");
        assert!(
            !racine
                .path()
                .join("kesh-pre-import-x.keshbackup.partial")
                .exists()
        );
    }

    /// Revue P1 (E3, A-4) — `ensure_backup_dir` sur deux niveaux absents :
    /// **chaque** niveau créé est en `0700`, le parent intermédiaire compris
    /// (`DirBuilder` récursif) ; et un dossier **existant** n'est pas modifié
    /// (`0755` reste `0755`, cas du `./backup` créé par Docker). Assertion de
    /// montage : un dossier témoin créé par `std::fs` n'est pas en `0700`.
    #[cfg(unix)]
    #[tokio::test]
    async fn ensure_backup_dir_dossiers_crees_et_existants() {
        use std::os::unix::fs::PermissionsExt;
        let mode = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
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
        ensure_backup_dir(feuille.to_str().unwrap())
            .await
            .expect("création sous deux niveaux absents");
        assert_eq!(mode(&parent), 0o700, "parent intermédiaire créé par Kesh");
        assert_eq!(mode(&feuille), 0o700, "dossier de sauvegarde créé par Kesh");

        let existant = racine.path().join("existant");
        std::fs::create_dir(&existant).unwrap();
        std::fs::set_permissions(&existant, std::fs::Permissions::from_mode(0o755)).unwrap();
        ensure_backup_dir(existant.to_str().unwrap())
            .await
            .expect("dossier existant");
        assert_eq!(
            mode(&existant),
            0o755,
            "un dossier existant n'est pas modifié"
        );
    }

    /// Test 12 (a) — un nom final déjà occupé fait échouer l'écriture
    /// (`AlreadyExists`) ; le fichier existant garde son contenu et aucun
    /// `.partial` ne reste.
    #[test]
    fn write_backup_file_n_ecrase_pas() {
        let racine = tempfile::tempdir().expect("tempdir");
        let path = racine.path().join("kesh-pre-import-x.keshbackup");
        std::fs::write(&path, b"existant").unwrap();
        let err = write_backup_file(&path, ecrire(b"nouveau"))
            .expect_err("le nom final existe : échec attendu");
        assert_eq!(err.source.kind(), std::io::ErrorKind::AlreadyExists);
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
    #[test]
    fn write_backup_file_passe_par_un_partiel() {
        let racine = tempfile::tempdir().expect("tempdir");
        let path = racine.path().join("kesh-pre-import-y.keshbackup");
        let partial = racine.path().join("kesh-pre-import-y.keshbackup.partial");
        std::fs::write(&partial, b"interrompu").unwrap();
        let err = write_backup_file(&path, ecrire(b"complet"))
            .expect_err("un .partial préexistant doit faire échouer l'appel");
        assert_eq!(err.source.kind(), std::io::ErrorKind::AlreadyExists);
        assert!(!path.exists(), "le nom final ne doit pas apparaître");
        assert_eq!(std::fs::read(&partial).unwrap(), b"interrompu");
    }

    /// Story 31-1a (AC4) — une erreur de la fermeture de remplissage est
    /// rendue telle quelle (`kind()` conservé) et le `.partial` est supprimé.
    #[test]
    fn write_backup_file_rend_l_erreur_du_remplissage() {
        let racine = tempfile::tempdir().expect("tempdir");
        let path = racine.path().join("kesh-pre-import-z.keshbackup");
        let err = write_backup_file(&path, |_| -> Result<(), BackupWriteError> {
            Err(BackupWriteError::new(
                "test",
                std::io::Error::from(std::io::ErrorKind::StorageFull),
            ))
        })
        .expect_err("échec du remplissage");
        assert_eq!(err.source.kind(), std::io::ErrorKind::StorageFull);
        assert!(std::fs::read_dir(racine.path()).unwrap().next().is_none());
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
        // Story 31-1a — l'erreur propre de `build_keshbackup`.
        match avant_sauvegarde(crate::admin_backup::export::BackupError::Task("t".into())) {
            AppError::AdminPreImportBackupFailed(d) => assert!(d.contains('t')),
            autre => panic!("conversion attendue, trouvé {autre:?}"),
        }
    }

    /// Test 20 — garde LEXICALE (C-15-13-28) : `avant_sauvegarde` est appliquée
    /// aux appels de `check_schema_compat` et `build_keshbackup` de l'import
    /// (`full_import`, `run_backup_and_restore`, et depuis la Story 31-1a
    /// `write_pre_import_backup`, qui porte l'appel de la sauvegarde
    /// pré-import), et PAS à celui de l'export (`full_export`). Lit ce
    /// fichier, tronqué à la première ligne `#[cfg(test)]`, commentaires `//`
    /// écartés ; une instruction court jusqu'au `;` suivant (rustfmt peut la
    /// couper) — d'où les fermetures `remplir` nommées **avant** l'appel.
    /// Assertion de montage : un appel de `check_schema_compat`, deux de
    /// `build_keshbackup`.
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
                "full_import" | "run_backup_and_restore" | "write_pre_import_backup" => assert!(
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

    /// Corps de la fonction `signature` (de sa ligne à la première ligne
    /// réduite à `}`) dans le code de production.
    fn corps(signature: &str) -> String {
        let code = code_de_production();
        let debut = code
            .find(signature)
            .unwrap_or_else(|| panic!("{signature} introuvable"));
        let fin = debut
            + code[debut..]
                .find("\n}")
                .expect("fin de fonction introuvable");
        code[debut..fin].to_string()
    }

    /// Revue P1 (E5), étendue par la Story 31-1a — garde LEXICALE des
    /// conversions directes : tout ce qui précède l'écriture de la sauvegarde
    /// pré-import — dans `run_backup_and_restore`, jusqu'à l'appel de
    /// `write_pre_import_backup` (ouverture de transaction, verrou `FOR
    /// UPDATE`, ligne `_kesh_version` absente) ; et dans
    /// `write_pre_import_backup`, jusqu'à la fin de l'instruction
    /// `build_keshbackup` (dossier, sauvegarde) — ne construit ni
    /// `AdminFullImportFailed` ni `AdminFullExportFailed`, et chaque `?` y est
    /// couvert : autant de `?` que de `AdminPreImportBackupFailed` construites
    /// plus de `.map_err(avant_sauvegarde)`. Assertions de montage : trois
    /// constructions directes dans le premier segment ; une construction et
    /// une conversion dans le second. Un nouveau site d'échec fait rougir le
    /// test : il est à trier, puis le décompte à mettre à jour.
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
        let premier = code[debut..fin].to_string();

        let ecriture = corps("async fn write_pre_import_backup(");
        let appel = ecriture
            .find("build_keshbackup(")
            .expect("appel de build_keshbackup introuvable");
        let fin_appel = appel + ecriture[appel..].find(';').expect("fin d'instruction");
        let second = ecriture[..=fin_appel].to_string();

        for (segment, attendu) in [(&premier, (3, 0)), (&second, (1, 1))] {
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
                attendu,
                "assertion de montage sur le segment :\n{segment}"
            );
            assert_eq!(
                segment.matches('?').count(),
                directes + par_avant_sauvegarde,
                "un `?` avant la sauvegarde n'est pas couvert par une conversion :\n{segment}"
            );
        }
    }

    /// Story 31-1a (AC8b, M-a16) — garde LEXICALE : le corps de `full_import`
    /// lance une tâche `tokio::spawn(` et l'appel de `run_backup_and_restore`
    /// se trouve **dans** la fermeture de cette tâche (entre la parenthèse
    /// ouvrante du `spawn` et sa fermante, appariées), jamais hors d'elle.
    #[test]
    fn import_dans_une_tache_detachee() {
        let corps = corps("pub async fn full_import(");
        let spawn = corps
            .find("tokio::spawn(")
            .expect("tokio::spawn( absent de full_import");
        let ouvrante = spawn + "tokio::spawn".len();
        let mut profondeur = 0usize;
        let mut fermante = None;
        for (i, c) in corps[ouvrante..].char_indices() {
            match c {
                '(' => profondeur += 1,
                ')' => {
                    profondeur -= 1;
                    if profondeur == 0 {
                        fermante = Some(ouvrante + i);
                        break;
                    }
                }
                _ => {}
            }
        }
        let fermante = fermante.expect("parenthèse fermante du spawn introuvable");
        let appels: Vec<usize> = corps
            .match_indices("run_backup_and_restore(")
            .map(|(p, _)| p)
            .collect();
        assert_eq!(
            appels.len(),
            1,
            "un seul appel de run_backup_and_restore attendu"
        );
        assert!(
            appels[0] > spawn && appels[0] < fermante,
            "run_backup_and_restore doit être appelée dans la tâche du spawn"
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
