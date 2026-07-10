//! Routes company — lecture de la configuration de l'organisation,
//! et mise à jour de l'e-mail de contact (Story 20-3b1, Reply-To des
//! e-mails métier).

use axum::Extension;
use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};

use kesh_db::entities::{BankAccount, Company, CompanyUpdate};
use kesh_db::repositories::{bank_accounts, companies};

use crate::AppState;
use crate::errors::AppError;
use crate::helpers::get_company_for;
use crate::middleware::auth::CurrentUser;

/// Réponse JSON pour la company courante + comptes bancaires.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyCurrentResponse {
    pub company: CompanyJson,
    pub bank_accounts: Vec<BankAccountJson>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyJson {
    pub id: i64,
    pub name: String,
    /// Prénom / nom (#213) — renseignés si la société est une personne physique.
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    /// Chaîne d'affichage dérivée (#213).
    pub address: String,
    /// Adresse structurée (source de vérité éditable, #213).
    pub address_structured: crate::address_input::StructuredAddressInput,
    pub ide_number: Option<String>,
    pub org_type: String,
    pub accounting_language: String,
    pub instance_language: String,
    /// E-mail de contact de la société (Story 20-3b1) — Reply-To des e-mails
    /// métier. `null` = non renseigné (Reply-To omis à l'envoi).
    pub email: Option<String>,
    /// Verrou optimiste — requis par `PUT /companies/current/email`.
    pub version: i32,
}

impl From<Company> for CompanyJson {
    fn from(c: Company) -> Self {
        let address_structured =
            crate::address_input::StructuredAddressInput::from(&c.structured_address());
        Self {
            id: c.id,
            name: c.name,
            first_name: c.first_name,
            last_name: c.last_name,
            address_structured,
            address: c.address,
            ide_number: c.ide_number,
            org_type: c.org_type.as_str().to_string(),
            accounting_language: c.accounting_language.as_str().to_string(),
            instance_language: c.instance_language.as_str().to_string(),
            email: c.email,
            version: c.version,
        }
    }
}

/// Payload de `PUT /api/v1/companies/current/email` (Story 20-3b1).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCompanyEmailRequest {
    /// Nouvel e-mail de contact. `null`/vide = effacer (Reply-To omis).
    #[serde(default)]
    pub email: Option<String>,
    pub version: i32,
}

/// PUT /api/v1/companies/current/email — met à jour l'e-mail de contact de
/// la société (Admin-only, Story 20-3b1). Endpoint dédié minimal : il
/// n'existe aucune route générique d'update company (seul l'onboarding
/// `set_coordinates` touche les coordonnées) ; les autres champs restent
/// donc hors scope ici. Réutilise `companies::update` (verrou optimiste +
/// no-op KF-004) avec un `CompanyUpdate` reconstruit depuis l'état courant.
pub async fn update_company_email(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<UpdateCompanyEmailRequest>,
) -> Result<Json<CompanyJson>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;

    let email = match req.email.as_deref().map(str::trim) {
        None | Some("") => None,
        Some(e) => {
            if !crate::routes::contacts::is_valid_email_simple(e) {
                return Err(AppError::Validation(crate::errors::t(
                    "error-company-email-invalid",
                    "L'adresse e-mail de la société n'est pas valide.",
                )));
            }
            Some(e.to_string())
        }
    };

    let changes = CompanyUpdate {
        name: company.name.clone(),
        first_name: company.first_name.clone(),
        last_name: company.last_name.clone(),
        address_structured: company.structured_address(),
        ide_number: company.ide_number.clone(),
        org_type: company.org_type,
        accounting_language: company.accounting_language,
        instance_language: company.instance_language,
        email,
    };

    let updated = companies::update(&state.pool, company.id, req.version, changes).await?;
    Ok(Json(CompanyJson::from(updated)))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BankAccountJson {
    pub id: i64,
    pub bank_name: String,
    pub iban: String,
    pub qr_iban: Option<String>,
    pub is_primary: bool,
    /// Story 8-5a-zero — compte du plan comptable lié (None = non configuré).
    pub journal_account_id: Option<i64>,
}

impl From<BankAccount> for BankAccountJson {
    fn from(b: BankAccount) -> Self {
        Self {
            id: b.id,
            bank_name: b.bank_name,
            iban: b.iban,
            qr_iban: b.qr_iban,
            is_primary: b.is_primary,
            journal_account_id: b.journal_account_id,
        }
    }
}

/// GET /api/v1/companies/current — retourne la company courante + bank accounts.
/// Story 6.2: Scoped by CurrentUser.company_id (KF-002 fix).
pub async fn get_current(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<CompanyCurrentResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let accounts =
        bank_accounts::list_by_company(&state.pool, company.id, /*include_archived=*/ false)
            .await?;

    Ok(Json(CompanyCurrentResponse {
        company: company.into(),
        bank_accounts: accounts.into_iter().map(Into::into).collect(),
    }))
}
