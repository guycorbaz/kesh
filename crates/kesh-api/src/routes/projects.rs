//! Endpoints CRUD des projets analytiques (Epic 19, Story 19-1).
//!
//! - `GET    /api/v1/projects`             liste (tout rôle) — `?includeArchived`
//! - `GET    /api/v1/projects/{id}`        détail (tout rôle)
//! - `POST   /api/v1/projects`             créer (Comptable+)
//! - `PUT    /api/v1/projects/{id}`        modifier (Comptable+, verrou optimiste)
//! - `POST   /api/v1/projects/{id}/archive`   archiver (Comptable+)
//! - `POST   /api/v1/projects/{id}/unarchive` désarchiver (Comptable+)
//!
//! Scoping `company_id` sur toute requête. Mutations sous sentinel lock + audit_log,
//! dans une transaction. Hiérarchie 2 niveaux validée ici (repo mécanique).

use axum::Extension;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chrono::{NaiveDate, NaiveDateTime};
use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::{NewProject, Project, UpdateProject};
use kesh_db::repositories::{audit_log, bank_accounts, projects};
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Transaction};

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectResponse {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub archived: bool,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub version: i32,
    pub created_at: NaiveDateTime,
}

impl From<Project> for ProjectResponse {
    fn from(p: Project) -> Self {
        Self {
            id: p.id,
            parent_id: p.parent_id,
            code: p.code,
            name: p.name,
            description: p.description,
            archived: p.archived,
            start_date: p.start_date,
            end_date: p.end_date,
            version: p.version,
            created_at: p.created_at,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListProjectsQuery {
    #[serde(default)]
    pub include_archived: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectBody {
    pub parent_id: Option<i64>,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectBody {
    pub parent_id: Option<i64>,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub version: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveProjectBody {
    pub version: i32,
}

/// Valide les champs textuels (non vides + longueurs `VARCHAR`) et normalise (trim).
/// Les bornes correspondent au schéma (`code VARCHAR(32)`, `name VARCHAR(150)`) : sans
/// ces gardes, une saisie trop longue déclencherait une erreur DB 1406 non mappée → 500.
fn validate_fields(code: &str, name: &str) -> Result<(String, String), AppError> {
    let code = code.trim().to_string();
    let name = name.trim().to_string();
    if code.is_empty() {
        return Err(AppError::Validation("Le code du projet est requis.".into()));
    }
    if code.chars().count() > 32 {
        return Err(AppError::Validation(
            "Le code du projet dépasse 32 caractères.".into(),
        ));
    }
    if name.is_empty() {
        return Err(AppError::Validation("Le nom du projet est requis.".into()));
    }
    if name.chars().count() > 150 {
        return Err(AppError::Validation(
            "Le nom du projet dépasse 150 caractères.".into(),
        ));
    }
    Ok((code, name))
}

/// Garde de hiérarchie **2 niveaux** (DC2). `self_id` = id du projet en cours de
/// modification (`None` à la création). Si `parent_id` est renseigné : le parent doit
/// exister (même company), être une **racine** (`parent_id IS NULL`), et différer de
/// soi-même. Sur un update, un projet qui a déjà des sous-projets ne peut pas devenir
/// lui-même sous-projet.
async fn validate_hierarchy(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    parent_id: Option<i64>,
    self_id: Option<i64>,
) -> Result<(), AppError> {
    let Some(pid) = parent_id else {
        return Ok(()); // projet racine : rien à valider
    };
    if Some(pid) == self_id {
        return Err(AppError::Validation(
            "Un projet ne peut pas être son propre parent.".into(),
        ));
    }
    let parent = projects::find_by_id_in_tx(tx, company_id, pid)
        .await?
        .ok_or_else(|| AppError::Validation("Le projet parent est introuvable.".into()))?;
    if parent.parent_id.is_some() {
        return Err(AppError::Validation(
            "Un sous-projet ne peut pas être parent (hiérarchie limitée à 2 niveaux).".into(),
        ));
    }
    // Un parent archivé rendrait le nouveau sous-projet actif « orphelin » dans la
    // vue par défaut (sa racine n'y figure plus). On l'interdit (M2).
    if parent.archived {
        return Err(AppError::Validation(
            "Le projet parent est archivé — désarchivez-le d'abord.".into(),
        ));
    }
    if let Some(id) = self_id {
        if projects::has_children(tx, company_id, id).await? {
            return Err(AppError::Validation(
                "Ce projet a des sous-projets : il ne peut pas devenir lui-même sous-projet."
                    .into(),
            ));
        }
    }
    Ok(())
}

/// `GET /api/v1/projects` — liste des projets (tout rôle).
pub async fn list_projects(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ListProjectsQuery>,
) -> Result<Json<Vec<ProjectResponse>>, AppError> {
    let items =
        projects::list_by_company(&state.pool, current_user.company_id, query.include_archived)
            .await?;
    Ok(Json(items.into_iter().map(ProjectResponse::from).collect()))
}

/// `GET /api/v1/projects/{id}` — détail d'un projet (tout rôle).
pub async fn get_project(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<ProjectResponse>, AppError> {
    let project = projects::get_for_company(&state.pool, current_user.company_id, id)
        .await?
        .ok_or(AppError::Database(kesh_db::errors::DbError::NotFound))?;
    Ok(Json(ProjectResponse::from(project)))
}

/// `POST /api/v1/projects` — crée un projet (Comptable+).
pub async fn create_project(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(body): Json<CreateProjectBody>,
) -> Result<(StatusCode, Json<ProjectResponse>), AppError> {
    let (code, name) = validate_fields(&body.code, &body.name)?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;
    bank_accounts::acquire_company_sentinel_lock(&mut tx, current_user.company_id).await?;

    validate_hierarchy(&mut tx, current_user.company_id, body.parent_id, None).await?;

    let new = NewProject {
        company_id: current_user.company_id,
        parent_id: body.parent_id,
        code,
        name,
        description: body
            .description
            .map(|d| d.trim().to_string())
            .filter(|d| !d.is_empty()),
        start_date: body.start_date,
        end_date: body.end_date,
    };
    let created = projects::create_for_company(&mut tx, &new).await?;

    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::for_actor(
            current_user.user_id,
            current_user.api_key_id,
            "project.created",
            "project",
            created.id,
            Some(serde_json::json!({ "code": created.code, "name": created.name })),
        ),
    )
    .await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit tx: {e}")))?;

    Ok((StatusCode::CREATED, Json(ProjectResponse::from(created))))
}

/// `PUT /api/v1/projects/{id}` — modifie un projet (Comptable+, verrou optimiste).
pub async fn update_project(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateProjectBody>,
) -> Result<Json<ProjectResponse>, AppError> {
    let (code, name) = validate_fields(&body.code, &body.name)?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;
    bank_accounts::acquire_company_sentinel_lock(&mut tx, current_user.company_id).await?;

    validate_hierarchy(&mut tx, current_user.company_id, body.parent_id, Some(id)).await?;

    let fields = UpdateProject {
        parent_id: body.parent_id,
        code,
        name,
        description: body
            .description
            .map(|d| d.trim().to_string())
            .filter(|d| !d.is_empty()),
        start_date: body.start_date,
        end_date: body.end_date,
    };
    let updated =
        projects::update_for_company(&mut tx, current_user.company_id, id, &fields, body.version)
            .await?;

    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::for_actor(
            current_user.user_id,
            current_user.api_key_id,
            "project.updated",
            "project",
            updated.id,
            Some(serde_json::json!({ "code": updated.code, "name": updated.name })),
        ),
    )
    .await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit tx: {e}")))?;

    Ok(Json(ProjectResponse::from(updated)))
}

async fn set_archived(
    state: AppState,
    current_user: CurrentUser,
    id: i64,
    archived: bool,
    version: i32,
) -> Result<Json<ProjectResponse>, AppError> {
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;
    bank_accounts::acquire_company_sentinel_lock(&mut tx, current_user.company_id).await?;

    // Gardes anti-orphelin (M2) : une racine ne peut être archivée tant qu'elle a des
    // sous-projets actifs ; un sous-projet ne peut être désarchivé si sa racine est
    // archivée. Ces deux règles interdisent l'état « sous-projet actif sous racine archivée ».
    let current = projects::find_by_id_in_tx(&mut tx, current_user.company_id, id)
        .await?
        .ok_or(AppError::Database(kesh_db::errors::DbError::NotFound))?;
    if archived {
        if projects::has_active_children(&mut tx, current_user.company_id, id).await? {
            return Err(AppError::Validation(
                "Ce projet a des sous-projets actifs — archivez-les d'abord.".into(),
            ));
        }
    } else if let Some(parent_id) = current.parent_id {
        let parent = projects::find_by_id_in_tx(&mut tx, current_user.company_id, parent_id)
            .await?
            .ok_or(AppError::Database(kesh_db::errors::DbError::NotFound))?;
        if parent.archived {
            return Err(AppError::Validation(
                "Le projet parent est archivé — désarchivez-le d'abord.".into(),
            ));
        }
    }

    let project =
        projects::set_archived_for_company(&mut tx, current_user.company_id, id, archived, version)
            .await?;

    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::for_actor(
            current_user.user_id,
            current_user.api_key_id,
            if archived {
                "project.archived"
            } else {
                "project.unarchived"
            },
            "project",
            project.id,
            None,
        ),
    )
    .await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit tx: {e}")))?;

    Ok(Json(ProjectResponse::from(project)))
}

/// `POST /api/v1/projects/{id}/archive` — archive un projet (Comptable+).
pub async fn archive_project(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(body): Json<ArchiveProjectBody>,
) -> Result<Json<ProjectResponse>, AppError> {
    set_archived(state, current_user, id, true, body.version).await
}

/// `POST /api/v1/projects/{id}/unarchive` — désarchive un projet (Comptable+).
pub async fn unarchive_project(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(body): Json<ArchiveProjectBody>,
) -> Result<Json<ProjectResponse>, AppError> {
    set_archived(state, current_user, id, false, body.version).await
}
