//! Routes company — lecture de la configuration de l'organisation,
//! et mise à jour de l'e-mail de contact (Story 20-3b1, Reply-To des
//! e-mails métier), des coordonnées (16-3a), de l'adresse (33-5, #909) et du
//! réglage multi-devises (37-1).

use axum::Extension;
use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};

use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::{
    BankAccount, BankAccountKind, Company, CompanyUpdate, CurrencySettingsUpdate,
};
use kesh_db::errors::map_db_error;
use kesh_db::repositories::{audit_log, bank_accounts, companies};

use crate::AppState;
use crate::dates_recues::{borner, borner_opt};
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
    /// Faux si le pays enregistré n'est pas un code ISO connu (Story 33-5, #909) :
    /// la QR-facture ne se produit pas, et l'écran le signale sans recopier la
    /// liste. Un pays vide (société d'avant les composants) n'est pas inconnu.
    pub address_country_known: bool,
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
    /// Mode multi-devises de la société (Story 37-1, AC-2). Toujours présent :
    /// ce n'est pas un champ en devise (C-37-9).
    pub multi_currency_enabled: bool,
    /// Compte des gains de change de la société (Story 37-1) ; `null` si non
    /// désigné. Toujours présent.
    pub fx_gain_account_id: Option<i64>,
    /// Compte des pertes de change de la société (Story 37-1). Toujours présent.
    pub fx_loss_account_id: Option<i64>,
    /// Compte de conversion entre devises (Story 37-1). Toujours présent.
    pub fx_transit_account_id: Option<i64>,
    /// Devise principale de la comptabilité (Story 37-1) : la constante
    /// [`kesh_core::currency::BASE_CURRENCY`] (`"CHF"`) ; elle ne change pas.
    pub base_currency: &'static str,
    /// Verrou optimiste — requis par les routes `PUT /companies/current/*`.
    pub version: i32,
}

impl From<Company> for CompanyJson {
    fn from(c: Company) -> Self {
        let address_structured =
            crate::address_input::StructuredAddressInput::from(&c.structured_address());
        // Sans la normalisation d'`is_known_country` : le PDF transmet la colonne telle
        // quelle, et refuse `ch` ; l'avis doit dire la même chose que lui.
        let address_country_known = c.address_country.trim().is_empty()
            || kesh_qrbill::is_iso_3166_alpha2(&c.address_country);
        Self {
            address_country_known,
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
            multi_currency_enabled: c.multi_currency_enabled,
            fx_gain_account_id: c.fx_gain_account_id,
            fx_loss_account_id: c.fx_loss_account_id,
            fx_transit_account_id: c.fx_transit_account_id,
            base_currency: kesh_core::currency::BASE_CURRENCY,
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

    // Story 25-1b (AC 4, 8, 9) — `details_json` ne porte QUE les champs que cette
    // route touche : `companies::update` est un full-replace et les handlers
    // reconstruisent tous les champs depuis l'état courant, si bien qu'un
    // `{before, after}` complet dirait que quinze champs ont changé quand un seul
    // a bougé.
    //
    // ⚠️ C'est ici que la trace vaut le plus : une clé absente vaut `null` et
    // EFFACE. Un effacement par omission ne laissait aucune trace.
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(map_db_error(e)))?;
    let updated = companies::update_in_tx(&mut tx, company.id, req.version, changes).await?;
    // AC 8 — le repository court-circuite le no-op sans incrémenter `version`.
    if updated.version != company.version {
        audit_log::insert_in_tx(
            &mut tx,
            // `::user` est sûr : la route est dans `admin_routes`, seul bloc
            // porteur de `require_not_pat`.
            NewAuditLogEntry::user(
                current_user.user_id,
                "company.updated",
                "company",
                company.id,
                Some(serde_json::json!({
                    "before": { "email": company.email },
                    "after": { "email": updated.email },
                })),
            ),
        )
        .await?;
    }
    tx.commit()
        .await
        .map_err(|e| AppError::Database(map_db_error(e)))?;
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
/// n'existe aucune route générique d'update company, et
/// `update_company_coordinates_in_tx` (onboarding) lève le drapeau provisoire
/// par `companies::clear_stub_in_tx`, avec un appelant unique (Story 15-7a2) —
/// deux invariants documentés qu'on ne touche pas.
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

    // Story 25-1b (AC 4, 8, 9) — `details_json` ne porte QUE les champs que cette
    // route touche : `companies::update` est un full-replace et les handlers
    // reconstruisent tous les champs depuis l'état courant, si bien qu'un
    // `{before, after}` complet dirait que quinze champs ont changé quand un seul
    // a bougé.
    //
    // ⚠️ C'est ici que la trace vaut le plus : une clé absente vaut `null` et
    // EFFACE. Un effacement par omission ne laissait aucune trace.
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(map_db_error(e)))?;
    let updated = companies::update_in_tx(&mut tx, company.id, req.version, changes).await?;
    // AC 8 — le repository court-circuite le no-op sans incrémenter `version`.
    if updated.version != company.version {
        audit_log::insert_in_tx(
            &mut tx,
            // `::user` est sûr : la route est dans `admin_routes`, seul bloc
            // porteur de `require_not_pat`.
            NewAuditLogEntry::user(
                current_user.user_id,
                "company.updated",
                "company",
                company.id,
                Some(serde_json::json!({
                    "before": { "phone": company.phone, "website": company.website },
                    "after": { "phone": updated.phone, "website": updated.website },
                })),
            ),
        )
        .await?;
    }
    tx.commit()
        .await
        .map_err(|e| AppError::Database(map_db_error(e)))?;
    Ok(Json(CompanyJson::from(updated)))
}

/// Payload de `PUT /api/v1/companies/current/address` (Story 33-5, #909).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCompanyAddressRequest {
    pub address: crate::address_input::StructuredAddressInput,
    pub version: i32,
}

/// PUT /api/v1/companies/current/address — corrige l'adresse de la société après
/// l'assistant (Story 33-5, #909), sur le patron d'[`update_company_contact_details`].
///
/// ⛔ **`validate_required_for_qr`, et non `validate_optional_keeping`** (C-e334-5,
/// C-e334-41) : il passe par `validate_required`, puis juge les composants rognés
/// au jeu de caractères de la QR-facture. Un
/// pays inconnu est refusé **même inchangé**. Le contact peut garder le sien parce
/// que sa QR est facultative ; la société est créancière de toute QR-facture, et
/// accepter son pays annoncerait un succès en laissant le PDF en panne.
///
/// `is_stub` ne bouge pas : le lever reste l'affaire de l'assistant. Une facture au
/// PDF déjà figé ne change pas.
pub async fn update_company_address(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<UpdateCompanyAddressRequest>,
) -> Result<Json<CompanyJson>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let address = req.address.validate_required_for_qr()?;

    let changes = CompanyUpdate {
        name: company.name.clone(),
        first_name: company.first_name.clone(),
        last_name: company.last_name.clone(),
        address_structured: address,
        ide_number: company.ide_number.clone(),
        org_type: company.org_type,
        accounting_language: company.accounting_language,
        instance_language: company.instance_language,
        // Reportés à l'identique : `update_in_tx` est un full-replace.
        email: company.email.clone(),
        phone: company.phone.clone(),
        website: company.website.clone(),
    };

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(map_db_error(e)))?;
    let updated = companies::update_in_tx(&mut tx, company.id, req.version, changes).await?;
    // Le repository court-circuite le no-op sans incrémenter `version` (KF-004).
    if updated.version != company.version {
        // Les cinq composants (25-1b, AC 4, 8, 9) ; plus l'ancien texte quand ils ne
        // le redonnent pas : c'est le cas d'une société d'avant les composants, dont
        // la route écrase la seule source de l'adresse.
        let mut before = serde_json::json!(crate::address_input::StructuredAddressInput::from(
            &company.structured_address()
        ));
        if company.address != company.structured_address().combined() {
            before["address"] = serde_json::json!(company.address);
        }
        audit_log::insert_in_tx(
            &mut tx,
            // `::user` est sûr : la route est dans `admin_routes`, seul bloc
            // porteur de `require_not_pat`.
            NewAuditLogEntry::user(
                current_user.user_id,
                "company.updated",
                "company",
                company.id,
                Some(serde_json::json!({
                    "before": before,
                    "after": crate::address_input::StructuredAddressInput::from(
                        &updated.structured_address()
                    ),
                })),
            ),
        )
        .await?;
    }
    tx.commit()
        .await
        .map_err(|e| AppError::Database(map_db_error(e)))?;
    Ok(Json(CompanyJson::from(updated)))
}

/// Payload de `PUT /api/v1/companies/current/currency-settings` (Story 37-1,
/// AC-3). Les trois identifiants sont présents ou `null` (absents = `null`).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCurrencySettingsRequest {
    pub multi_currency_enabled: bool,
    #[serde(default)]
    pub fx_gain_account_id: Option<i64>,
    #[serde(default)]
    pub fx_loss_account_id: Option<i64>,
    #[serde(default)]
    pub fx_transit_account_id: Option<i64>,
    pub version: i32,
}

/// PUT /api/v1/companies/current/currency-settings — écrit le réglage
/// multi-devises de la société (Story 37-1, AC-3) : mode **irréversible** une
/// fois activé, comptes des gains et des pertes de change, compte de
/// conversion. Administrateur seul, par une session (cookie de l'écran ou
/// `Authorization: Bearer` d'un `POST /auth/login`), jamais par une clé d'API
/// (`require_not_pat`).
///
/// Les gardes, l'ordre des refus, les verrous et l'audit vivent dans
/// `companies::update_currency_settings`. **Rejouée sur interblocage** : la
/// transaction tient `companies` `FOR UPDATE` puis, à l'activation, les
/// exercices ; une saisie d'écriture prend l'exercice puis `companies` en
/// partage (clé étrangère), et la validation d'une facture les réglages de
/// facturation avant l'exercice — des cycles que le rejeu résout,
/// transaction unique sûre à relancer.
pub async fn update_currency_settings(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<UpdateCurrencySettingsRequest>,
) -> Result<Json<CompanyJson>, AppError> {
    let changes = CurrencySettingsUpdate {
        multi_currency_enabled: req.multi_currency_enabled,
        fx_gain_account_id: req.fx_gain_account_id,
        fx_loss_account_id: req.fx_loss_account_id,
        fx_transit_account_id: req.fx_transit_account_id,
    };
    let updated = kesh_db::retry::retry_on_deadlock("companies::update_currency_settings", || {
        let pool = state.pool.clone();
        async move {
            companies::update_currency_settings(
                &pool,
                current_user.user_id,
                current_user.company_id,
                req.version,
                changes,
            )
            .await
        }
    })
    .await?;
    Ok(Json(CompanyJson::from(updated)))
}

/// Payload de `PUT /api/v1/companies/current/name` (Story v0190-nom-societe, #1097).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCompanyNameRequest {
    pub name: String,
    pub version: i32,
}

/// Le nom de la société, rogné, s'il passe les contrôles que la QR-facture lui
/// applique (`kesh_qrbill::validation`, `validate_address`) : rien de visible
/// (`champ_vide_pour_qr`, qui voit U+00AD quand `trim` ne le voit pas), plus de
/// [`kesh_qrbill::validation::NAME_MAX`] caractères, ou un caractère hors du jeu.
///
/// Même ordre que le PDF (vide, longueur, jeu), pour que le refus de la saisie
/// soit celui que le PDF aurait opposé. Messages en français littéral, comme la
/// route d'adresse (`address_input::validate_required_for_qr`) : limite tracée
/// par #710.
pub(crate) fn validate_company_name_for_qr(raw: &str) -> Result<String, AppError> {
    let name = raw.trim();
    if kesh_qrbill::champ_vide_pour_qr(name) {
        return Err(AppError::Validation(
            "Le nom de la société est obligatoire.".to_string(),
        ));
    }
    let len = name.chars().count();
    let max = kesh_qrbill::validation::NAME_MAX;
    if len > max {
        return Err(AppError::Validation(format!(
            "Le nom de la société dépasse {max} caractères ({len}) : la QR-facture n'en admet pas davantage."
        )));
    }
    if let Some(c) = kesh_qrbill::first_qr_refused_char(name) {
        return Err(AppError::Validation(format!(
            "Le nom de la société contient un caractère que la QR-facture n'admet pas : « {} » (U+{:04X})",
            c.escape_debug(),
            c as u32
        )));
    }
    Ok(name.to_string())
}

/// PUT /api/v1/companies/current/name — corrige le nom de la société après
/// l'assistant (Story v0190-nom-societe, #1097), sur le patron
/// d'[`update_company_email`].
///
/// Le nom se juge **avant** la lecture de la société : un nom refusé à version
/// périmée est un `400`, non un `409`. `first_name` / `last_name` sont reportés
/// à l'identique (limite D3, #1227) : pour une personne physique, le nom corrigé
/// peut ne plus être « prénom nom ».
///
/// ⚠️ Comme ses jumelles, cette route **reconstruit un `CompanyUpdate` complet**
/// depuis l'état courant : `companies::update_in_tx` est un full-replace. Une
/// facture au PDF déjà figé ne change pas.
pub async fn update_company_name(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<UpdateCompanyNameRequest>,
) -> Result<Json<CompanyJson>, AppError> {
    let name = validate_company_name_for_qr(&req.name)?;
    let company = get_company_for(&current_user, &state.pool).await?;

    let changes = CompanyUpdate {
        name,
        first_name: company.first_name.clone(),
        last_name: company.last_name.clone(),
        address_structured: company.structured_address(),
        ide_number: company.ide_number.clone(),
        org_type: company.org_type,
        accounting_language: company.accounting_language,
        instance_language: company.instance_language,
        // Reportés à l'identique : `update_in_tx` est un full-replace.
        email: company.email.clone(),
        phone: company.phone.clone(),
        website: company.website.clone(),
    };

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(map_db_error(e)))?;
    let updated = companies::update_in_tx(&mut tx, company.id, req.version, changes).await?;
    // Le repository court-circuite le no-op sans incrémenter `version` (KF-004).
    if updated.version != company.version {
        audit_log::insert_in_tx(
            &mut tx,
            // `::user` est sûr : la route est dans `admin_routes`, seul bloc
            // porteur de `require_not_pat`.
            NewAuditLogEntry::user(
                current_user.user_id,
                "company.updated",
                "company",
                company.id,
                Some(serde_json::json!({
                    "before": { "name": company.name },
                    "after": { "name": updated.name },
                })),
            ),
        )
        .await?;
    }
    tx.commit()
        .await
        .map_err(|e| AppError::Database(map_db_error(e)))?;
    Ok(Json(CompanyJson::from(updated)))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BankAccountJson {
    pub id: i64,
    pub bank_name: String,
    /// Story 26-2b (#622) — description optionnelle (`null` si absente) : source
    /// du nom des comptes de *Paramètres*, de l'import bancaire et de la
    /// réconciliation.
    pub description: Option<String>,
    /// Story 34-11a1 (#972) — `null` pour un compte sans IBAN (carte de crédit).
    pub iban: Option<String>,
    pub qr_iban: Option<String>,
    /// Story 34-11a1 (#972) — `"bank"` ou `"credit_card"`.
    pub account_kind: BankAccountKind,
    /// Story 34-11a1 (#972) — quatre derniers chiffres d'une carte, `null` sinon.
    pub card_last4: Option<String>,
    /// Story 38-11d (AC-1) — prestataire dont le compte est le solde
    /// (`"paypal"`), `null` pour un compte ordinaire : les écrans d'import
    /// bancaire et de réconciliation qui lisent cette liste n'offrent pas un
    /// compte de prestataire (`withoutProviders`).
    pub provider: Option<String>,
    pub is_primary: bool,
    /// Story 8-5a-zero — compte du plan comptable lié (None = non configuré).
    pub journal_account_id: Option<i64>,
}

impl From<BankAccount> for BankAccountJson {
    fn from(b: BankAccount) -> Self {
        Self {
            id: b.id,
            bank_name: b.bank_name,
            description: b.description,
            iban: b.iban,
            qr_iban: b.qr_iban,
            account_kind: b.account_kind,
            card_last4: b.card_last4,
            provider: b.provider_settings.provider,
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
    // Story 28-4a (#642) — ni archivés ni clos : `/companies/current` nourrit
    // les sélecteurs d'import et de réconciliation, où un compte clos ne doit
    // plus être proposé.
    let accounts =
        bank_accounts::list_by_company(&state.pool, company.id, Default::default()).await?;

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
    borner(req.through, "through")?;
    let company = get_company_for(&current_user, &state.pool).await?;
    let updated = companies::lock_books(
        &state.pool,
        current_user.user_id,
        company.id,
        req.through,
        state.horloge.aujourd_hui(),
    )
    .await?;
    Ok(Json(updated.into()))
}

/// POST /api/v1/companies/current/books-lock/release — **recule ou retire** le
/// verrou de période (Admin uniquement, motif obligatoire, Story 24-4c).
pub async fn unlock_company_books(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<UnlockBooksRequest>,
) -> Result<Json<CompanyJson>, AppError> {
    borner_opt(req.through, "through")?;
    let company = get_company_for(&current_user, &state.pool).await?;
    let updated = companies::unlock_books(
        &state.pool,
        current_user.user_id,
        company.id,
        req.through,
        req.motif,
        state.horloge.aujourd_hui(),
    )
    .await?;
    Ok(Json(updated.into()))
}

#[cfg(test)]
mod tests {
    use super::validate_company_name_for_qr;
    use crate::errors::AppError;

    /// Le message du refus, ou la panique si le nom passe.
    fn refus(raw: &str) -> String {
        match validate_company_name_for_qr(raw) {
            Err(AppError::Validation(m)) => m,
            other => panic!("refus attendu pour {raw:?} : {other:?}"),
        }
    }

    /// AC2 — 0 caractère, et U+00AD seul, que `trim` ne retire pas : vides au sens
    /// de la QR-facture.
    #[test]
    fn un_nom_sans_rien_de_visible_est_refuse() {
        for raw in ["", "   ", "\u{AD}", " \u{AD} "] {
            assert!(refus(raw).contains("obligatoire"), "{raw:?}");
        }
    }

    /// AC2 — 1 et 70 caractères passent, multioctets compris ; 71 est refusé, et
    /// le message dit le maximum et le nombre saisi.
    #[test]
    fn la_borne_de_70_caracteres() {
        assert_eq!(validate_company_name_for_qr("A").unwrap(), "A");
        let soixante_dix = "é".repeat(70);
        assert_eq!(
            validate_company_name_for_qr(&soixante_dix).unwrap(),
            soixante_dix
        );
        let m = refus(&"é".repeat(71));
        assert!(m.contains("70") && m.contains("(71)"), "{m}");
    }

    /// Le nom est rogné avant d'être compté et jugé : 70 caractères entourés de
    /// blancs passent.
    #[test]
    fn le_nom_est_rogne_avant_les_controles() {
        let nom = format!("  {}\n", "a".repeat(70));
        assert_eq!(validate_company_name_for_qr(&nom).unwrap(), "a".repeat(70));
    }

    /// AC2 — le refus nomme le PREMIER caractère refusé, avec son point de code.
    #[test]
    fn le_premier_caractere_refuse_est_nomme() {
        let m = refus("Démo 张 SA \u{2603}");
        assert!(m.contains("« 张 »") && m.contains("U+5F20"), "{m}");
        assert!(!m.contains("U+2603"), "{m}");
        let m = refus("Démo\nSA");
        assert!(m.contains("\\n") && m.contains("U+000A"), "{m}");
    }

    /// Ce que le jeu de la QR-facture admet passe : `&`, `é`, U+2019.
    #[test]
    fn les_caracteres_admis_passent() {
        let nom = "Müller & Fils’ Sàrl";
        assert_eq!(validate_company_name_for_qr(nom).unwrap(), nom);
    }
}
