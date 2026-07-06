//! Routes CRUD pour les personnes de contact d'une entreprise (#213, CRM).
//!
//! **Purement informatif** — jamais utilisé sur les factures / QR-bill / pain.001.
//! Nesté sous un contact `Entreprise` (`/contacts/{contactId}/persons`). Scopé
//! multi-tenant par `current_user.company_id`.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

use kesh_db::entities::contact::ContactType;
use kesh_db::entities::contact_person::{ContactPerson, ContactPersonUpdate, NewContactPerson};
use kesh_db::errors::DbError;
use kesh_db::repositories::{contact_persons, contacts};

use crate::AppState;
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

/// POST /api/v1/contacts/{contactId}/persons
pub async fn create_person(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(contact_id): Path<i64>,
    Json(body): Json<ContactPersonBody>,
) -> Result<(StatusCode, Json<ContactPersonResponse>), AppError> {
    ensure_entreprise_contact(&state, contact_id, current_user.company_id).await?;
    let v = validate(&body)?;
    let person = contact_persons::create(
        &state.pool,
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
    let person = contact_persons::update(
        &state.pool,
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
    Ok(Json(person.into()))
}

/// DELETE /api/v1/contact-persons/{id} — archive.
pub async fn delete_person(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    contact_persons::archive(&state.pool, id, current_user.company_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
