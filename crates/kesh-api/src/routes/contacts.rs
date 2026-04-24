//! Routes CRUD pour les contacts du carnet d'adresses (Story 4.1).
//!
//! **Security Note (Story 6.2):** All handlers scope by `current_user.company_id` from JWT.
//! The company_id in JWT can become stale if a user is reassigned to a different company
//! during an active session. See `middleware/auth.rs` for staleness window (proportional to
//! `KESH_JWT_EXPIRY_MINUTES`, default 15 min).

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use kesh_core::listing::SortDirection;
use kesh_core::types::CheNumber;
use kesh_db::entities::contact::{Contact, ContactType, ContactUpdate, NewContact};
use kesh_db::errors::DbError;
use kesh_db::repositories::contacts::{self, ContactListQuery, ContactSortBy};

use crate::AppState;
use crate::errors::AppError;
use crate::helpers::get_company_for;
use crate::middleware::auth::CurrentUser;
use crate::routes::ListResponse;

// ---------------------------------------------------------------------------
// Limites
// ---------------------------------------------------------------------------

const MAX_NAME_LEN: usize = 255;
const MAX_EMAIL_LEN: usize = 320;
const MAX_PHONE_LEN: usize = 50;
const MAX_ADDRESS_LEN: usize = 500;
const MAX_PAYMENT_TERMS_LEN: usize = 100;
const MAX_LIST_LIMIT: i64 = 100;
const DEFAULT_LIST_LIMIT: i64 = 20;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListContactsQuery {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub contact_type: Option<ContactType>,
    #[serde(default)]
    pub is_client: Option<bool>,
    #[serde(default)]
    pub is_supplier: Option<bool>,
    #[serde(default)]
    pub include_archived: bool,
    #[serde(default)]
    pub sort_by: Option<ContactSortBy>,
    #[serde(default)]
    pub sort_direction: Option<SortDirection>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContactRequest {
    pub contact_type: ContactType,
    pub name: String,
    #[serde(default)]
    pub is_client: bool,
    #[serde(default)]
    pub is_supplier: bool,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub ide_number: Option<String>,
    #[serde(default)]
    pub default_payment_terms: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateContactRequest {
    pub contact_type: ContactType,
    pub name: String,
    #[serde(default)]
    pub is_client: bool,
    #[serde(default)]
    pub is_supplier: bool,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub ide_number: Option<String>,
    #[serde(default)]
    pub default_payment_terms: Option<String>,
    pub version: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveContactRequest {
    pub version: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactResponse {
    pub id: i64,
    pub company_id: i64,
    pub contact_type: ContactType,
    pub name: String,
    pub is_client: bool,
    pub is_supplier: bool,
    pub address: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    /// Forme normalisée `"CHE109322551"`. Le frontend la formate pour l'affichage.
    pub ide_number: Option<String>,
    pub default_payment_terms: Option<String>,
    pub active: bool,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<Contact> for ContactResponse {
    fn from(c: Contact) -> Self {
        Self {
            id: c.id,
            company_id: c.company_id,
            contact_type: c.contact_type,
            name: c.name,
            is_client: c.is_client,
            is_supplier: c.is_supplier,
            address: c.address,
            email: c.email,
            phone: c.phone,
            // Copie directe — déjà normalisée en base via CheNumber::new().as_str()
            // au moment de l'INSERT. Pas de re-parse CheNumber ici.
            ide_number: c.ide_number,
            default_payment_terms: c.default_payment_terms,
            active: c.active,
            version: c.version,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Validation email caractère-par-caractère (pas de crate `regex` dans le workspace).
///
/// Format minimal RFC 5322 simplifié : `{local}@{domain}.{tld}` sans whitespace.
/// **Limite connue v0.1** : `user@@domain.ch` est faussement accepté (dette
/// intentionnelle documentée dans la spec 4.1).
fn is_valid_email_simple(s: &str) -> bool {
    let Some(at_pos) = s.find('@') else {
        return false;
    };
    let (local, rest) = s.split_at(at_pos);
    let domain = &rest[1..];
    !local.is_empty()
        && !local.contains(char::is_whitespace)
        && !domain.is_empty()
        && !domain.contains(char::is_whitespace)
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.contains("..")
}

/// Récupère la company courante (v0.1 single-company).
/// Normalise un `Option<String>` en retirant les whitespace et trim ;
/// retourne `None` si vide après trim.
fn normalize_optional(s: Option<String>) -> Option<String> {
    s.and_then(|v| {
        let t = v.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    })
}

/// Valide + normalise un IDE optionnel via `CheNumber`.
/// Retourne la forme normalisée (12 chars `"CHE123456789"`).
fn validate_optional_ide(raw: Option<String>) -> Result<Option<String>, AppError> {
    let Some(s) = raw else { return Ok(None) };
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let che = CheNumber::new(trimmed).map_err(|_| {
        AppError::Validation("Numéro IDE suisse invalide (format ou checksum)".into())
    })?;
    Ok(Some(che.as_str().to_string()))
}

/// Validation commune des champs métier (create + update).
struct ValidatedFields {
    contact_type: ContactType,
    name: String,
    is_client: bool,
    is_supplier: bool,
    address: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    ide_number: Option<String>,
    default_payment_terms: Option<String>,
}

#[allow(clippy::too_many_arguments)]
fn validate_common(
    contact_type: ContactType,
    name: String,
    is_client: bool,
    is_supplier: bool,
    address: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    ide_number: Option<String>,
    default_payment_terms: Option<String>,
) -> Result<ValidatedFields, AppError> {
    let trimmed_name = name.trim().to_string();
    if trimmed_name.is_empty() {
        return Err(AppError::Validation("Le nom est obligatoire".into()));
    }
    if trimmed_name.chars().count() > MAX_NAME_LEN {
        return Err(AppError::Validation(format!(
            "Le nom doit faire au plus {MAX_NAME_LEN} caractères"
        )));
    }

    let address = normalize_optional(address);
    if let Some(ref a) = address {
        if a.chars().count() > MAX_ADDRESS_LEN {
            return Err(AppError::Validation(format!(
                "L'adresse doit faire au plus {MAX_ADDRESS_LEN} caractères"
            )));
        }
    }

    let email = normalize_optional(email);
    if let Some(ref e) = email {
        if e.chars().count() > MAX_EMAIL_LEN {
            return Err(AppError::Validation(format!(
                "L'email doit faire au plus {MAX_EMAIL_LEN} caractères"
            )));
        }
        if !is_valid_email_simple(e) {
            return Err(AppError::Validation("Format d'email invalide".into()));
        }
    }

    let phone = normalize_optional(phone);
    if let Some(ref p) = phone {
        if p.chars().count() > MAX_PHONE_LEN {
            return Err(AppError::Validation(format!(
                "Le téléphone doit faire au plus {MAX_PHONE_LEN} caractères"
            )));
        }
    }

    let default_payment_terms = normalize_optional(default_payment_terms);
    if let Some(ref t) = default_payment_terms {
        if t.chars().count() > MAX_PAYMENT_TERMS_LEN {
            return Err(AppError::Validation(format!(
                "Les conditions de paiement doivent faire au plus {MAX_PAYMENT_TERMS_LEN} caractères"
            )));
        }
    }

    let ide_number = validate_optional_ide(ide_number)?;

    Ok(ValidatedFields {
        contact_type,
        name: trimmed_name,
        is_client,
        is_supplier,
        address,
        email,
        phone,
        ide_number,
        default_payment_terms,
    })
}

/// Intercepte les `UniqueConstraintViolation` portant sur la contrainte
/// `uq_contacts_company_ide` et remappe vers le code client dédié
/// `IDE_ALREADY_EXISTS`. Sinon propage tel quel.
///
/// **Note** : on ne matche que le **nom de contrainte** (`uq_contacts_company_ide`),
/// pas le nom de colonne (`ide_number`) — le format du message d'erreur
/// MariaDB peut varier entre versions (10.x vs 11.x, schéma préfixé ou non).
fn map_contact_error(err: DbError) -> AppError {
    if let DbError::UniqueConstraintViolation(ref m) = err {
        if m.contains("uq_contacts_company_ide") {
            return AppError::IdeAlreadyExists("Un contact avec ce numéro IDE existe déjà".into());
        }
    }
    AppError::from(err)
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/v1/contacts — liste paginée avec filtres.
/// Story 6.2: Scoped by current_user.company_id.
pub async fn list_contacts(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<ListContactsQuery>,
) -> Result<Json<ListResponse<ContactResponse>>, AppError> {
    // Validate company exists (defensive: company_id staleness window)
    let _ = get_company_for(&current_user, &state.pool).await?;

    let limit = params
        .limit
        .unwrap_or(DEFAULT_LIST_LIMIT)
        .clamp(1, MAX_LIST_LIMIT);
    let offset = params.offset.unwrap_or(0).max(0);

    let query = ContactListQuery {
        search: params.search,
        contact_type: params.contact_type,
        is_client: params.is_client,
        is_supplier: params.is_supplier,
        include_archived: params.include_archived,
        sort_by: params.sort_by.unwrap_or_default(),
        sort_direction: params.sort_direction.unwrap_or(SortDirection::Asc),
        limit,
        offset,
    };

    let result =
        contacts::list_by_company_paginated(&state.pool, current_user.company_id, query).await?;

    Ok(Json(ListResponse {
        items: result
            .items
            .into_iter()
            .map(ContactResponse::from)
            .collect(),
        total: result.total,
        limit: result.limit,
        offset: result.offset,
    }))
}

/// GET /api/v1/contacts/{id} — retourne un contact par ID.
/// Story 6.2: Scoped by current_user.company_id.
pub async fn get_contact(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<ContactResponse>, AppError> {
    // Story 6.2: Multi-tenant scoping via find_by_id_in_company
    let contact = contacts::find_by_id_in_company(&state.pool, id, current_user.company_id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;

    Ok(Json(ContactResponse::from(contact)))
}

/// POST /api/v1/contacts — crée un contact.
/// Story 6.2: Scoped by current_user.company_id.
pub async fn create_contact(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateContactRequest>,
) -> Result<(StatusCode, Json<ContactResponse>), AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;

    let v = validate_common(
        req.contact_type,
        req.name,
        req.is_client,
        req.is_supplier,
        req.address,
        req.email,
        req.phone,
        req.ide_number,
        req.default_payment_terms,
    )?;

    let new = NewContact {
        company_id: company.id,
        contact_type: v.contact_type,
        name: v.name,
        is_client: v.is_client,
        is_supplier: v.is_supplier,
        address: v.address,
        email: v.email,
        phone: v.phone,
        ide_number: v.ide_number,
        default_payment_terms: v.default_payment_terms,
    };

    let contact = contacts::create(&state.pool, current_user.user_id, new)
        .await
        .map_err(map_contact_error)?;

    Ok((StatusCode::CREATED, Json(ContactResponse::from(contact))))
}

/// PUT /api/v1/contacts/{id} — met à jour un contact.
/// Story 6.2: Scoped by current_user.company_id.
pub async fn update_contact(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateContactRequest>,
) -> Result<Json<ContactResponse>, AppError> {
    // Story 6.2: Multi-tenant scoping via find_by_id_in_company (IDOR check)
    let _existing = contacts::find_by_id_in_company(&state.pool, id, current_user.company_id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;

    let v = validate_common(
        req.contact_type,
        req.name,
        req.is_client,
        req.is_supplier,
        req.address,
        req.email,
        req.phone,
        req.ide_number,
        req.default_payment_terms,
    )?;

    let changes = ContactUpdate {
        contact_type: v.contact_type,
        name: v.name,
        is_client: v.is_client,
        is_supplier: v.is_supplier,
        address: v.address,
        email: v.email,
        phone: v.phone,
        ide_number: v.ide_number,
        default_payment_terms: v.default_payment_terms,
    };

    let contact = contacts::update(&state.pool, id, req.version, current_user.user_id, changes)
        .await
        .map_err(map_contact_error)?;

    Ok(Json(ContactResponse::from(contact)))
}

/// PUT /api/v1/contacts/{id}/archive — archive un contact.
/// Story 6.2: Scoped by current_user.company_id via find_by_id_in_company.
pub async fn archive_contact(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<ArchiveContactRequest>,
) -> Result<Json<ContactResponse>, AppError> {
    // Story 6.2: Multi-tenant scoping via find_by_id_in_company (IDOR check)
    let _existing = contacts::find_by_id_in_company(&state.pool, id, current_user.company_id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;

    let contact = contacts::archive(&state.pool, id, req.version, current_user.user_id).await?;
    Ok(Json(ContactResponse::from(contact)))
}

// ---------------------------------------------------------------------------
// Tests unitaires
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_valid_cases() {
        assert!(is_valid_email_simple("user@domain.ch"));
        assert!(is_valid_email_simple("user@subdomain.domain.com"));
        assert!(is_valid_email_simple("a@b.co"));
    }

    #[test]
    fn email_invalid_cases() {
        assert!(!is_valid_email_simple("no-at-sign.com"));
        assert!(!is_valid_email_simple("@no-local.com"));
        assert!(!is_valid_email_simple("user@.ch"));
        assert!(!is_valid_email_simple("user@ch"));
        assert!(!is_valid_email_simple("user name@domain.ch"));
        assert!(!is_valid_email_simple("user@domain..ch"));
        assert!(!is_valid_email_simple(""));
        assert!(!is_valid_email_simple("user@"));
    }

    #[test]
    fn validate_ide_normalizes_with_separators() {
        let result = validate_optional_ide(Some("CHE-109.322.551".to_string())).unwrap();
        assert_eq!(result, Some("CHE109322551".to_string()));
    }

    #[test]
    fn validate_ide_normalizes_mwst_suffix() {
        let result = validate_optional_ide(Some("CHE-109.322.551 MWST".to_string())).unwrap();
        assert_eq!(result, Some("CHE109322551".to_string()));
    }

    #[test]
    fn validate_ide_rejects_invalid_checksum() {
        let err = validate_optional_ide(Some("CHE-109.322.552".to_string()));
        assert!(err.is_err());
    }

    #[test]
    fn validate_ide_accepts_valid_zero_checksum() {
        // CHE-000.000.000 est VALIDE (modulo 11 : 0).
        let result = validate_optional_ide(Some("CHE-000.000.000".to_string())).unwrap();
        assert_eq!(result, Some("CHE000000000".to_string()));
    }

    #[test]
    fn validate_ide_empty_is_none() {
        assert_eq!(validate_optional_ide(Some("".to_string())).unwrap(), None);
        assert_eq!(
            validate_optional_ide(Some("   ".to_string())).unwrap(),
            None
        );
        assert_eq!(validate_optional_ide(None).unwrap(), None);
    }

    #[test]
    fn map_contact_error_ide_unique_maps_to_dedicated_variant() {
        let err = DbError::UniqueConstraintViolation("uq_contacts_company_ide".into());
        let app_err = map_contact_error(err);
        match app_err {
            AppError::IdeAlreadyExists(_) => {}
            other => panic!("expected IdeAlreadyExists, got {other:?}"),
        }
    }

    #[test]
    fn map_contact_error_other_unique_maps_to_generic_conflict() {
        let err = DbError::UniqueConstraintViolation("some_other_constraint".into());
        let app_err = map_contact_error(err);
        // Doit être mappé en AppError::Database (pas IdeAlreadyExists).
        match app_err {
            AppError::Database(_) => {}
            other => panic!("expected Database, got {other:?}"),
        }
    }

    #[test]
    fn normalize_optional_trims_and_collapses_empty_to_none() {
        assert_eq!(
            normalize_optional(Some("  hello  ".into())),
            Some("hello".into())
        );
        assert_eq!(normalize_optional(Some("   ".into())), None);
        assert_eq!(normalize_optional(Some("".into())), None);
        assert_eq!(normalize_optional(None), None);
    }
}
