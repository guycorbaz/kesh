//! Repository des projets analytiques (Epic 19, Story 19-1).
//!
//! Toutes les fonctions sont **scopées `company_id`** (règle multi-tenant : pas de
//! `find_by_id(id)` global). Verrou optimiste (`version`) sur update/archive.
//! Jamais de hard-delete : archivage (`archived = TRUE`) pour préserver l'historique
//! analytique. La validation métier de la hiérarchie 2 niveaux se fait côté handler
//! (à l'aide de `find_by_id_in_tx` + `has_children`), à l'image du non-chevauchement
//! des taux TVA.

use sqlx::{MySql, MySqlPool, Transaction};

use crate::entities::{NewProject, Project, UpdateProject};
use crate::errors::{DbError, map_db_error};

const PROJECT_COLS: &str = "id, company_id, parent_id, code, name, description, archived, \
     start_date, end_date, version, created_at, updated_at";

/// Liste les projets d'une company (racines puis sous-projets, tri stable). Filtre
/// les archivés sauf si `include_archived`.
pub async fn list_by_company(
    pool: &MySqlPool,
    company_id: i64,
    include_archived: bool,
) -> Result<Vec<Project>, DbError> {
    // `COALESCE(parent_id, id)` regroupe chaque racine avec ses sous-projets ;
    // `parent_id IS NOT NULL` place la racine avant ses enfants ; `id` stabilise.
    let filter = if include_archived {
        ""
    } else {
        " AND archived = FALSE"
    };
    sqlx::query_as::<_, Project>(&format!(
        "SELECT {PROJECT_COLS} FROM projects WHERE company_id = ?{filter} \
         ORDER BY COALESCE(parent_id, id), parent_id IS NOT NULL, id"
    ))
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

/// Lecture scopée company (handler read-only, hors transaction).
pub async fn get_for_company(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
) -> Result<Option<Project>, DbError> {
    sqlx::query_as::<_, Project>(&format!(
        "SELECT {PROJECT_COLS} FROM projects WHERE company_id = ? AND id = ?"
    ))
    .bind(company_id)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Lecture scopée company **dans une transaction** (flux de mutation sous sentinel lock).
pub async fn find_by_id_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
) -> Result<Option<Project>, DbError> {
    sqlx::query_as::<_, Project>(&format!(
        "SELECT {PROJECT_COLS} FROM projects WHERE company_id = ? AND id = ?"
    ))
    .bind(company_id)
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)
}

/// Valide un ensemble de projets analytiques pour un tagging (Epic 19) : chaque
/// id doit exister, appartenir à `company_id` et ne pas être archivé.
///
/// Ordre de verrouillage global (docs/MULTI-TENANT-SCOPING-PATTERNS.md, Pattern 5) :
/// verrou sentinelle `companies` **une seule fois**, PUIS `FOR UPDATE` sur les
/// lignes projets — évite l'inversion ABBA (deadlock) avec le chemin d'archivage
/// (`set_archived` prend le sentinel puis met à jour le projet) et ferme la race
/// d'archivage concurrent. No-op si `project_ids` est vide (aucun verrou pris).
///
/// Erreurs : id manquant ou cross-company → [`DbError::NotFound`] ; projet
/// archivé → [`DbError::IllegalStateTransition`].
pub async fn validate_taggable_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    project_ids: &[i64],
) -> Result<(), DbError> {
    if project_ids.is_empty() {
        return Ok(());
    }
    let mut ids: Vec<i64> = project_ids.to_vec();
    ids.sort_unstable();
    ids.dedup();

    crate::repositories::bank_accounts::acquire_company_sentinel_lock(tx, company_id).await?;

    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!(
        "SELECT id, archived FROM projects \
         WHERE company_id = ? AND id IN ({placeholders}) FOR UPDATE"
    );
    let mut q = sqlx::query_as::<_, (i64, bool)>(&sql).bind(company_id);
    for pid in &ids {
        q = q.bind(pid);
    }
    let rows: Vec<(i64, bool)> = q.fetch_all(&mut **tx).await.map_err(map_db_error)?;

    if rows.len() != ids.len() {
        return Err(DbError::NotFound);
    }
    if rows.iter().any(|(_, archived)| *archived) {
        return Err(DbError::IllegalStateTransition(
            "le projet analytique est archivé".into(),
        ));
    }
    Ok(())
}

/// `true` si le projet a au moins un sous-projet (garde 2 niveaux : un projet
/// parent ne peut pas devenir lui-même sous-projet).
pub async fn has_children(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
) -> Result<bool, DbError> {
    let exists: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM projects WHERE company_id = ? AND parent_id = ? LIMIT 1")
            .bind(company_id)
            .bind(id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_db_error)?;
    Ok(exists.is_some())
}

/// `true` si le projet a au moins un sous-projet **actif** (non archivé). Garde
/// d'archivage : on refuse d'archiver une racine tant que des sous-projets actifs
/// y sont rattachés (sinon ils deviendraient orphelins dans la vue par défaut).
pub async fn has_active_children(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
) -> Result<bool, DbError> {
    let exists: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM projects WHERE company_id = ? AND parent_id = ? AND archived = FALSE LIMIT 1",
    )
    .bind(company_id)
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;
    Ok(exists.is_some())
}

/// Crée un projet. À appeler **sous sentinel lock**, après validation de la
/// hiérarchie (2 niveaux) par le handler. Unicité `(company_id, code)` → l'INSERT
/// remonte `UniqueConstraintViolation` (mappée 409 côté API).
pub async fn create_for_company(
    tx: &mut Transaction<'_, MySql>,
    new: &NewProject,
) -> Result<Project, DbError> {
    let result = sqlx::query(
        "INSERT INTO projects \
            (company_id, parent_id, code, name, description, start_date, end_date, archived, version) \
         VALUES (?, ?, ?, ?, ?, ?, ?, FALSE, 0)",
    )
    .bind(new.company_id)
    .bind(new.parent_id)
    .bind(&new.code)
    .bind(&new.name)
    .bind(&new.description)
    .bind(new.start_date)
    .bind(new.end_date)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let id = result.last_insert_id() as i64;
    find_by_id_in_tx(tx, new.company_id, id)
        .await?
        .ok_or_else(|| DbError::Invariant(format!("project create: id={id} not found post-insert")))
}

/// Modifie un projet (verrou optimiste). La validation 2 niveaux est faite par le
/// handler avant l'appel. Conflit de version → `OptimisticLockConflict` (409).
pub async fn update_for_company(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
    fields: &UpdateProject,
    expected_version: i32,
) -> Result<Project, DbError> {
    let existing = match find_by_id_in_tx(tx, company_id, id).await? {
        Some(p) => p,
        None => return Err(DbError::NotFound),
    };
    if existing.version != expected_version {
        return Err(DbError::OptimisticLockConflict);
    }

    let rows = sqlx::query(
        "UPDATE projects \
         SET parent_id = ?, code = ?, name = ?, description = ?, start_date = ?, end_date = ?, \
             version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ?",
    )
    .bind(fields.parent_id)
    .bind(&fields.code)
    .bind(&fields.name)
    .bind(&fields.description)
    .bind(fields.start_date)
    .bind(fields.end_date)
    .bind(id)
    .bind(company_id)
    .bind(expected_version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        return Err(DbError::OptimisticLockConflict);
    }

    find_by_id_in_tx(tx, company_id, id)
        .await?
        .ok_or(DbError::NotFound)
}

/// Archive ou désarchive un projet (soft-delete, verrou optimiste).
pub async fn set_archived_for_company(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
    archived: bool,
    expected_version: i32,
) -> Result<Project, DbError> {
    let existing = match find_by_id_in_tx(tx, company_id, id).await? {
        Some(p) => p,
        None => return Err(DbError::NotFound),
    };
    if existing.version != expected_version {
        return Err(DbError::OptimisticLockConflict);
    }

    let rows = sqlx::query(
        "UPDATE projects SET archived = ?, version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ?",
    )
    .bind(archived)
    .bind(id)
    .bind(company_id)
    .bind(expected_version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        return Err(DbError::OptimisticLockConflict);
    }

    find_by_id_in_tx(tx, company_id, id)
        .await?
        .ok_or(DbError::NotFound)
}
