//! Routes CRUD pour les comptes du plan comptable.

use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use kesh_db::entities::account::{Account, AccountType, AccountUpdate, NewAccount};
use kesh_db::repositories::{accounts, companies};

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAccountsQuery {
    #[serde(default)]
    pub include_archived: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountRequest {
    pub number: String,
    pub name: String,
    pub account_type: AccountType,
    pub parent_id: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccountRequest {
    pub name: String,
    pub account_type: AccountType,
    pub version: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveAccountRequest {
    pub version: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountResponse {
    pub id: i64,
    pub company_id: i64,
    pub number: String,
    pub name: String,
    pub account_type: AccountType,
    pub parent_id: Option<i64>,
    pub active: bool,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<Account> for AccountResponse {
    fn from(a: Account) -> Self {
        Self {
            id: a.id,
            company_id: a.company_id,
            number: a.number,
            name: a.name,
            account_type: a.account_type,
            parent_id: a.parent_id,
            active: a.active,
            version: a.version,
            created_at: a.created_at,
            updated_at: a.updated_at,
        }
    }
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

async fn get_company(state: &AppState) -> Result<kesh_db::entities::Company, AppError> {
    let list = companies::list(&state.pool, 1, 0).await?;
    list.into_iter()
        .next()
        .ok_or_else(|| AppError::Internal("Aucune company en base".into()))
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/v1/accounts — liste les comptes de la company courante.
pub async fn list_accounts(
    State(state): State<AppState>,
    Query(params): Query<ListAccountsQuery>,
) -> Result<Json<Vec<AccountResponse>>, AppError> {
    let company = get_company(&state).await?;
    let list = accounts::list_by_company(&state.pool, company.id, params.include_archived).await?;
    Ok(Json(list.into_iter().map(AccountResponse::from).collect()))
}

/// POST /api/v1/accounts — crée un compte.
pub async fn create_account(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateAccountRequest>,
) -> Result<(axum::http::StatusCode, Json<AccountResponse>), AppError> {
    let company = get_company(&state).await?;

    let trimmed_number = req.number.trim().to_string();
    let trimmed_name = req.name.trim().to_string();

    if trimmed_number.is_empty() {
        return Err(AppError::Validation("number must not be empty".into()));
    }
    if trimmed_number.len() > 10 {
        return Err(AppError::Validation(
            "number must not exceed 10 characters".into(),
        ));
    }
    if trimmed_name.is_empty() {
        return Err(AppError::Validation("name must not be empty".into()));
    }
    if trimmed_name.len() > 255 {
        return Err(AppError::Validation(
            "name must not exceed 255 characters".into(),
        ));
    }

    // Valider que le parent existe et est actif
    if let Some(pid) = req.parent_id {
        let parent = accounts::find_by_id(&state.pool, pid).await?;
        match parent {
            None => return Err(AppError::Validation("parent account not found".into())),
            Some(p) if !p.active => {
                return Err(AppError::Validation("parent account is archived".into()));
            }
            _ => {}
        }
    }

    let new = NewAccount {
        company_id: company.id,
        number: trimmed_number,
        name: trimmed_name,
        account_type: req.account_type,
        parent_id: req.parent_id,
    };

    let account = accounts::create(&state.pool, current_user.user_id, new).await?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(AccountResponse::from(account)),
    ))
}

/// PUT /api/v1/accounts/{id} — modifie un compte (nom et type).
pub async fn update_account(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateAccountRequest>,
) -> Result<Json<AccountResponse>, AppError> {
    let trimmed_name = req.name.trim().to_string();
    if trimmed_name.is_empty() {
        return Err(AppError::Validation("name must not be empty".into()));
    }
    if trimmed_name.len() > 255 {
        return Err(AppError::Validation(
            "name must not exceed 255 characters".into(),
        ));
    }

    let changes = AccountUpdate {
        name: trimmed_name,
        account_type: req.account_type,
    };

    let account =
        accounts::update(&state.pool, id, req.version, current_user.user_id, changes).await?;
    Ok(Json(AccountResponse::from(account)))
}

/// PUT /api/v1/accounts/{id}/archive — archive un compte.
pub async fn archive_account(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<ArchiveAccountRequest>,
) -> Result<Json<AccountResponse>, AppError> {
    let account = accounts::archive(&state.pool, id, req.version, current_user.user_id).await?;
    Ok(Json(AccountResponse::from(account)))
}
