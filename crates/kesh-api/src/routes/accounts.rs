//! Routes CRUD pour les comptes du plan comptable.

use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use kesh_db::entities::account::{Account, AccountRole, AccountType, AccountUpdate, NewAccount};
use kesh_db::errors::DbError;
use kesh_db::repositories::accounts;

use crate::AppState;
use crate::errors::AppError;
use crate::helpers::get_company_for;
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
    /// Rôle métier explicite (Story 14-3a) — optionnel, `null` par défaut.
    #[serde(default)]
    pub role: Option<AccountRole>,
    /// Postabilité (Story 14-3a) — optionnel, `true` par défaut.
    #[serde(default = "default_true")]
    pub postable: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
/// Sémantique **full-replace** — `role` et `postable` sont **obligatoires**.
///
/// `name` et `account_type` le sont déjà : omettre `name` produit un 400 depuis
/// toujours. Rendre les deux nouveaux champs optionnels aurait signifié qu'un
/// client corrigeant un libellé efface silencieusement le rôle du compte (ou
/// rende postable un compte qui ne l'était pas). Une donnée perdue en silence
/// est pire qu'un 400 explicite. Pour retirer un rôle, envoyer `role: null`.
pub struct UpdateAccountRequest {
    pub name: String,
    pub account_type: AccountType,
    pub role: Option<AccountRole>,
    pub postable: bool,
    pub version: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveAccountRequest {
    pub version: i32,
}

/// Story 14-3a (#269) — réactivation d'un compte archivé.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactivateAccountRequest {
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
    /// Rôle métier explicite, `null` si aucun (Story 14-3a).
    pub role: Option<AccountRole>,
    /// Postabilité — **indicatif en 14-3a**, appliqué à la saisie par 14-3b.
    pub postable: bool,
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
            role: a.role,
            postable: a.postable,
            version: a.version,
            created_at: a.created_at,
            updated_at: a.updated_at,
        }
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/v1/accounts — liste les comptes de la company courante.
/// Story 6.2: Scoped by current_user.company_id.
pub async fn list_accounts(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<ListAccountsQuery>,
) -> Result<Json<Vec<AccountResponse>>, AppError> {
    // Validate company exists (defensive: company_id staleness window)
    let _ = get_company_for(&current_user, &state.pool).await?;

    let list = accounts::list_by_company(
        &state.pool,
        current_user.company_id,
        params.include_archived,
    )
    .await?;
    Ok(Json(list.into_iter().map(AccountResponse::from).collect()))
}

/// POST /api/v1/accounts — crée un compte.
/// Story 6.2: Scoped by current_user.company_id.
pub async fn create_account(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateAccountRequest>,
) -> Result<(axum::http::StatusCode, Json<AccountResponse>), AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;

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
        role: req.role,
        postable: req.postable,
    };

    let account = accounts::create(&state.pool, current_user.user_id, new).await?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(AccountResponse::from(account)),
    ))
}

/// PUT /api/v1/accounts/{id} — modifie un compte (nom et type).
/// Story 6.2: Validate company exists (defensive: company_id staleness window).
pub async fn update_account(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateAccountRequest>,
) -> Result<Json<AccountResponse>, AppError> {
    // Validate company exists (defensive: company_id staleness window)
    let _ = get_company_for(&current_user, &state.pool).await?;

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
        role: req.role,
        postable: req.postable,
    };

    let account =
        accounts::update(&state.pool, id, req.version, current_user.user_id, changes).await?;
    Ok(Json(AccountResponse::from(account)))
}

/// PUT /api/v1/accounts/{id}/archive — archive un compte.
/// Story 6.2: Scoped by current_user.company_id via find_by_id_in_company.
pub async fn archive_account(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<ArchiveAccountRequest>,
) -> Result<Json<AccountResponse>, AppError> {
    // Verify account belongs to current user's company (IDOR check)
    let _existing = accounts::find_by_id_in_company(&state.pool, id, current_user.company_id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;

    let account = accounts::archive(&state.pool, id, req.version, current_user.user_id).await?;
    Ok(Json(AccountResponse::from(account)))
}

/// PUT /api/v1/accounts/{id}/reactivate — réactive un compte archivé (#269).
///
/// Verbe `PUT` par symétrie **locale** avec `PUT /accounts/{id}/archive` (la
/// feature Projets utilise `POST /unarchive` ; on privilégie ici la cohérence
/// interne de la ressource `accounts`).
///
/// Refus possibles, tous en 409 : parent archivé, rôle singleton repris depuis
/// l'archivage, conflit de version. Réactiver un compte déjà actif est un no-op.
pub async fn reactivate_account(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<ReactivateAccountRequest>,
) -> Result<Json<AccountResponse>, AppError> {
    // Verify account belongs to current user's company (IDOR check)
    let _existing = accounts::find_by_id_in_company(&state.pool, id, current_user.company_id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;

    let account = accounts::reactivate(&state.pool, id, req.version, current_user.user_id).await?;
    Ok(Json(AccountResponse::from(account)))
}
