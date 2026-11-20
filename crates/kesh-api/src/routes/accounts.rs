//! Routes CRUD pour les comptes du plan comptable.
//!
//! La lecture du plan (`list_accounts`) passe par la couche de services
//! (`crate::services::plan_comptable`, Story 32-3), qui porte son contrôle
//! d'accès ; les mutations restent ici.

use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use kesh_db::entities::account::{
    Account, AccountRole, AccountType, AccountUpdate, CurrencyChange, NewAccount, OwnFxAccounts,
    OwnFxAccountsChange, ParentChange,
};
use kesh_db::errors::{DbError, FxAccountField, FxAccountReason, map_db_error};
use kesh_db::repositories::{accounts, letterings};

use crate::AppState;
use crate::errors::AppError;
use crate::helpers::get_company_for;
use crate::middleware::auth::CurrentUser;
use crate::services::acces::ContexteAcces;
use crate::services::plan_comptable;

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
    /// Devise (Story 37-1, AC-4), code ISO 4217 ; absente ou `null` = CHF.
    /// `"CHF"` (toute casse) est normalisé en CHF avant tout contrôle.
    #[serde(default)]
    pub currency: Option<String>,
    /// Compte de gains de change propre (Story 37-1, AC-4 bis) ; absent = par
    /// défaut. Va avec [`Self::fx_loss_account_id`].
    #[serde(default)]
    pub fx_gain_account_id: Option<i64>,
    /// Compte de pertes de change propre (Story 37-1, AC-4 bis).
    #[serde(default)]
    pub fx_loss_account_id: Option<i64>,
}

fn default_true() -> bool {
    true
}

/// Le contrôle de forme d'un compte parent, **commun à la création et à la
/// modification** (correctif `fix-api-parent-compte`) : il existe, appartient
/// à la société courante et est actif. Sinon `400 VALIDATION_ERROR`.
///
/// Le scope société n'était pas vérifié à l'origine : un `parentId` d'une autre
/// société était accepté (code review 14-3a — d'autant plus sensible que
/// rattacher un enfant modifie la postabilité du parent). Lecture hors
/// transaction : le dépôt relit le parent sous verrou à la modification
/// (`accounts::update`), où il refuse en plus un cycle.
async fn check_parent_in_company(
    state: &AppState,
    parent_id: i64,
    company_id: i64,
) -> Result<(), AppError> {
    match accounts::find_by_id_in_company(&state.pool, parent_id, company_id).await? {
        None => Err(AppError::Validation("parent account not found".into())),
        Some(p) if !p.active => Err(AppError::Validation("parent account is archived".into())),
        Some(_) => Ok(()),
    }
}

/// Normalise une devise reçue (Story 37-1, AC-4) : `trim`, majuscules ASCII,
/// `CHF` → `None` (devise principale), code hors liste ISO 4217 →
/// `CURRENCY_UNKNOWN` (400). Avant tout autre contrôle de devise.
///
/// La valeur refusée est reprise dans le message et les `details` : elle est
/// **tronquée** à [`CURRENCY_ECHO_MAX`] caractères, une saisie illimitée ne
/// revenant pas deux fois dans la réponse (revue de code P1, 37-1-CR-P1-1).
fn normalize_currency(raw: &str) -> Result<Option<String>, AppError> {
    kesh_core::currency::normalize_foreign_currency(raw)
        .map(|code| code.map(String::from))
        .map_err(|_| {
            AppError::Database(DbError::CurrencyUnknown {
                value: raw.chars().take(CURRENCY_ECHO_MAX).collect(),
            })
        })
}

/// Nombre maximal de caractères d'une devise refusée repris dans la réponse
/// (`CURRENCY_UNKNOWN`) : un code ISO 4217 en compte trois.
const CURRENCY_ECHO_MAX: usize = 32;

/// Le refus de forme des comptes propres (Story 37-1, AC-4 bis) : l'un sans
/// l'autre → `FX_ACCOUNT_INVALID` `missing` sur le champ manquant, avant la
/// transaction.
fn own_fx_missing(missing_gain: bool, holder_account_id: Option<i64>) -> AppError {
    AppError::Database(DbError::FxAccountInvalid {
        field: if missing_gain {
            FxAccountField::Gain
        } else {
            FxAccountField::Loss
        },
        reason: FxAccountReason::Missing,
        holder_account_id,
    })
}

/// Comptes propres d'une création : les deux, ou aucun.
fn own_fx_for_create(
    gain: Option<i64>,
    loss: Option<i64>,
) -> Result<Option<OwnFxAccounts>, AppError> {
    match (gain, loss) {
        (None, None) => Ok(None),
        (Some(gain), Some(loss)) => Ok(Some(OwnFxAccounts { gain, loss })),
        (None, Some(_)) => Err(own_fx_missing(true, None)),
        (Some(_), None) => Err(own_fx_missing(false, None)),
    }
}

/// Comptes propres d'une modification (trois états, Story 37-1, AC-4 bis) :
/// les deux absents = inchangés ; les deux `null` = par défaut ; les deux
/// posés = comptes propres ; toute autre forme → `missing`.
fn own_fx_for_update(
    gain: Option<Option<i64>>,
    loss: Option<Option<i64>>,
    holder_account_id: i64,
) -> Result<OwnFxAccountsChange, AppError> {
    match (gain, loss) {
        (None, None) => Ok(OwnFxAccountsChange::Unchanged),
        (Some(None), Some(None)) => Ok(OwnFxAccountsChange::Company),
        (Some(Some(gain)), Some(Some(loss))) => {
            Ok(OwnFxAccountsChange::Own(OwnFxAccounts { gain, loss }))
        }
        // Le gain posé (perte absente ou `null`), ou le gain `null` et la perte
        // absente : c'est la perte qui manque.
        (Some(Some(_)), _) | (Some(None), None) => {
            Err(own_fx_missing(false, Some(holder_account_id)))
        }
        // Le gain absent, ou `null` face à une perte posée.
        _ => Err(own_fx_missing(true, Some(holder_account_id))),
    }
}

/// Désérialise un champ en distinguant **absent** de **`null`**.
///
/// `Option<Option<T>>` seul ne suffit pas : serde mappe `null` sur le `None`
/// **externe**, rendant les deux cas indiscernables. En déléguant à
/// `Option<T>` puis en enveloppant systématiquement dans `Some`, on obtient
/// `None` uniquement quand serde n'appelle pas ce désérialiseur — c'est-à-dire
/// quand la clé est absente (grâce à `#[serde(default)]`).
fn present_or_null<'de, T, D>(de: D) -> Result<Option<Option<T>>, D::Error>
where
    T: serde::Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    Option::<T>::deserialize(de).map(Some)
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
    /// `None` = champ **absent** du payload (rejeté en 400) ; `Some(None)` =
    /// `"role": null` explicite (retire le rôle) ; `Some(Some(r))` = rôle posé.
    ///
    /// Le double `Option` est indispensable : serde considère un `Option<T>`
    /// manquant comme `None`, donc un simple `Option<AccountRole>` aurait
    /// silencieusement **effacé le rôle** d'un compte dès qu'un client omet le
    /// champ — précisément la perte de donnée que le contrat full-replace veut
    /// empêcher.
    #[serde(default, deserialize_with = "present_or_null")]
    pub role: Option<Option<AccountRole>>,
    /// `None` = champ absent → 400. Optionnel côté serde uniquement pour que
    /// l'absence produise la **même** erreur explicite que `role` (un `bool` nu
    /// serait rejeté par serde en 422, incohérent pour le client).
    #[serde(default)]
    pub postable: Option<bool>,
    /// Story 25-2-a ([#382], [#274]) — autorise le changement d'`account_type`
    /// d'un compte **qui porte des écritures**. Absent ≡ `false`.
    ///
    /// ⚠️ **L'asymétrie avec `role` et `postable` ci-dessus est délibérée, et
    /// elle ne contredit pas le contrat full-replace.** Ces deux champs sont
    /// obligatoires parce que leur omission ferait **perdre une donnée en
    /// silence** — un client corrigeant un libellé effacerait le rôle du compte.
    /// Ici l'effet est inverse : le défaut est le défaut **sûr**, celui qui
    /// refuse. Rendre le champ obligatoire casserait tous les clients existants
    /// sans rien protéger de plus.
    ///
    /// Suit le motif `confirm_*` de `routes/bank_imports.rs` — le seul motif de
    /// confirmation explicite du dépôt, qu'il ne faut pas doubler d'un second.
    #[serde(default)]
    pub confirm_account_retype: bool,
    /// Devise (Story 37-1, AC-4) : `None` = champ **absent** → devise
    /// **inchangée** ; `Some(None)` = `null` → CHF ; `Some(Some(code))` = devise.
    ///
    /// ⚠️ **Pas de 400 sur l'absence, à la différence de `role`** : le motif du
    /// full-replace est d'éviter l'effacement silencieux d'une donnée ; ici
    /// « absent = inchangé » n'efface rien, et un 400 casserait tout client
    /// mono-devise qui ne connaît pas le champ. L'absence se résout dans le
    /// dépôt, sous le `FOR UPDATE` du compte.
    #[serde(default, deserialize_with = "present_or_null")]
    pub currency: Option<Option<String>>,
    /// Compte de gains de change propre (Story 37-1, AC-4 bis), mêmes trois
    /// états que `currency` (absent = inchangé, `null` = par défaut) ; va avec
    /// [`Self::fx_loss_account_id`].
    #[serde(default, deserialize_with = "present_or_null")]
    pub fx_gain_account_id: Option<Option<i64>>,
    /// Compte de pertes de change propre (Story 37-1, AC-4 bis).
    #[serde(default, deserialize_with = "present_or_null")]
    pub fx_loss_account_id: Option<Option<i64>>,
    /// Compte parent (correctif `fix-api-parent-compte`), mêmes trois états
    /// que `currency` : `None` = champ **absent** → parent **inchangé** ;
    /// `Some(None)` = `null` → le compte devient racine ; `Some(Some(id))` =
    /// le nouveau parent. Pas de 400 sur l'absence, pour la même raison que
    /// `currency` : un client qui ne connaît pas le champ ne détache rien.
    #[serde(default, deserialize_with = "present_or_null")]
    pub parent_id: Option<Option<i64>>,
    pub version: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveAccountRequest {
    pub version: i32,
    /// Archiver un compte désigné dans les paramètres de facturation (Story 33-8,
    /// #486). Absent : `409 ACCOUNT_DESIGNATED_IN_SETTINGS`, qui nomme les réglages.
    #[serde(default)]
    pub confirm_designated: bool,
}

/// Story 14-3a (#269) — réactivation d'un compte archivé.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactivateAccountRequest {
    pub version: i32,
    /// Réactiver **sans** le rôle porté au moment de l'archivage (code review
    /// 14-3a, D4). `false` par défaut : la réactivation échoue en 409 si le rôle
    /// a été repris entre-temps, ce qui reste le comportement voulu quand
    /// l'utilisateur veut récupérer son rôle. Passer `true` est le geste
    /// « réactiver quand même », proposé par l'interface après ce 409.
    #[serde(default)]
    pub clear_role: bool,
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
    /// Le compte se lettre-t-il (R4 du lettrage, Story 15-1b AC11) : actif ou
    /// passif, et désigné par aucun compte bancaire — archivé compris. Calculé
    /// par la seule règle `letterings::is_letterable`.
    pub letterable: bool,
    /// Devise du compte (Story 37-1, AC-4), code ISO 4217 ; **absente** pour
    /// un compte en CHF (C-37-9 : le JSON reste identique en mono-devise).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// La devise est verrouillée : le compte porte au moins une ligne
    /// d'écriture (Story 37-1, AC-4). **Absent** quand `false`.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub currency_locked: bool,
    /// Compte de gains de change propre (Story 37-1, AC-4 bis) ; absent pour
    /// un compte « par défaut ».
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fx_gain_account_id: Option<i64>,
    /// Compte de pertes de change propre (Story 37-1, AC-4 bis) ; absent pour
    /// un compte « par défaut ».
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fx_loss_account_id: Option<i64>,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl AccountResponse {
    /// `letterable` ne se déduit pas de l'entité seule (un compte bancaire peut
    /// le désigner) : l'appelant le lit par `letterings::is_letterable_account`
    /// (une réponse) ou `letterings::letterable_account_ids` (la liste). De même
    /// `currency_locked` (Story 37-1) : `accounts::is_currency_locked` ou
    /// `accounts::currency_locked_account_ids`.
    pub fn new(a: Account, letterable: bool, currency_locked: bool) -> Self {
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
            letterable,
            currency: a.currency,
            currency_locked,
            fx_gain_account_id: a.fx_gain_account_id,
            fx_loss_account_id: a.fx_loss_account_id,
            version: a.version,
            created_at: a.created_at,
            updated_at: a.updated_at,
        }
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// La réponse d'un compte unique : sa lettrabilité en une requête
/// (`letterings::is_letterable_account`, Story 15-1b AC11) et le verrou de sa
/// devise en une autre (`accounts::is_currency_locked`, Story 37-1).
async fn response(state: &AppState, account: Account) -> Result<AccountResponse, AppError> {
    let mut conn = state.pool.acquire().await.map_err(map_db_error)?;
    let letterable =
        letterings::is_letterable_account(&mut conn, account.company_id, account.id).await?;
    let currency_locked =
        accounts::is_currency_locked(&mut conn, account.company_id, account.id).await?;
    Ok(AccountResponse::new(account, letterable, currency_locked))
}

/// GET /api/v1/accounts — liste les comptes de la company courante.
/// Story 6.2: Scoped by current_user.company_id.
pub async fn list_accounts(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<ListAccountsQuery>,
) -> Result<Json<Vec<AccountResponse>>, AppError> {
    let acces = ContexteAcces::depuis_api(&current_user);
    let comptes = plan_comptable::lister(&acces, &state.lecture(), params.include_archived).await?;
    Ok(Json(
        comptes
            .into_iter()
            .map(|c| AccountResponse::new(c.compte, c.lettrable, c.devise_verrouillee))
            .collect(),
    ))
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

    // Story 37-1 (AC-4, AC-4 bis) — forme, avant la transaction : devise
    // normalisée (`CHF` → CHF, code inconnu → `CURRENCY_UNKNOWN`), puis la
    // paire des comptes propres (`missing`).
    let currency = match req.currency.as_deref() {
        Some(raw) => normalize_currency(raw)?,
        None => None,
    };
    let own_fx_accounts = own_fx_for_create(req.fx_gain_account_id, req.fx_loss_account_id)?;

    if let Some(pid) = req.parent_id {
        check_parent_in_company(&state, pid, company.id).await?;
    }

    let new = NewAccount {
        company_id: company.id,
        number: trimmed_number,
        name: trimmed_name,
        account_type: req.account_type,
        parent_id: req.parent_id,
        role: req.role,
        postable: req.postable,
        currency,
        own_fx_accounts,
    };

    // ⚠️ **Rejouée sur interblocage** (Story 37-1, AC-4 bis, 37-1-CR3-P2-1 ;
    // opération `accounts::create`) : un `POST` qui porte des comptes propres
    // tient `companies` `FOR UPDATE`, puis passe le parent à non imputable
    // (verrou exclusif sur sa ligne) ; une écriture CHF sur ce parent le tient
    // en partage puis demande `companies` en partage à son `INSERT` — cycle.
    // Le dépôt ouvre et conclut sa propre transaction à chaque tentative.
    let user_id = current_user.user_id;
    let account = crate::retry::retry_app_on_deadlock("accounts::create", || {
        let new = new.clone();
        let pool = state.pool.clone();
        async move {
            accounts::create(&pool, user_id, new)
                .await
                .map_err(AppError::from)
        }
    })
    .await?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(response(&state, account).await?),
    ))
}

/// PUT /api/v1/accounts/{id} — modifie un compte : nom, type, rôle,
/// postabilité, devise, comptes propres et parent (`parentId`, correctif
/// `fix-api-parent-compte` : absent = inchangé, `null` = racine).
/// Story 6.2: Validate company exists (defensive: company_id staleness window).
pub async fn update_account(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateAccountRequest>,
) -> Result<Json<AccountResponse>, AppError> {
    // Verify account belongs to current user's company (IDOR check).
    //
    // Code review 14-3a : `archive_account` et `reactivate_account` faisaient
    // cette vérification, `update_account` non — l'`UPDATE` du repository ne
    // porte pas de `company_id`, si bien qu'un utilisateur pouvait modifier le
    // compte d'une autre société. Défaut antérieur à la story, mais celle-ci y
    // fait désormais transiter `role` et `postable` : l'impact passe d'un
    // renommage à la corruption de la configuration comptable d'un autre tenant.
    let _existing = accounts::find_by_id_in_company(&state.pool, id, current_user.company_id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;

    let trimmed_name = req.name.trim().to_string();
    if trimmed_name.is_empty() {
        return Err(AppError::Validation("name must not be empty".into()));
    }
    if trimmed_name.len() > 255 {
        return Err(AppError::Validation(
            "name must not exceed 255 characters".into(),
        ));
    }

    // Contrat full-replace : les deux champs doivent être présents — `role`
    // peut valoir `null` (retire le rôle), mais son absence est une erreur.
    let role = req.role.ok_or_else(|| {
        AppError::Validation(
            "role is required (send `null` to clear it) — full-replace semantics".into(),
        )
    })?;
    let postable = req.postable.ok_or_else(|| {
        AppError::Validation("postable is required — full-replace semantics".into())
    })?;

    // Story 37-1 (AC-4, AC-4 bis) — après le pré-vol existant, avant la
    // transaction : devise normalisée (`CURRENCY_UNKNOWN`), puis la forme des
    // comptes propres (`missing`). Absents = inchangés, résolus par le dépôt.
    let currency = match req.currency {
        None => CurrencyChange::Unchanged,
        Some(None) => CurrencyChange::Base,
        Some(Some(raw)) => match normalize_currency(&raw)? {
            None => CurrencyChange::Base,
            Some(code) => CurrencyChange::Foreign(code),
        },
    };
    let own_fx_accounts = own_fx_for_update(req.fx_gain_account_id, req.fx_loss_account_id, id)?;

    // Correctif `fix-api-parent-compte` — le parent, mêmes contrôles de forme
    // que la création ; le cycle et l'état sous verrou sont jugés par le dépôt.
    let parent = match req.parent_id {
        None => ParentChange::Unchanged,
        Some(None) => ParentChange::Root,
        Some(Some(pid)) => {
            check_parent_in_company(&state, pid, current_user.company_id).await?;
            ParentChange::Under(pid)
        }
    };

    let changes = AccountUpdate {
        name: trimmed_name,
        account_type: req.account_type,
        role,
        postable,
        currency,
        own_fx_accounts,
        parent,
    };

    // Story 25-2-a : le dépôt refuse en 409 `ACCOUNT_HAS_ENTRIES` si le compte
    // porte des écritures et que le client n'a pas confirmé. L'écran n'a rien à
    // pré-vérifier — il réagit au refus, qui lui dit l'ampleur.
    //
    // ⚠️ **Rejouée sur interblocage** (Story 34-11a2, #972, C-e341-40 ; opération
    // `accounts::update`) : le retypage prend la ligne `accounts` `FOR UPDATE`
    // puis lit les comptes bancaires liés sous verrou partagé ; une création,
    // un `PUT` ou un `PATCH` de compte bancaire tient le type en partagé et
    // l'écrivain d'un `INSERT` concurrent peut fermer un cycle avec lui. Le
    // dépôt ouvre et conclut sa propre transaction à chaque tentative.
    let (version, user_id, confirm) = (
        req.version,
        current_user.user_id,
        req.confirm_account_retype,
    );
    let account = crate::retry::retry_app_on_deadlock("accounts::update", || {
        let changes = changes.clone();
        let pool = state.pool.clone();
        async move {
            accounts::update(&pool, id, version, user_id, changes, confirm)
                .await
                .map_err(AppError::from)
        }
    })
    .await?;
    Ok(Json(response(&state, account).await?))
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

    let account = accounts::archive_guarded(
        &state.pool,
        id,
        req.version,
        current_user.user_id,
        req.confirm_designated,
    )
    .await?;
    Ok(Json(response(&state, account).await?))
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

    let account = accounts::reactivate(
        &state.pool,
        id,
        req.version,
        current_user.user_id,
        req.clear_role,
    )
    .await?;
    Ok(Json(response(&state, account).await?))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 37-1-CR-P1-1 : une devise démesurée est refusée, et sa reprise dans la
    /// réponse est bornée ; une saisie courte est reprise telle quelle.
    #[test]
    fn unknown_currency_echo_is_bounded() {
        let long = "é".repeat(5000);
        match normalize_currency(&long) {
            Err(AppError::Database(DbError::CurrencyUnknown { value })) => {
                assert_eq!(value.chars().count(), CURRENCY_ECHO_MAX);
            }
            _ => panic!("devise démesurée non refusée en CURRENCY_UNKNOWN"),
        }
        match normalize_currency("usd1") {
            Err(AppError::Database(DbError::CurrencyUnknown { value })) => {
                assert_eq!(value, "usd1");
            }
            _ => panic!("« usd1 » non refusée en CURRENCY_UNKNOWN"),
        }
        assert!(matches!(normalize_currency(" chf "), Ok(None)));
        assert_eq!(normalize_currency("usd").unwrap().as_deref(), Some("USD"));
    }
}
