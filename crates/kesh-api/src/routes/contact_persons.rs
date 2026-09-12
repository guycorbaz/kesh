//! Routes CRUD pour les personnes de contact d'une entreprise (#213, CRM).
//!
//! **Purement informatif** — jamais utilisé sur les factures / QR-bill / pain.001.
//! Nesté sous un contact `Entreprise` (`/contacts/{contactId}/persons`). Scopé
//! multi-tenant par `current_user.company_id`.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::contact::ContactType;
use kesh_db::entities::contact_person::{ContactPerson, ContactPersonUpdate, NewContactPerson};
use kesh_db::errors::{DbError, map_db_error};
use kesh_db::repositories::{audit_log, contact_persons, contacts};

use crate::AppState;
use crate::audit::AuditActor;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;

const MAX_NAME: usize = 70;
const MAX_ROLE: usize = 100;
const MAX_EMAIL: usize = 320;
const MAX_PHONE: usize = 50;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactPersonBody {
    pub first_name: String,
    pub last_name: String,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    /// Présent uniquement pour l'update (optimistic lock). Ignoré à la création.
    #[serde(default)]
    pub version: Option<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactPersonResponse {
    pub id: i64,
    pub contact_id: i64,
    pub first_name: String,
    pub last_name: String,
    pub role: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub version: i32,
}

impl From<ContactPerson> for ContactPersonResponse {
    fn from(p: ContactPerson) -> Self {
        Self {
            id: p.id,
            contact_id: p.contact_id,
            first_name: p.first_name,
            last_name: p.last_name,
            role: p.role,
            email: p.email,
            phone: p.phone,
            version: p.version,
        }
    }
}

fn norm(o: Option<String>) -> Option<String> {
    o.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// Champs validés/normalisés d'une personne de contact.
struct ValidatedPerson {
    first_name: String,
    last_name: String,
    role: Option<String>,
    email: Option<String>,
    phone: Option<String>,
}

fn validate(body: &ContactPersonBody) -> Result<ValidatedPerson, AppError> {
    let first_name = body.first_name.trim().to_string();
    let last_name = body.last_name.trim().to_string();
    if first_name.is_empty() || last_name.is_empty() {
        return Err(AppError::Validation(
            "Le prénom et le nom sont obligatoires".into(),
        ));
    }
    if first_name.chars().count() > MAX_NAME || last_name.chars().count() > MAX_NAME {
        return Err(AppError::Validation(format!(
            "Prénom / nom : maximum {MAX_NAME} caractères"
        )));
    }
    let role = norm(body.role.clone());
    if role.as_ref().is_some_and(|r| r.chars().count() > MAX_ROLE) {
        return Err(AppError::Validation(format!(
            "Fonction : maximum {MAX_ROLE} caractères"
        )));
    }
    let email = norm(body.email.clone());
    if let Some(ref e) = email
        && (e.chars().count() > MAX_EMAIL || !crate::routes::contacts::is_valid_email_simple(e))
    {
        return Err(AppError::Validation("Format d'email invalide".into()));
    }
    let phone = norm(body.phone.clone());
    if phone
        .as_ref()
        .is_some_and(|p| p.chars().count() > MAX_PHONE)
    {
        return Err(AppError::Validation(format!(
            "Téléphone : maximum {MAX_PHONE} caractères"
        )));
    }
    Ok(ValidatedPerson {
        first_name,
        last_name,
        role,
        email,
        phone,
    })
}

/// Vérifie que le contact parent existe, appartient à la company et est une Entreprise.
async fn ensure_entreprise_contact(
    state: &AppState,
    contact_id: i64,
    company_id: i64,
) -> Result<(), AppError> {
    let contact = contacts::find_by_id_in_company(&state.pool, contact_id, company_id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;
    if contact.contact_type != ContactType::Entreprise {
        return Err(AppError::Validation(
            "Les personnes de contact ne s'appliquent qu'aux contacts de type Entreprise".into(),
        ));
    }
    Ok(())
}

/// GET /api/v1/contacts/{contactId}/persons
pub async fn list_persons(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(contact_id): Path<i64>,
) -> Result<Json<Vec<ContactPersonResponse>>, AppError> {
    ensure_entreprise_contact(&state, contact_id, current_user.company_id).await?;
    let persons =
        contact_persons::list_by_contact(&state.pool, current_user.company_id, contact_id).await?;
    Ok(Json(persons.into_iter().map(Into::into).collect()))
}

/// Instantané d'une personne de contact pour `details_json` — Story 25-1b.
///
/// Les clés sont en **snake_case** : la surface HTTP est en camelCase, la piste
/// d'audit ne l'est pas, pour que `details_json->>'$.last_name'` marche en SQL.
fn person_snapshot_json(p: &ContactPerson) -> serde_json::Value {
    serde_json::json!({
        "contact_id": p.contact_id,
        "first_name": p.first_name,
        "last_name": p.last_name,
        "role": p.role,
        "email": p.email,
        "phone": p.phone,
        "version": p.version,
    })
}

/// POST /api/v1/contacts/{contactId}/persons
pub async fn create_person(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(contact_id): Path<i64>,
    Json(body): Json<ContactPersonBody>,
) -> Result<(StatusCode, Json<ContactPersonResponse>), AppError> {
    ensure_entreprise_contact(&state, contact_id, current_user.company_id).await?;
    let v = validate(&body)?;
    // Story 25-1b (AC 2, 7, 9) — le handler mène la transaction : l'audit doit
    // partager celle de la mutation, et `from_current_user` est obligatoire ici
    // car la route est dans `comptable_routes`, donc atteignable par un jeton.
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(map_db_error(e)))?;
    let person = contact_persons::create(
        &mut tx,
        NewContactPerson {
            company_id: current_user.company_id,
            contact_id,
            first_name: v.first_name,
            last_name: v.last_name,
            role: v.role,
            email: v.email,
            phone: v.phone,
        },
    )
    .await?;
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::from_current_user(
            &current_user,
            "contact_person.created",
            "contact_person",
            person.id,
            Some(person_snapshot_json(&person)),
        ),
    )
    .await?;
    tx.commit()
        .await
        .map_err(|e| AppError::Database(map_db_error(e)))?;
    Ok((StatusCode::CREATED, Json(person.into())))
}

/// PUT /api/v1/contact-persons/{id}
pub async fn update_person(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(body): Json<ContactPersonBody>,
) -> Result<Json<ContactPersonResponse>, AppError> {
    let version = body
        .version
        .ok_or_else(|| AppError::Validation("Le champ version est requis".into()))?;
    let v = validate(&body)?;
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(map_db_error(e)))?;
    // L'état « avant » est lu DANS la transaction : sans lui, la trace dirait ce
    // que la personne est devenue sans dire ce qu'elle était.
    let before =
        contact_persons::find_by_id_in_company_in_tx(&mut tx, id, current_user.company_id).await?;
    let person = contact_persons::update(
        &mut tx,
        id,
        current_user.company_id,
        version,
        ContactPersonUpdate {
            first_name: v.first_name,
            last_name: v.last_name,
            role: v.role,
            email: v.email,
            phone: v.phone,
        },
    )
    .await?;
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::from_current_user(
            &current_user,
            "contact_person.updated",
            "contact_person",
            id,
            Some(serde_json::json!({
                "before": before.as_ref().map(person_snapshot_json),
                "after": person_snapshot_json(&person),
            })),
        ),
    )
    .await?;
    tx.commit()
        .await
        .map_err(|e| AppError::Database(map_db_error(e)))?;
    Ok(Json(person.into()))
}

/// DELETE /api/v1/contact-persons/{id} — archive.
pub async fn delete_person(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    // ⚠️ Le handler ne connaît QUE l'identifiant : sans pré-chargement, la trace
    // ne nommerait personne. Précédent : `contact.archived`, qui journalise un
    // instantané complet.
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(map_db_error(e)))?;
    let before =
        contact_persons::find_by_id_in_company_in_tx(&mut tx, id, current_user.company_id).await?;
    contact_persons::archive(&mut tx, id, current_user.company_id).await?;
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::from_current_user(
            &current_user,
            "contact_person.archived",
            "contact_person",
            id,
            before.as_ref().map(person_snapshot_json),
        ),
    )
    .await?;
    tx.commit()
        .await
        .map_err(|e| AppError::Database(map_db_error(e)))?;
    Ok(StatusCode::NO_CONTENT)
}
