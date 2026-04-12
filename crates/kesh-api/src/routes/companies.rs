//! Routes company — lecture de la configuration de l'organisation.

use axum::Json;
use axum::extract::State;
use serde::Serialize;

use kesh_db::entities::{BankAccount, Company};
use kesh_db::repositories::{bank_accounts, companies};

use crate::AppState;
use crate::errors::AppError;

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
    pub address: String,
    pub ide_number: Option<String>,
    pub org_type: String,
    pub accounting_language: String,
    pub instance_language: String,
}

impl From<Company> for CompanyJson {
    fn from(c: Company) -> Self {
        Self {
            id: c.id,
            name: c.name,
            address: c.address,
            ide_number: c.ide_number,
            org_type: c.org_type.as_str().to_string(),
            accounting_language: c.accounting_language.as_str().to_string(),
            instance_language: c.instance_language.as_str().to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BankAccountJson {
    pub id: i64,
    pub bank_name: String,
    pub iban: String,
    pub qr_iban: Option<String>,
    pub is_primary: bool,
}

impl From<BankAccount> for BankAccountJson {
    fn from(b: BankAccount) -> Self {
        Self {
            id: b.id,
            bank_name: b.bank_name,
            iban: b.iban,
            qr_iban: b.qr_iban,
            is_primary: b.is_primary,
        }
    }
}

/// GET /api/v1/companies/current — retourne la company courante + bank accounts.
pub async fn get_current(
    State(state): State<AppState>,
) -> Result<Json<CompanyCurrentResponse>, AppError> {
    let list = companies::list(&state.pool, 1, 0).await?;
    let company = list
        .into_iter()
        .next()
        .ok_or(AppError::Database(kesh_db::errors::DbError::NotFound))?;

    let accounts = bank_accounts::list_by_company(&state.pool, company.id).await?;

    Ok(Json(CompanyCurrentResponse {
        company: company.into(),
        bank_accounts: accounts.into_iter().map(Into::into).collect(),
    }))
}
