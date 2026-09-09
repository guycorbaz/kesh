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
    /// Téléphone et site web (Story 16-3a, #151), rendus sur le PDF de facture.
    ///
    /// ⚠️ Ce DTO est un miroir **écrit à la main** : aucun compilateur ne le
    /// vérifie contre `Company`. Un champ oublié ici est stocké en base, rendu
    /// sur le PDF, et **invisible dans l'écran de réglages, pour toujours**.
    pub phone: Option<String>,
    pub website: Option<String>,
    /// Borne **inclusive** du verrou de période (Story 24-4c, #380).
    /// `null` = aucun verrou. L'écran s'en sert pour le bandeau et pour le
    /// `min` du champ date de saisie.
    pub books_locked_through: Option<chrono::NaiveDate>,
    /// Verrou optimiste — requis par les routes `PUT /companies/current/*`.
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
            phone: c.phone,
            website: c.website,
            books_locked_through: c.books_locked_through,
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
        // Story 16-3a (#151) — report à l'identique : cette route ne change
        // QUE l'e-mail, et `companies::update` est un full-replace. Omettre ces
        // deux lignes effacerait silencieusement le téléphone et le site web à
        // chaque modification d'e-mail.
        phone: company.phone.clone(),
        website: company.website.clone(),
    };

    let updated = companies::update(&state.pool, company.id, req.version, changes).await?;
    Ok(Json(CompanyJson::from(updated)))
}

/// Longueurs maximales — alignées sur le schéma (`VARCHAR(50)` / `VARCHAR(255)`),
/// pas dérivées de la largeur du bloc PDF. Valider plus court que la colonne
/// rendrait le message d'erreur incohérent avec ce que la base accepte ;
/// valider plus long produirait une troncature MariaDB silencieuse.
const MAX_PHONE_LEN: usize = 50;
const MAX_WEBSITE_LEN: usize = 255;

/// Payload de `PUT /api/v1/companies/current/contact-details` (Story 16-3a, #151).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCompanyContactDetailsRequest {
    /// Téléphone de contact. `null`/vide = effacer (ligne omise du PDF).
    ///
    /// ⚠️ **Une clé ABSENTE efface aussi** : `#[serde(default)]` la rend
    /// indistinguable de `null`, et `companies::update` est un full-replace.
    /// Un client qui n'envoie qu'un des deux champs efface l'autre, en `200`.
    /// Épinglé par `an_omitted_field_clears_it_just_like_null`.
    #[serde(default)]
    pub phone: Option<String>,
    /// Site web. `null`/vide = effacer — et une clé absente aussi, cf. `phone`.
    #[serde(default)]
    pub website: Option<String>,
    pub version: i32,
}

/// Normalise un champ de coordonnée : `None`/vide ⇒ `None`, sinon trim + borne.
///
/// **D5 — aucune validation de FORMAT.** Les formats de téléphone
/// internationaux sont innombrables, et un utilisateur écrira aussi bien
/// `example.ch` que `https://example.ch` : refuser une saisie sur un champ
/// purement décoratif coûterait plus que ça ne rapporte. Seule la longueur est
/// bornée, parce qu'elle, la base la refuserait.
fn normalize_contact_field(
    raw: Option<&str>,
    max: usize,
    key: &'static str,
    fallback: &str,
) -> Result<Option<String>, AppError> {
    match raw.map(str::trim) {
        None | Some("") => Ok(None),
        Some(v) => {
            if v.chars().count() > max {
                return Err(AppError::Validation(crate::errors::t(key, fallback)));
            }
            Ok(Some(v.to_string()))
        }
    }
}

/// PUT /api/v1/companies/current/contact-details — téléphone et site web de
/// la société (Story 16-3a, #151), rendus sur le PDF de facture.
///
/// **Endpoint dédié, sur le patron exact d'`update_company_email`** (D4) : il
/// n'existe aucune route générique d'update company, et `update_company_coordinates`
/// (onboarding) pose `is_stub = FALSE` inconditionnellement avec un appelant
/// unique — deux invariants documentés qu'on ne touche pas.
///
/// ⚠️ Comme sa jumelle, cette route **reconstruit un `CompanyUpdate` complet**
/// depuis l'état courant : `companies::update` est un full-replace. Tout champ
/// non reporté serait **effacé en silence**.
pub async fn update_company_contact_details(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<UpdateCompanyContactDetailsRequest>,
) -> Result<Json<CompanyJson>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;

    let phone = normalize_contact_field(
        req.phone.as_deref(),
        MAX_PHONE_LEN,
        "error-company-phone-too-long",
        "Le numéro de téléphone de la société est trop long (50 caractères au plus).",
    )?;
    let website = normalize_contact_field(
        req.website.as_deref(),
        MAX_WEBSITE_LEN,
        "error-company-website-too-long",
        "L'adresse du site web de la société est trop longue (255 caractères au plus).",
    )?;

    let changes = CompanyUpdate {
        name: company.name.clone(),
        first_name: company.first_name.clone(),
        last_name: company.last_name.clone(),
        address_structured: company.structured_address(),
        ide_number: company.ide_number.clone(),
        org_type: company.org_type,
        accounting_language: company.accounting_language,
        instance_language: company.instance_language,
        // Reporté à l'identique — cette route ne touche PAS l'e-mail.
        email: company.email.clone(),
        phone,
        website,
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

// ---------------------------------------------------------------------------
// Story 24-4c (#380) — le verrou de période
// ---------------------------------------------------------------------------

/// Payload de `POST /api/v1/companies/current/books-lock`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockBooksRequest {
    /// Borne **inclusive** : les écritures datées de ce jour ou avant sont
    /// refusées. Doit être strictement antérieure à aujourd'hui, et strictement
    /// postérieure à la borne courante s'il y en a une.
    pub through: chrono::NaiveDate,
}

/// Payload de `POST /api/v1/companies/current/books-lock/release`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockBooksRequest {
    /// Nouvelle borne, ou `null` pour retirer le verrou entièrement.
    #[serde(default)]
    pub through: Option<chrono::NaiveDate>,
    /// ⛔ Obligatoire et non blanc : déverrouiller défait une garantie, et le
    /// journal d'audit doit pouvoir dire pourquoi.
    pub motif: String,
}

/// POST /api/v1/companies/current/books-lock — pose ou **avance** le verrou de
/// période (Admin + Comptable, Story 24-4c).
///
/// ⚠️ Ce point d'entrée ne peut PAS reculer la borne : le repository refuse
/// toute date antérieure ou égale à la borne courante. Sans cette garde de
/// **valeur**, la garde de **rôle** serait contournable par le verbe.
pub async fn lock_company_books(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<LockBooksRequest>,
) -> Result<Json<CompanyJson>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let updated =
        companies::lock_books(&state.pool, current_user.user_id, company.id, req.through).await?;
    Ok(Json(updated.into()))
}

/// POST /api/v1/companies/current/books-lock/release — **recule ou retire** le
/// verrou de période (Admin uniquement, motif obligatoire, Story 24-4c).
pub async fn unlock_company_books(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<UnlockBooksRequest>,
) -> Result<Json<CompanyJson>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let updated = companies::unlock_books(
        &state.pool,
        current_user.user_id,
        company.id,
        req.through,
        req.motif,
    )
    .await?;
    Ok(Json(updated.into()))
}
