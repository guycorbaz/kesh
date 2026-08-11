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
use kesh_db::entities::Language;
use kesh_db::entities::contact::{Contact, ContactType, ContactUpdate, NewContact, Salutation};
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
const MAX_PAYMENT_TERMS_LEN: usize = 100;
/// Story 16-3b (#151) — aligné sur la colonne `contacts.client_number`
/// VARCHAR(50). Longueur de STOCKAGE : le PDF, lui, tronque à l'affichage.
const MAX_CLIENT_NUMBER_LEN: usize = 50;
/// Borne haute du délai de paiement en jours (#245) — miroir du CHECK SQL
/// `chk_contacts_payment_terms_days` (le CHECK n'est que le filet).
const MAX_PAYMENT_TERMS_DAYS: i32 = 365;
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
    /// Prénom / nom (#213) — requis si `contact_type == Personne`, sinon ignorés.
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub is_client: bool,
    #[serde(default)]
    pub is_supplier: bool,
    /// Adresse structurée (#213). Bloc vide = pas d'adresse.
    #[serde(default)]
    pub address_structured: crate::address_input::StructuredAddressInput,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub ide_number: Option<String>,
    /// Numéro de client (Story 16-3b, #151). Absent/null/`""` = non renseigné
    /// — `normalize_optional` effondre le vide en `None`, sans quoi deux
    /// contacts soumis avec `""` percuteraient l'unicité.
    #[serde(default)]
    pub client_number: Option<String>,
    #[serde(default)]
    pub default_payment_terms: Option<String>,
    /// Délai de paiement en jours (#245). Absent/null = non renseigné.
    /// Renseigné → prime sur le texte libre (libellé auto-généré).
    #[serde(default)]
    pub default_payment_terms_days: Option<i32>,
    /// Langue de correspondance (Story 20-3b1). Absent/null = hérite de la
    /// langue d'instance de la société.
    #[serde(default)]
    pub language: Option<Language>,
    /// Civilité (Story 20-3b1). Absent = `Neutre`.
    #[serde(default)]
    pub salutation: Option<Salutation>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateContactRequest {
    pub contact_type: ContactType,
    pub name: String,
    /// Prénom / nom (#213) — requis si `contact_type == Personne`, sinon ignorés.
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub is_client: bool,
    #[serde(default)]
    pub is_supplier: bool,
    /// Adresse structurée (#213). Bloc vide = pas d'adresse.
    #[serde(default)]
    pub address_structured: crate::address_input::StructuredAddressInput,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub ide_number: Option<String>,
    /// Numéro de client (Story 16-3b, #151). Absent/null/`""` = non renseigné
    /// — `normalize_optional` effondre le vide en `None`, sans quoi deux
    /// contacts soumis avec `""` percuteraient l'unicité.
    #[serde(default)]
    pub client_number: Option<String>,
    #[serde(default)]
    pub default_payment_terms: Option<String>,
    /// Délai de paiement en jours (#245). Absent/null = effacé (PUT full-payload).
    #[serde(default)]
    pub default_payment_terms_days: Option<i32>,
    /// Langue de correspondance (Story 20-3b1). Absent/null = héritage instance.
    #[serde(default)]
    pub language: Option<Language>,
    /// Civilité (Story 20-3b1). Absent = `Neutre`.
    #[serde(default)]
    pub salutation: Option<Salutation>,
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
    /// Prénom / nom (#213) — renseignés pour les Personne.
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_client: bool,
    pub is_supplier: bool,
    /// Chaîne d'affichage dérivée (#213).
    pub address: Option<String>,
    /// Adresse structurée (source de vérité éditable côté frontend, #213).
    pub address_structured: crate::address_input::StructuredAddressInput,
    pub email: Option<String>,
    pub phone: Option<String>,
    /// Forme normalisée `"CHE109322551"`. Le frontend la formate pour l'affichage.
    pub ide_number: Option<String>,
    /// Numéro de client attribué par l'émetteur (Story 16-3b, #151).
    pub client_number: Option<String>,
    pub default_payment_terms: Option<String>,
    /// Délai de paiement en jours (#245). `null` = non renseigné.
    pub default_payment_terms_days: Option<i32>,
    /// Libellé localisé des conditions de paiement (#245), généré côté
    /// serveur dans la **langue du contact** (fallback langue d'instance) —
    /// le i18n frontend ne connaît que la locale UI. `null` si `days` absent.
    pub default_payment_terms_label: Option<String>,
    /// Langue de correspondance (Story 20-3b1). `null` = héritage instance.
    pub language: Option<Language>,
    /// Civilité (Story 20-3b1).
    pub salutation: Salutation,
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
            first_name: c.first_name,
            last_name: c.last_name,
            is_client: c.is_client,
            is_supplier: c.is_supplier,
            address_structured: crate::address_input::StructuredAddressInput {
                street: c.address_street.unwrap_or_default(),
                building: c.address_building.unwrap_or_default(),
                postal_code: c.address_postal_code.unwrap_or_default(),
                city: c.address_city.unwrap_or_default(),
                country: c.address_country,
            },
            address: c.address,
            email: c.email,
            phone: c.phone,
            // Copie directe — déjà normalisée en base via CheNumber::new().as_str()
            // au moment de l'INSERT. Pas de re-parse CheNumber ici.
            ide_number: c.ide_number,
            client_number: c.client_number,
            default_payment_terms: c.default_payment_terms,
            default_payment_terms_days: c.default_payment_terms_days,
            // Le libellé exige la Company (langue d'instance) + I18nBundle —
            // posé par `contact_response_with_label`, jamais par ce From.
            default_payment_terms_label: None,
            language: c.language,
            salutation: c.salutation,
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
///
/// `pub(crate)` (Story 17-4a) : réutilisé par `routes/setup` et `routes/users`
/// pour valider l'email optionnel du compte (recovery #122).
pub(crate) fn is_valid_email_simple(s: &str) -> bool {
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
///
/// `pub(crate)` (Story 17-4a) : réutilisé pour normaliser l'email optionnel
/// du compte (vide → `None` = effaçage).
/// ⚠️ **« Vide » se juge sur ce qui MARQUE, pas sur `trim().is_empty()`.**
/// `str::trim` suit la propriété Unicode `White_Space`, qui **n'inclut pas** les
/// caractères de largeur nulle : `U+200B` (ZWSP), `U+FEFF` (BOM) et `U+2060`
/// (word joiner) traversent `trim()` intacts. Une valeur qui n'en contient que
/// de ceux-là est vide **pour l'utilisateur** — le champ paraît vide à l'écran —
/// mais serait stockée comme une valeur ordinaire.
///
/// Le coût est concret sur le numéro de client (16-3b), dont l'unicité est
/// portée par un index : deux fiches où l'utilisateur croit avoir laissé le
/// champ vide se percutent en **409 `CLIENT_NUMBER_ALREADY_EXISTS`**, sur une
/// valeur qu'il ne peut ni voir ni effacer. C'est exactement ce que
/// `empty_client_number_is_stored_as_null_and_never_collides` promet
/// d'empêcher, et que le seul `trim()` ne tenait pas.
///
/// Une valeur qui **mélange** invisible et visible passe inchangée : seule la
/// valeur intégralement invisible est ramenée à `None`.
///
/// *(Jumeau côté rendu : `is_invisible` dans `kesh-qrbill/src/pdf.rs`, qui garde
/// le PDF contre les valeurs écrites hors API — restauration d'une sauvegarde
/// produite ailleurs, correction SQL directe. Les deux vivent dans des crates
/// sans dépendance commune ; ce sont deux couches, pas une duplication de
/// commodité. Passe 2 de `bmad-code-review`.)*
pub(crate) fn normalize_optional(s: Option<String>) -> Option<String> {
    s.and_then(|v| {
        let t = v.trim();
        if t.is_empty() || t.chars().all(is_invisible) {
            None
        } else {
            Some(t.to_string())
        }
    })
}

/// Vrai si le caractère ne **marque** rien à l'écran ni à l'impression.
///
/// `U+00AD` (trait d'union conditionnel) est inclus : il ne se rend pas hors
/// point de césure.
fn is_invisible(c: char) -> bool {
    c.is_whitespace()
        || c.is_control()
        || matches!(c,
            '\u{00AD}' | '\u{200B}'..='\u{200F}' | '\u{2060}'..='\u{2064}' | '\u{FEFF}')
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
    /// Nom canonique d'affichage/QR (#213) : raison sociale (Entreprise) ou
    /// « Prénom Nom » recomposé (Personne).
    name: String,
    first_name: Option<String>,
    last_name: Option<String>,
    is_client: bool,
    is_supplier: bool,
    address_structured: Option<kesh_db::entities::address::StructuredAddress>,
    email: Option<String>,
    phone: Option<String>,
    ide_number: Option<String>,
    client_number: Option<String>,
    default_payment_terms: Option<String>,
    default_payment_terms_days: Option<i32>,
}

#[allow(clippy::too_many_arguments)]
fn validate_common(
    contact_type: ContactType,
    name: String,
    first_name: Option<String>,
    last_name: Option<String>,
    is_client: bool,
    is_supplier: bool,
    address_structured: crate::address_input::StructuredAddressInput,
    email: Option<String>,
    phone: Option<String>,
    ide_number: Option<String>,
    client_number: Option<String>,
    default_payment_terms: Option<String>,
    default_payment_terms_days: Option<i32>,
) -> Result<ValidatedFields, AppError> {
    // #213 — Personne : prénom + nom séparés, `name` recomposé « Prénom Nom ».
    // Entreprise : raison sociale unique (prénom/nom laissés vides).
    let clean = |o: Option<String>| o.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let (trimmed_name, first_name, last_name) = match contact_type {
        ContactType::Personne => match (clean(first_name), clean(last_name)) {
            (Some(f), Some(l)) => (format!("{f} {l}"), Some(f), Some(l)),
            _ => {
                return Err(AppError::Validation(
                    "Le prénom et le nom sont obligatoires pour une personne".into(),
                ));
            }
        },
        ContactType::Entreprise => {
            let n = name.trim().to_string();
            if n.is_empty() {
                return Err(AppError::Validation(
                    "La raison sociale est obligatoire pour une entreprise".into(),
                ));
            }
            (n, None, None)
        }
    };
    if trimmed_name.chars().count() > MAX_NAME_LEN {
        return Err(AppError::Validation(format!(
            "Le nom doit faire au plus {MAX_NAME_LEN} caractères"
        )));
    }

    // Adresse structurée (#213) : longueurs SIX type S, bloc vide → None.
    let address_structured = address_structured.validate_optional()?;

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
    if let Some(ref p) = phone
        && p.chars().count() > MAX_PHONE_LEN
    {
        return Err(AppError::Validation(format!(
            "Le téléphone doit faire au plus {MAX_PHONE_LEN} caractères"
        )));
    }

    let default_payment_terms = normalize_optional(default_payment_terms);
    if let Some(ref t) = default_payment_terms
        && t.chars().count() > MAX_PAYMENT_TERMS_LEN
    {
        return Err(AppError::Validation(format!(
            "Les conditions de paiement doivent faire au plus {MAX_PAYMENT_TERMS_LEN} caractères"
        )));
    }

    // #245 : borne 0..=MAX_PAYMENT_TERMS_DAYS — chaîne FR en dur, cohérent
    // avec les autres messages de validate_common (aucun i18n dans ce fichier).
    if let Some(d) = default_payment_terms_days
        && !(0..=MAX_PAYMENT_TERMS_DAYS).contains(&d)
    {
        return Err(AppError::Validation(format!(
            "Le délai de paiement doit être compris entre 0 et {MAX_PAYMENT_TERMS_DAYS} jours"
        )));
    }

    // Story 16-3b : `normalize_optional` AVANT toute chose — `""` n'est pas
    // `NULL` pour un index UNIQUE, et le cas majoritaire (aucun numéro) se
    // percuterait dès le deuxième contact.
    let client_number = normalize_optional(client_number);
    if let Some(ref cn) = client_number
        && cn.chars().count() > MAX_CLIENT_NUMBER_LEN
    {
        return Err(AppError::Validation(format!(
            "Le numéro de client doit faire au plus {MAX_CLIENT_NUMBER_LEN} caractères"
        )));
    }

    let ide_number = validate_optional_ide(ide_number)?;

    Ok(ValidatedFields {
        contact_type,
        name: trimmed_name,
        first_name,
        last_name,
        is_client,
        is_supplier,
        address_structured,
        email,
        phone,
        ide_number,
        client_number,
        default_payment_terms,
        default_payment_terms_days,
    })
}

/// Libellé localisé des conditions de paiement depuis le délai en jours (#245).
///
/// Appelle `bundle.format(&locale, …)` **directement** (PAS `t`/`t_args` de
/// `errors.rs`, qui lisent la locale globale de la *requête*) : le libellé
/// doit être dans la langue du **contact**, indépendante de la locale UI.
///
/// `pub(crate)` : réutilisé par `routes/invoices` (libellé auto copié dans
/// `invoices.payment_terms` à la création).
pub(crate) fn payment_terms_label(
    days: i32,
    locale: kesh_i18n::Locale,
    i18n: &kesh_i18n::I18nBundle,
) -> String {
    let label = if days == 0 {
        // « Payable au comptant » / « Zahlbar sofort » / « Pagabile a vista »
        // / « Due upon receipt »
        i18n.format(&locale, "contact-payment-terms-immediate-label", None)
    } else {
        let mut args = kesh_i18n::FluentArgs::new();
        args.set("days", i64::from(days));
        i18n.format(&locale, "contact-payment-terms-days-label", Some(&args))
    };
    // Fluent entoure les variables interpolées de marques d'isolation BiDi
    // (U+2068 FSI / U+2069 PDI). Ce libellé est copié dans
    // `invoices.payment_terms` et imprimé sur le PDF (Helvetica WinAnsi,
    // aucun glyphe pour ces codepoints) → on les retire, texte 100 % LTR.
    label.replace(['\u{2068}', '\u{2069}'], "")
}

/// Construit la `ContactResponse` en posant le libellé localisé (#245).
///
/// **Tous les handlers qui renvoient un contact passent par ici** (list, get,
/// create, update, archive) — `ContactPicker` du formulaire facture consomme
/// l'endpoint list, un label absent y casserait le pré-remplissage.
fn contact_response_with_label(
    contact: Contact,
    company: &kesh_db::entities::company::Company,
    i18n: &kesh_i18n::I18nBundle,
) -> ContactResponse {
    let language = crate::routes::invoice_email::resolve_language(&contact, company);
    let label = contact
        .default_payment_terms_days
        .map(|d| payment_terms_label(d, kesh_i18n::Locale::from(language.as_str()), i18n));
    let mut resp = ContactResponse::from(contact);
    resp.default_payment_terms_label = label;
    resp
}

/// Remappe les `UniqueConstraintViolation` de la table `contacts` vers leur code
/// client dédié, selon la contrainte violée :
///
/// - `uq_contacts_company_ide` → `IDE_ALREADY_EXISTS`
/// - `uq_contacts_company_client_number` → `CLIENT_NUMBER_ALREADY_EXISTS` (16-3b)
///
/// Toute autre violation est propagée telle quelle — c'est ce que vérifie
/// `map_contact_error_other_unique_maps_to_generic_conflict`.
///
/// **Note** : on ne matche que le **nom de contrainte** (`uq_contacts_company_ide`),
/// pas le nom de colonne (`ide_number`) — le format du message d'erreur
/// MariaDB peut varier entre versions (10.x vs 11.x, schéma préfixé ou non).
fn map_contact_error(err: DbError) -> AppError {
    if let DbError::UniqueConstraintViolation(ref m) = err
        && m.contains("uq_contacts_company_ide")
    {
        return AppError::IdeAlreadyExists("Un contact avec ce numéro IDE existe déjà".into());
    }
    // Story 16-3b : même nature, même table, donc même code HTTP (409). La
    // contrainte porte sur la colonne GÉNÉRÉE `client_number_uniq`, mais c'est
    // bien son NOM (`uq_contacts_company_client_number`) qui apparaît dans le
    // message MariaDB — et c'est le nom de contrainte qu'on matche, jamais
    // celui de la colonne, pour la raison déjà documentée ci-dessus.
    if let DbError::UniqueConstraintViolation(ref m) = err
        && m.contains("uq_contacts_company_client_number")
    {
        return AppError::ClientNumberAlreadyExists(
            "Un contact avec ce numéro de client existe déjà".into(),
        );
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
    // Company : contrôle défensif (staleness window) ET langue d'instance
    // pour le libellé des conditions de paiement (#245).
    let company = get_company_for(&current_user, &state.pool).await?;

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
            .map(|c| contact_response_with_label(c, &company, &state.i18n))
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

    // #245 : Company requise pour la langue d'instance (fallback du libellé).
    let company = get_company_for(&current_user, &state.pool).await?;
    Ok(Json(contact_response_with_label(
        contact,
        &company,
        &state.i18n,
    )))
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
        req.first_name,
        req.last_name,
        req.is_client,
        req.is_supplier,
        req.address_structured,
        req.email,
        req.phone,
        req.ide_number,
        req.client_number,
        req.default_payment_terms,
        req.default_payment_terms_days,
    )?;

    let a = v.address_structured.as_ref();
    let new = NewContact {
        company_id: company.id,
        contact_type: v.contact_type,
        name: v.name,
        first_name: v.first_name,
        last_name: v.last_name,
        is_client: v.is_client,
        is_supplier: v.is_supplier,
        // `address` (chaîne dérivée) recomposée par le repo depuis les champs structurés.
        address: None,
        address_street: a.map(|s| s.street.clone()),
        address_building: a.map(|s| s.building.clone()),
        address_postal_code: a.map(|s| s.postal_code.clone()),
        address_city: a.map(|s| s.city.clone()),
        address_country: a.map(|s| s.country.clone()),
        email: v.email,
        phone: v.phone,
        ide_number: v.ide_number,
        client_number: v.client_number,
        default_payment_terms: v.default_payment_terms,
        default_payment_terms_days: v.default_payment_terms_days,
        // Story 20-3b1 : enums typés — serde a déjà rejeté toute valeur
        // invalide (400 body parse). Civilité absente = Neutre.
        language: req.language,
        salutation: req.salutation.unwrap_or_default(),
    };

    let contact = contacts::create(&state.pool, current_user.user_id, new)
        .await
        .map_err(map_contact_error)?;

    Ok((
        StatusCode::CREATED,
        Json(contact_response_with_label(contact, &company, &state.i18n)),
    ))
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
        req.first_name,
        req.last_name,
        req.is_client,
        req.is_supplier,
        req.address_structured,
        req.email,
        req.phone,
        req.ide_number,
        req.client_number,
        req.default_payment_terms,
        req.default_payment_terms_days,
    )?;

    let a = v.address_structured.as_ref();
    let changes = ContactUpdate {
        contact_type: v.contact_type,
        name: v.name,
        first_name: v.first_name,
        last_name: v.last_name,
        is_client: v.is_client,
        is_supplier: v.is_supplier,
        address: None,
        address_street: a.map(|s| s.street.clone()),
        address_building: a.map(|s| s.building.clone()),
        address_postal_code: a.map(|s| s.postal_code.clone()),
        address_city: a.map(|s| s.city.clone()),
        address_country: a.map(|s| s.country.clone()),
        email: v.email,
        phone: v.phone,
        ide_number: v.ide_number,
        client_number: v.client_number,
        default_payment_terms: v.default_payment_terms,
        default_payment_terms_days: v.default_payment_terms_days,
        // Story 20-3b1 : cf. create_contact — enums typés validés par serde.
        language: req.language,
        salutation: req.salutation.unwrap_or_default(),
    };

    let contact = contacts::update(&state.pool, id, req.version, current_user.user_id, changes)
        .await
        .map_err(map_contact_error)?;

    // #245 : Company requise pour la langue d'instance (fallback du libellé).
    let company = get_company_for(&current_user, &state.pool).await?;
    Ok(Json(contact_response_with_label(
        contact,
        &company,
        &state.i18n,
    )))
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

    // #245 : uniformité API — le libellé est posé sur les 5 endpoints contacts.
    let company = get_company_for(&current_user, &state.pool).await?;
    Ok(Json(contact_response_with_label(
        contact,
        &company,
        &state.i18n,
    )))
}

// ---------------------------------------------------------------------------
// Tests unitaires
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

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

    /// Story 16-3b : la contrainte jumelle rend la variante dédiée.
    #[test]
    fn map_contact_error_client_number_unique_maps_to_dedicated_variant() {
        let err = DbError::UniqueConstraintViolation("uq_contacts_company_client_number".into());
        match map_contact_error(err) {
            AppError::ClientNumberAlreadyExists(_) => {}
            other => panic!("expected ClientNumberAlreadyExists, got {other:?}"),
        }
    }

    /// Story 16-3b — **non-sur-capture**, dans les DEUX sens.
    ///
    /// `map_contact_error` matche par `contains` : avec deux contraintes sur la
    /// même table, rien ne garantit a priori qu'aucune ne capture l'autre. Une
    /// sur-capture rendrait le mauvais code d'erreur au frontend, qui branche
    /// les codes un par un.
    #[test]
    fn map_contact_error_does_not_confuse_the_two_contact_constraints() {
        match map_contact_error(DbError::UniqueConstraintViolation(
            "Duplicate entry 'CHE109322551' for key 'uq_contacts_company_ide'".into(),
        )) {
            AppError::IdeAlreadyExists(_) => {}
            other => panic!("l'IDE ne doit PAS être capturé par le numéro de client : {other:?}"),
        }
        match map_contact_error(DbError::UniqueConstraintViolation(
            "Duplicate entry '1-CLI-1' for key 'uq_contacts_company_client_number'".into(),
        )) {
            AppError::ClientNumberAlreadyExists(_) => {}
            other => panic!("le numéro de client ne doit PAS être capturé par l'IDE : {other:?}"),
        }
    }

    /// Story 16-3b : la variante rend bien un **409**, et non un 500.
    ///
    /// Le contrôle de la CHAÎNE `CLIENT_NUMBER_ALREADY_EXISTS` — l'interface
    /// réelle avec le frontend — est fait par `client_number_conflict_has_its_own_code`
    /// dans `errors.rs`, seul endroit où le corps de la réponse est lisible
    /// (helper `response_body`). Ce test-ci ne couvre que le statut : son nom le
    /// disait autrefois davantage qu'il ne le vérifiait.
    #[test]
    fn client_number_conflict_renders_409() {
        let response = AppError::ClientNumberAlreadyExists("déjà pris".into()).into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
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

    /// Une valeur faite **uniquement** de caractères invisibles est vide pour
    /// l'utilisateur, et doit l'être pour la base.
    ///
    /// ⚠️ `trim()` ne suffit pas : la propriété Unicode `White_Space` n'inclut
    /// pas les caractères de largeur nulle. Sans ce traitement, deux fiches où
    /// l'utilisateur croit avoir laissé le numéro de client vide se percutent
    /// en 409 sur une valeur invisible qu'il ne peut ni voir ni effacer.
    #[test]
    fn normalize_optional_collapses_invisible_only_values_to_none() {
        for (label, value) in [
            ("ZWSP U+200B", "\u{200B}"),
            ("BOM U+FEFF", "\u{FEFF}"),
            ("word joiner U+2060", "\u{2060}"),
            ("soft hyphen U+00AD", "\u{00AD}"),
            ("ZWSP entouré d'espaces", "  \u{200B}  "),
            ("plusieurs invisibles", "\u{200B}\u{FEFF}\u{2060}"),
        ] {
            assert_eq!(
                normalize_optional(Some(value.into())),
                None,
                "« {label} » doit être ramené à None"
            );
        }
    }

    /// ⚠️ Le pendant du test précédent, et il est indispensable : une garde trop
    /// large mangerait des valeurs légitimes. Un invisible **entouré de
    /// visible** doit passer INCHANGÉ — c'est du contenu réel, mal collé.
    #[test]
    fn normalize_optional_keeps_values_mixing_visible_and_invisible() {
        assert_eq!(
            normalize_optional(Some("CLI\u{200B}-1".into())),
            Some("CLI\u{200B}-1".into())
        );
        assert_eq!(
            normalize_optional(Some("  CLI-1\u{FEFF}  ".into())),
            Some("CLI-1\u{FEFF}".into())
        );
    }
}
