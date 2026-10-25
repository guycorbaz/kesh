//! Routes bank-accounts — Story 8-5a-zero (foundation `journal_account_id`)
//! + Story v014-1 (CRUD complet post-onboarding + solde calculé).
//!
//! ## Endpoints
//!
//! - `GET    /api/v1/bank-accounts` — liste les bank_accounts de la company
//!   (tous rôles authentifiés). Query `?includeArchived=true` retourne aussi
//!   les comptes archivés (défaut false). Payload étendu avec `currentBalance`
//!   calculé depuis `journal_entry_lines` (status='Posted', solde toutes années
//!   confondues — cf. FINDING-10 Pass 3 Opus).
//! - `POST   /api/v1/bank-accounts` — création d'un compte bancaire
//!   (Comptable+). Transition primary silencieuse symétrique au PUT. Guard
//!   `step_completed >= 7` (sauf mode demo).
//! - `PUT    /api/v1/bank-accounts/{id}` — édition complète (Comptable+).
//!   Optimistic lock via `version`. Transition primary atomique.
//! - `DELETE /api/v1/bank-accounts/{id}` — soft-delete via `archived=TRUE`
//!   (Comptable+). Refuse 412 si bank_transactions existent OU si primary +
//!   autres comptes actifs.
//! - `PATCH  /api/v1/bank-accounts/{id}` — legacy 8-5a-zero, scope strict
//!   `journal_account_id` (Comptable+). Audit log `details_json.trigger =
//!   "journal_account_link"` (F7 Pass 3 Opus — cohérence audit avec PUT).
//! - `POST   /api/v1/bank-accounts/{id}/close` et `…/reopen` — Story 28-4a
//!   (#642) : clore un compte (lecture seule, historique conservé) et le
//!   rouvrir (Comptable+). `PUT`, `PATCH` et `DELETE` refusent un compte clos
//!   en `409 BANK_ACCOUNT_CLOSED` ; `GET` l'écarte sauf `?includeClosed=true`.
//!
//! Audit log émis depuis le handler (jamais le repo) dans la même transaction
//! que les UPDATEs — atomicité garantie. Court-circuit no-op KF-004 sur le
//! PATCH legacy.

use crate::audit::AuditActor;
use axum::Extension;
use axum::Json;
use axum::extract::Query;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Path, Request, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use kesh_db::entities::account::AccountType;
use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::bank_account::{
    BANK_ACCOUNT_DESCRIPTION_MAX_CHARS, BankAccount, NewBankAccount,
};
use kesh_db::errors::{DbError, NonPostableAccount};
use kesh_db::repositories::bank_accounts::BankAccountPutFields;
use kesh_db::repositories::company_invoice_settings::{self, ClaimAccounts};
use kesh_db::repositories::{accounts, audit_log, bank_accounts, onboarding};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Transaction};

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;

// ===========================================================================
// Payload types
// ===========================================================================

/// Body du PATCH `/bank-accounts/{id}`.
///
/// Story 8-5a-zero : un seul champ mutable v0.1 (`journalAccountId`).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchJournalLinkBody {
    /// `Some(account_id)` lie au compte comptable, `None` délie.
    pub journal_account_id: Option<i64>,
    /// Optimistic lock — version attendue de la row.
    pub version: i32,
}

/// Body du POST `/bank-accounts` — création v014-1.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBankAccountBody {
    pub bank_name: String,
    /// Story 26-2a (#622) — description optionnelle ; absente ou `null` →
    /// pas de description.
    #[serde(default)]
    pub description: Option<String>,
    pub iban: String,
    #[serde(default)]
    pub qr_iban: Option<String>,
    #[serde(default)]
    pub is_primary: bool,
    #[serde(default)]
    pub journal_account_id: Option<i64>,
}

/// Body du PUT `/bank-accounts/{id}` — édition complète v014-1.
///
/// **Remplacement complet** : un champ optionnel absent est effacé — `qrIban`
/// comme `description` (Story 26-2a, C-26-2-8, C-26-2-19).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBankAccountBody {
    pub bank_name: String,
    /// Story 26-2a (#622) — description optionnelle ; **absente ⇒ effacée**.
    #[serde(default)]
    pub description: Option<String>,
    pub iban: String,
    #[serde(default)]
    pub qr_iban: Option<String>,
    #[serde(default)]
    pub is_primary: bool,
    #[serde(default)]
    pub journal_account_id: Option<i64>,
    /// Optimistic lock.
    pub version: i32,
}

/// Body du DELETE `/bank-accounts/{id}` — archive v014-1.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveBankAccountBody {
    /// Optimistic lock.
    pub version: i32,
}

/// Query string du GET `/bank-accounts` — paramètres `includeArchived` et,
/// depuis la Story 28-4a (#642), `includeClosed` (défaut `false` : un compte
/// clos n'est pas proposé là où l'on écrit).
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListBankAccountsQuery {
    #[serde(default)]
    pub include_archived: bool,
    #[serde(default)]
    pub include_closed: bool,
}

/// Body de `POST /bank-accounts/{id}/close` — Story 28-4a (#642, AC3).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseBankAccountBody {
    /// Optimistic lock.
    pub version: i32,
    /// Date de clôture, informative (C-28-4-3) ; absente → date du jour UTC.
    #[serde(default)]
    pub closed_on: Option<chrono::NaiveDate>,
}

/// Body de `POST /bank-accounts/{id}/reopen` — Story 28-4a (#642, AC4).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReopenBankAccountBody {
    /// Optimistic lock.
    pub version: i32,
}

/// Payload étendu retourné par GET `/bank-accounts` — Story v014-1 T5.
///
/// `currentBalance: Option<Decimal>` — `None` si `journal_account_id` n'est
/// pas configuré sur ce compte ; sinon solde calculé `SUM(debit) - SUM(credit)`
/// sur `journal_entry_lines` (toutes les écritures du compte sont par
/// construction validées — pas de colonne `status` v0.1).
///
/// `lastTransactionDate: Option<NaiveDate>` — F13 Pass 1 code review (AC#30) :
/// date `MAX(je.entry_date)` agrégée sur le `journal_account_id` lié.
/// `None` si journal_account_id NULL ou aucune écriture.
///
/// `Decimal` est sérialisé via `serde-str` → JSON string. Le frontend convertit
/// via `Number(item.currentBalance)`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BankAccountWithBalance {
    #[serde(flatten)]
    pub account: BankAccount,
    /// **Solde comptable** du compte de grand livre lié (pas le solde bancaire).
    pub current_balance: Option<Decimal>,
    pub last_transaction_date: Option<chrono::NaiveDate>,
    /// Story 25-6-a (#389) — solde de clôture du relevé dont la période se termine
    /// le plus tard (`period_to` le plus récent, non le dernier importé) qui en porte un (CAMT ; un import CSV n'en a pas). Rendu **même sans compte de
    /// grand livre lié** : le relevé est un fait bancaire, seul l'écart exige le
    /// compte lié.
    pub statement_closing_balance: Option<Decimal>,
    /// Date de ce relevé (`period_to`).
    pub statement_date: Option<chrono::NaiveDate>,
    /// Solde comptable à `statement_date`, corrigé des dates de valeur ; `null`
    /// sans relevé, sans compte lié, ou si plusieurs comptes bancaires partagent
    /// le compte de grand livre.
    pub ledger_balance_at_statement: Option<Decimal>,
    /// Story 29-1a (#232, C-29-1-25) — l'écart entre les livres et le relevé,
    /// `bank_accounts::statement_gap`, au centime ; `null` si nul ou non
    /// calculable. Seule définition de l'écart : la page d'accueil l'affiche
    /// sans le recalculer, le contrôle K3 de la préparation de clôture le lit.
    pub statement_gap: Option<Decimal>,
}

// ===========================================================================
// Body extractors (convertit serde rejection en AppError::Validation 400)
// ===========================================================================

/// Macro factor extractor : convertit serde rejection en `AppError::Validation`
/// (400 standard Kesh) au lieu du 422 Axum natif.
macro_rules! impl_validation_extractor {
    ($extractor:ident, $body:ident, $field_doc:expr) => {
        pub struct $extractor(pub $body);

        impl<S> FromRequest<S> for $extractor
        where
            S: Send + Sync,
        {
            type Rejection = Response;

            async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
                match Json::<$body>::from_request(req, state).await {
                    Ok(Json(body)) => Ok(Self(body)),
                    Err(rej) => {
                        let message = match &rej {
                            JsonRejection::JsonDataError(_) | JsonRejection::JsonSyntaxError(_) => {
                                format!("corps JSON malformé ou champ requis manquant ({})", $field_doc)
                            }
                            JsonRejection::MissingJsonContentType(_) => {
                                "Content-Type attendu : application/json".to_string()
                            }
                            _ => {
                                tracing::warn!(rejection = %rej, "BodyExtractor: unhandled JsonRejection");
                                "requête invalide (corps non-parsable)".to_string()
                            }
                        };
                        Err(AppError::Validation(message).into_response())
                    }
                }
            }
        }
    };
}

impl_validation_extractor!(
    PatchJournalLinkBodyExtractor,
    PatchJournalLinkBody,
    "`journalAccountId` Option<i64>, `version` i32"
);
impl_validation_extractor!(
    CreateBankAccountBodyExtractor,
    CreateBankAccountBody,
    "`bankName` str, `description?` str, `iban` str, `qrIban?` str, `isPrimary?` bool, `journalAccountId?` i64"
);
impl_validation_extractor!(
    UpdateBankAccountBodyExtractor,
    UpdateBankAccountBody,
    "`bankName` str, `description?` str, `iban` str, `qrIban?` str, `isPrimary?` bool, `journalAccountId?` i64, `version` i32"
);
impl_validation_extractor!(
    ArchiveBankAccountBodyExtractor,
    ArchiveBankAccountBody,
    "`version` i32"
);
impl_validation_extractor!(
    CloseBankAccountBodyExtractor,
    CloseBankAccountBody,
    "`version` i32, `closedOn?` date AAAA-MM-JJ"
);
impl_validation_extractor!(
    ReopenBankAccountBodyExtractor,
    ReopenBankAccountBody,
    "`version` i32"
);

// ===========================================================================
// Helpers communs
// ===========================================================================

/// Guard onboarding (FINDING-11 Pass 3 Opus + F1 Pass 1 code review) — refuse
/// les CRUD post-onboarding si l'onboarding n'est pas terminé (step_completed
/// < 7). Mode demo autorise le CRUD (cohérent UX demo = full feature set).
///
/// **F1 Pass 1 code review** : `None` (DB fresh, onboarding_state jamais
/// initialisée) → `Err(OnboardingNotComplete)`. Defense-in-depth : en théorie
/// le middleware 423 Locked redirige vers `/setup` avant tout call de ces
/// endpoints, mais si `users_exist=true` sans `onboarding_state` (cas
/// pathologique post-recovery par exemple), le guard doit refuser le CRUD.
async fn assert_onboarding_complete(pool: &sqlx::MySqlPool) -> Result<(), AppError> {
    let state = onboarding::get_state(pool).await?;
    match state {
        Some(s) if s.step_completed >= 7 || s.is_demo => Ok(()),
        Some(_) | None => Err(AppError::OnboardingNotComplete),
    }
}

/// Champs métier d'un payload create/update, **normalisés** par
/// [`validate_bank_input`].
struct ValidatedBankInput {
    bank_name: String,
    /// Story 26-2a — `None` si absente, vide ou faite d'espaces.
    description: Option<String>,
    iban: String,
    qr_iban: Option<String>,
}

/// Caractères refusés dans une description de compte bancaire — Story 26-2a
/// (#622, C-26-2-17, C-26-2-28).
///
/// Depuis la Story 32-8b0, c'est le prédicat partagé d'une **valeur d'une
/// ligne**, [`kesh_core::text::is_hidden_in_line`] : tout contrôle Cc (dont
/// `\n`, `\r`, `\t`) et tout caractère caché de
/// [`kesh_core::text::is_hidden_format`] — catégories Unicode Cf, Co, Cn et
/// deux listes écrites d'invisibles d'autres catégories. Il remplace la liste
/// fermée de la 26-2a, dont chaque point reste refusé (test
/// `ancienne_liste_reste_refusee` de `kesh-core`) ; l'angle mort qui reste est
/// écrit sur `is_hidden_format`.
fn is_forbidden_in_description(c: char) -> bool {
    kesh_core::text::is_hidden_in_line(c)
}

/// Normalise et valide la description d'un compte bancaire — Story 26-2a
/// (#622, AC3).
///
/// Absente, vide ou faite d'espaces → `None`. Sinon `trim()` **puis** contrôles
/// (C-26-2-28 : `str::trim` retire tout `White_Space` des bords, donc
/// `"Épargne\n"` est accepté et stocké `"Épargne"`) : au plus
/// [`BANK_ACCOUNT_DESCRIPTION_MAX_CHARS`] caractères Unicode, aucun caractère
/// de [`is_forbidden_in_description`].
fn normalize_description(description: Option<&str>) -> Result<Option<String>, AppError> {
    let Some(trimmed) = description.map(str::trim).filter(|d| !d.is_empty()) else {
        return Ok(None);
    };
    if trimmed.chars().count() > BANK_ACCOUNT_DESCRIPTION_MAX_CHARS {
        return Err(AppError::Validation(format!(
            "La description ne peut pas dépasser {BANK_ACCOUNT_DESCRIPTION_MAX_CHARS} caractères"
        )));
    }
    if trimmed.chars().any(is_forbidden_in_description) {
        return Err(AppError::Validation(
            "La description ne peut pas contenir de saut de ligne ni de caractère de contrôle ou de mise en page"
                .into(),
        ));
    }
    Ok(Some(trimmed.to_string()))
}

/// Valide et normalise les champs métier d'un payload create/update.
///
/// **Ordre des refus de forme** (Story 26-2a, AC3) : banque vide → description
/// → IBAN → QR-IBAN.
fn validate_bank_input(
    bank_name: &str,
    description: Option<&str>,
    iban: &str,
    qr_iban: Option<&str>,
) -> Result<ValidatedBankInput, AppError> {
    let bank_name = bank_name.trim().to_string();
    if bank_name.is_empty() {
        return Err(AppError::Validation(
            "Le nom de la banque ne peut pas être vide".into(),
        ));
    }

    let description = normalize_description(description)?;

    let iban_normalized = kesh_core::types::Iban::new(iban)
        .map_err(|e| AppError::Validation(format!("IBAN invalide : {e}")))?
        .as_str()
        .to_string();

    let qr_iban_normalized = match qr_iban {
        Some(qr) if !qr.trim().is_empty() => {
            let qr_obj = kesh_core::types::QrIban::new(qr)
                .map_err(|e| AppError::Validation(format!("QR-IBAN invalide : {e}")))?;
            Some(qr_obj.as_iban().as_str().to_string())
        }
        _ => None,
    };

    Ok(ValidatedBankInput {
        bank_name,
        description,
        iban: iban_normalized,
        qr_iban: qr_iban_normalized,
    })
}

/// Validation pré-flight du `journalAccountId` (compte existe + actif +
/// Asset|Liability) — cohérent pattern PATCH.
///
/// `require_postable` (Story 15-5b, AC12, #427) : vrai à la **création**, où
/// un compte non imputable (regroupement, résultat, clôture) est refusé en 400
/// `ACCOUNT_NOT_POSTABLE` **après** les contrôles d'existence (404) et de type
/// (400). Faux au **remplacement** (PUT) et au **lien** (PATCH) : là, la
/// postabilité est contrôlée dans la transaction du dépôt, sous le verrou de
/// la ligne et **seulement si la valeur change** (choix C10) — un compte lié
/// devenu non imputable après coup reste accepté tel quel (D-A0, choix C6).
async fn validate_journal_account_id(
    pool: &sqlx::MySqlPool,
    company_id: i64,
    journal_account_id: Option<i64>,
    require_postable: bool,
) -> Result<(), AppError> {
    let Some(account_id) = journal_account_id else {
        return Ok(());
    };

    if account_id <= 0 {
        return Err(AppError::Validation(
            "journalAccountId doit être strictement positif".to_string(),
        ));
    }
    let account = accounts::find_by_id_in_company(pool, account_id, company_id)
        .await?
        .ok_or(AppError::AccountNotFound {
            account_id,
            missing_account_ids: None,
        })?;

    if !account.active {
        return Err(AppError::AccountNotFound {
            account_id,
            missing_account_ids: None,
        });
    }

    match account.account_type {
        AccountType::Asset | AccountType::Liability => {}
        other => {
            return Err(AppError::InvalidAccountType {
                account_id,
                account_type: other.as_str().to_string(),
            });
        }
    }

    if require_postable && !account.postable {
        return Err(AppError::Database(DbError::accounts_not_postable([
            NonPostableAccount {
                account_id,
                account_number: account.number,
            },
        ])));
    }
    Ok(())
}

/// Les comptes de créance des réglages, lus **sous verrou partagé** si la cible
/// du lien est un compte — sinon `ClaimAccounts::default()`, sans lecture ni
/// verrou (Story 15-6c, AC2 ; choix C-15-6-27) : une dé-liaison, une création
/// ou un remplacement sans compte lié n'attendent rien.
///
/// ⛔ **À appeler avant tout `FOR UPDATE` sur `bank_accounts`** (ordre global
/// `companies` → réglages → `bank_accounts`) : juste après la sentinelle à la
/// création et au remplacement, en première instruction au lien.
async fn claims_for_target(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    journal_account_id: Option<i64>,
) -> Result<ClaimAccounts, AppError> {
    match journal_account_id {
        Some(_) => {
            Ok(company_invoice_settings::claim_accounts_in_share_mode(tx, company_id).await?)
        }
        None => Ok(ClaimAccounts::default()),
    }
}

/// Émet l'audit log `bank_account.updated` `trigger=primary_transition` sur
/// l'ancien primary démoté (helper FINDING-3 Pass 3 Opus).
///
/// **F15 Pass 1 code review** : reçoit `(updated, before)` snapshot — pas
/// d'arithmétique sur `version` (cf. doc-comment `flip_primary_off_for_company`).
async fn audit_primary_transition(
    tx: &mut Transaction<'_, MySql>,
    user_id: i64,
    actor_api_key_id: Option<i64>,
    updated: &BankAccount,
    before: &BankAccount,
    new_primary_id: i64,
) -> Result<(), AppError> {
    let details = serde_json::json!({
        "bank_account_id": updated.id,
        "trigger": "primary_transition",
        "new_primary_id": new_primary_id,
        "before": { "is_primary": before.is_primary, "version": before.version },
        "after": { "is_primary": updated.is_primary, "version": updated.version },
    });
    // Story 17-2a (DC5 cat ii) — `actor_api_key_id` threadé depuis le handler
    // appelant (current_user.api_key_id) : attribue la mutation au PAT le cas échéant.
    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::for_actor(
            user_id,
            actor_api_key_id,
            "bank_account.updated",
            "bank_account",
            updated.id,
            Some(details),
        ),
    )
    .await?;
    Ok(())
}

// ===========================================================================
// Handlers
// ===========================================================================

/// Handler `GET /api/v1/bank-accounts` — liste les bank_accounts de la
/// company courante (multi-tenant scoping via `current_user.company_id`).
///
/// Story v014-1 T5 — payload étendu avec `currentBalance` calculé.
/// Query `?includeArchived=true` retourne aussi les archivés (défaut false).
///
/// **Périmètre du solde calculé** (F4 Pass 1 code review — clarification
/// vs F10 Pass 3 Opus) : `journal_entries` n'a **pas** de colonne `status`
/// dans le schéma v0.1 — toute écriture insérée est par construction validée
/// (la double-partie est balanced + toutes les FK existent + pas de notion
/// de draft). Le calcul `SUM(debit) - SUM(credit)` agrège donc toutes les
/// `journal_entry_lines` du `journal_account_id` lié, sans filtre de statut.
/// La spec F10 Pass 3 Opus prévoyait initialement un filtre `status='Posted'`
/// mais la vérification du schéma a invalidé cette hypothèse. Pas de filtre
/// `fiscal_year_id` non plus (« solde depuis création » v0.1).
pub async fn list_bank_accounts(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ListBankAccountsQuery>,
) -> Result<Json<Vec<BankAccountWithBalance>>, AppError> {
    let rows = bank_accounts::list_by_company_with_balances(
        &state.pool,
        current_user.company_id,
        bank_accounts::BankAccountListFilter {
            include_archived: query.include_archived,
            include_closed: query.include_closed,
        },
    )
    .await?;

    let payload: Vec<BankAccountWithBalance> = rows
        .into_iter()
        .map(|(account, balances)| BankAccountWithBalance {
            account,
            statement_gap: bank_accounts::statement_gap(&balances),
            current_balance: balances.current_balance,
            last_transaction_date: balances.last_transaction_date,
            statement_closing_balance: balances.statement_closing_balance,
            statement_date: balances.statement_date,
            ledger_balance_at_statement: balances.ledger_balance_at_statement,
        })
        .collect();

    Ok(Json(payload))
}

/// Handler `POST /api/v1/bank-accounts` — création v014-1 (Comptable+).
///
/// Transition primary silencieuse symétrique au PUT (FINDING-3 Pass 3 Opus).
/// Guard onboarding step >= 7 (sauf is_demo).
///
/// Story 15-5b (AC12, #427) : un `journalAccountId` non imputable est refusé
/// en 400 `ACCOUNT_NOT_POSTABLE`, après le 404 et le 400 de type.
///
/// **Ordre des erreurs** (Story 15-6c, AC6) : forme (400) → 404 / 400 type /
/// 400 `ACCOUNT_NOT_POSTABLE` du compte lié (`validate_journal_account_id`,
/// hors transaction) → 400 `BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT` (compte lié =
/// compte débiteurs ou créanciers désigné, contrôlé dans la transaction,
/// sous le verrou partagé des réglages). Un refus ne crée rien et ne démote
/// aucun principal.
///
/// ⚠️ **Rejouée sur interblocage** (opération `bank_accounts::create`, Story
/// 29-4a, revue de code P1, E-1, #651) ; la transaction vit dans
/// [`create_once`]. Comme le `PUT` et l'archivage, la création tient la
/// sentinelle `companies` en exclusif avant de prendre d'autres verrous (les
/// partagés des réglages, l'ancien principal qu'elle démote) : un import ou
/// un lot qui tient la ligne d'un compte en partagé par son `INSERT` et
/// demande `companies` ferme un cycle avec elle quand elle démote ce compte
/// principal — préexistant ; plus légère que l'écrivain, elle en était la
/// victime, en `500`. ⚠️ Raisonné, non mesuré (Story 29-4b, T6-bis, R-4) : la
/// démotion balaie l'index `fk_bank_accounts_company`, son exclusif peut
/// couvrir d'autres comptes de la société (aucun `EXPLAIN` au dépôt), et
/// l'ordre des contrôles de clé étrangère de l'`INSERT` d'un import est
/// supposé ; T29a-15 joue un cycle synthétique, non la démotion. Les
/// contrôles de forme, l'onboarding et
/// `validate_journal_account_id` (sur le pool) restent devant l'enveloppe.
pub async fn create_bank_account(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    CreateBankAccountBodyExtractor(body): CreateBankAccountBodyExtractor,
) -> Result<(StatusCode, Json<BankAccount>), AppError> {
    assert_onboarding_complete(&state.pool).await?;

    let input = validate_bank_input(
        &body.bank_name,
        body.description.as_deref(),
        &body.iban,
        body.qr_iban.as_deref(),
    )?;
    validate_journal_account_id(
        &state.pool,
        current_user.company_id,
        body.journal_account_id,
        true,
    )
    .await?;

    let (is_primary, journal_account_id) = (body.is_primary, body.journal_account_id);
    let inserted = crate::retry::retry_app_on_deadlock("bank_accounts::create", || {
        create_once(
            &state.pool,
            &current_user,
            &input,
            is_primary,
            journal_account_id,
        )
    })
    .await?;
    Ok((StatusCode::CREATED, Json(inserted)))
}

/// Une tentative de la création : sentinelle `companies`, refus du compte de
/// créance, démotion de l'ancien principal, `INSERT`, traces — conclusion par
/// [`conclure`] (`rollback` explicite sur refus). Chaque tentative clone ce
/// qu'elle consomme de `input`.
async fn create_once(
    pool: &sqlx::MySqlPool,
    current_user: &CurrentUser,
    input: &ValidatedBankInput,
    is_primary: bool,
    journal_account_id: Option<i64>,
) -> Result<BankAccount, AppError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;
    let result = async {
        // Advisory lock sentinel (L5 mitigation — FINDING-9 Pass 3 Opus).
        bank_accounts::acquire_company_sentinel_lock(&mut tx, current_user.company_id).await?;

        // Story 15-6c (AC3) — un compte bancaire ne se lie pas au compte de
        // créance désigné. Premier verrou après la sentinelle, avant la démotion
        // de l'ancien principal : un refus ne crée rien et ne démote rien.
        if let Some(account_id) = journal_account_id {
            let claims = company_invoice_settings::claim_accounts_in_share_mode(
                &mut tx,
                current_user.company_id,
            )
            .await?;
            bank_accounts::refuse_if_ledger_is_claim_account(
                &mut tx,
                current_user.company_id,
                account_id,
                &claims,
            )
            .await?;
        }

        // Si is_primary=true et un autre primary existe → flip silencieux atomique.
        // L'INSERT n'a pas encore d'id, on passe -1 comme excluded_id sentinel
        // (les ids MariaDB AUTO_INCREMENT sont toujours > 0 — F8 Pass 1 code review,
        // plus défensif que 0 qui pourrait coïncider avec un id dans des fixtures
        // pathologiques avec `AUTO_INCREMENT=0`).
        let demoted_primary = if is_primary {
            bank_accounts::flip_primary_off_for_company(&mut tx, current_user.company_id, -1)
                .await
                .map_err(AppError::Database)?
        } else {
            None
        };

        // INSERT du nouveau compte.
        let new = NewBankAccount {
            company_id: current_user.company_id,
            bank_name: input.bank_name.clone(),
            iban: input.iban.clone(),
            qr_iban: input.qr_iban.clone(),
            is_primary,
        };

        let result = sqlx::query(
            "INSERT INTO bank_accounts (company_id, bank_name, description, iban, qr_iban, is_primary, journal_account_id) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(new.company_id)
        .bind(&new.bank_name)
        .bind(&input.description)
        .bind(&new.iban)
        .bind(&new.qr_iban)
        .bind(new.is_primary)
        .bind(journal_account_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(kesh_db::errors::map_db_error(e)))?;

        let new_id = i64::try_from(result.last_insert_id())
            .map_err(|_| AppError::Internal("last_insert_id overflow".into()))?;

        let inserted = bank_accounts::find_by_id_in_tx(&mut tx, current_user.company_id, new_id)
            .await
            .map_err(AppError::Database)?;

        // Audit log primary_transition sur l'ancien primary démoté, puis création.
        if let Some((updated_old, before_old)) = demoted_primary.as_ref() {
            audit_primary_transition(
                &mut tx,
                current_user.user_id,
                current_user.api_key_id,
                updated_old,
                before_old,
                new_id,
            )
            .await?;
        }

        let details = serde_json::json!({
            "bank_account_id": new_id,
            "description": inserted.description,
            "is_primary": inserted.is_primary,
            "iban_present": true,
            "qr_iban_present": inserted.qr_iban.is_some(),
            "journal_account_id": inserted.journal_account_id,
        });
        audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::from_current_user(
                current_user,
                "bank_account.created",
                "bank_account",
                new_id,
                Some(details),
            ),
        )
        .await?;
        Ok(inserted)
    }
    .await;
    conclure(tx, result).await
}

/// Handler `PUT /api/v1/bank-accounts/{id}` — édition complète v014-1
/// (Comptable+).
///
/// **Remplacement complet** (Story 26-2a, C-26-2-8, C-26-2-19) : un champ
/// optionnel absent du corps est **effacé** — `qrIban` comme `description`.
/// Changement de contrat pour une intégration par clé d'API écrite pour la
/// 0.13.0 : un corps sans `description` efface celle saisie à l'écran.
///
/// **Ordre des erreurs** (Story 15-5b, AC12 ; Story 15-6c, AC6) : forme (400)
/// → 404 / 400 type du compte lié (`validate_journal_account_id`, hors
/// transaction) → 404 compte bancaire → 409 version → 400
/// `ACCOUNT_NOT_POSTABLE` (compte lié **changé** vers un compte non imputable,
/// contrôlé par le dépôt sous verrou) → 400 `BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT`
/// (compte lié **changé** vers le compte débiteurs ou créanciers désigné). Un
/// refus abandonne la transaction : l'ancien principal, s'il avait été démoté,
/// le reste.
///
/// ⚠️ **Rejouée sur interblocage** (opération `bank_accounts::update`, Story
/// 29-4a, revue de code P1, E-1, #651) ; la transaction vit dans
/// [`update_once`]. Le `PUT` tient la sentinelle `companies` en exclusif puis
/// demande la ligne du compte en exclusif (`update_for_company`) : un import
/// ou un lot qui tient cette ligne en partagé par le premier contrôle de clé
/// étrangère de son `INSERT` et demande `companies` (le second — ordre des
/// contrôles supposé, non mesuré : voir `recheck_open_locked_in_tx`) ferme un
/// cycle avec lui — à deux, ou à trois par la file quand la clôture, la
/// réouverture ou le `PATCH` attend la ligne entre eux. Préexistant ; plus
/// léger que l'écrivain, le `PUT` en était la victime, en `500`. Les contrôles
/// de forme, l'onboarding et `validate_journal_account_id` (sur le pool)
/// restent devant l'enveloppe.
pub async fn update_bank_account(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    UpdateBankAccountBodyExtractor(body): UpdateBankAccountBodyExtractor,
) -> Result<Json<BankAccount>, AppError> {
    assert_onboarding_complete(&state.pool).await?;

    if body.version < 1 {
        return Err(AppError::Validation("version doit être >= 1".to_string()));
    }

    let input = validate_bank_input(
        &body.bank_name,
        body.description.as_deref(),
        &body.iban,
        body.qr_iban.as_deref(),
    )?;
    validate_journal_account_id(
        &state.pool,
        current_user.company_id,
        body.journal_account_id,
        // Postabilité contrôlée dans le dépôt, sous verrou, si changée (AC12).
        false,
    )
    .await?;

    let (is_primary, journal_account_id, version) =
        (body.is_primary, body.journal_account_id, body.version);
    let updated = crate::retry::retry_app_on_deadlock("bank_accounts::update", || {
        update_once(
            &state.pool,
            &current_user,
            id,
            &input,
            is_primary,
            journal_account_id,
            version,
        )
    })
    .await?;
    Ok(Json(updated))
}

/// Une tentative du `PUT` : sentinelle `companies`, partagés des réglages
/// (`claims_for_target`), démotion de l'ancien principal, `FOR UPDATE` et
/// `UPDATE` du compte (`update_for_company`), traces — conclusion par
/// [`conclure`] (`rollback` explicite sur refus). Chaque tentative clone ce
/// qu'elle consomme de `input`.
async fn update_once(
    pool: &sqlx::MySqlPool,
    current_user: &CurrentUser,
    id: i64,
    input: &ValidatedBankInput,
    is_primary: bool,
    journal_account_id: Option<i64>,
    version: i32,
) -> Result<BankAccount, AppError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;
    let result = async {
        bank_accounts::acquire_company_sentinel_lock(&mut tx, current_user.company_id).await?;

        // Story 15-6c (AC2, AC4) — les comptes de créance, premier verrou après la
        // sentinelle ; la comparaison se fait dans le dépôt, sous le verrou de la
        // ligne, seulement si le compte lié change.
        let claims =
            claims_for_target(&mut tx, current_user.company_id, journal_account_id).await?;

        // Si l'utilisateur veut promouvoir ce compte primary, flip l'éventuel
        // ancien (avant l'UPDATE de notre row, dans la même tx).
        let demoted_primary = if is_primary {
            bank_accounts::flip_primary_off_for_company(&mut tx, current_user.company_id, id)
                .await
                .map_err(AppError::Database)?
        } else {
            None
        };

        let new = NewBankAccount {
            company_id: current_user.company_id,
            bank_name: input.bank_name.clone(),
            iban: input.iban.clone(),
            qr_iban: input.qr_iban.clone(),
            is_primary,
        };

        let (updated, before) = match bank_accounts::update_for_company(
            &mut tx,
            current_user.company_id,
            id,
            &new,
            BankAccountPutFields {
                journal_account_id,
                description: input.description.as_deref(),
            },
            version,
            &claims,
        )
        .await
        {
            Ok(pair) => pair,
            Err(DbError::NotFound) => return Err(AppError::BankAccountNotFound),
            Err(e) => return Err(AppError::Database(e)),
        };

        if let Some((updated_old, before_old)) = demoted_primary.as_ref() {
            audit_primary_transition(
                &mut tx,
                current_user.user_id,
                current_user.api_key_id,
                updated_old,
                before_old,
                id,
            )
            .await?;
        }

        let details = serde_json::json!({
            "bank_account_id": id,
            "trigger": "full_update",
            "before": {
                "bank_name": before.bank_name,
                "description": before.description,
                "iban_present": true,
                "qr_iban_present": before.qr_iban.is_some(),
                "is_primary": before.is_primary,
                "journal_account_id": before.journal_account_id,
                "version": before.version,
            },
            "after": {
                "bank_name": updated.bank_name,
                "description": updated.description,
                "iban_present": true,
                "qr_iban_present": updated.qr_iban.is_some(),
                "is_primary": updated.is_primary,
                "journal_account_id": updated.journal_account_id,
                "version": updated.version,
            },
        });
        audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::from_current_user(
                current_user,
                "bank_account.updated",
                "bank_account",
                id,
                Some(details),
            ),
        )
        .await?;
        Ok(updated)
    }
    .await;
    conclure(tx, result).await
}

/// Handler `DELETE /api/v1/bank-accounts/{id}` — soft-delete (archive) v014-1
/// (Comptable+).
///
/// AC#8 : refus 412 BANK_ACCOUNT_HAS_TRANSACTIONS si transactions existent.
/// AC#9 : refus 412 BANK_ACCOUNT_CANNOT_ARCHIVE_PRIMARY si primary + autres.
/// AC#10 : autorisé pour primary unique (cas dégénéré).
///
/// ⚠️ **Rejouée sur interblocage** (opération `bank_accounts::archive`, Story
/// 29-4a, revue de code P1, E-1, #651) ; la transaction vit dans
/// [`archive_once`]. Même cycle que le `PUT` : la sentinelle `companies` en
/// exclusif, puis la ligne du compte en exclusif (prise depuis la Story 29-4b
/// par `primary_flag_locked_in_tx`, dans la garde du principal, et redemandée
/// par `archive_for_company`), contre un import ou un lot qui tient cette
/// ligne en partagé par son `INSERT` et demande `companies`. Depuis la revérification de l'import et
/// du lot, aucun d'eux n'aboutit plus sur un compte archivé ; le rejeu ôte à
/// la course son dernier effet, le `500` de l'archivage. Le pré-vol (`404`,
/// `409 BANK_ACCOUNT_CLOSED`, sur le pool) reste devant l'enveloppe.
pub async fn archive_bank_account(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    ArchiveBankAccountBodyExtractor(body): ArchiveBankAccountBodyExtractor,
) -> Result<Json<BankAccount>, AppError> {
    assert_onboarding_complete(&state.pool).await?;

    if body.version < 1 {
        return Err(AppError::Validation("version doit être >= 1".to_string()));
    }

    // Pré-flight hors-tx pour 404 rapide (économise begin tx si compte
    // inexistant/cross-tenant/déjà archivé). Les guards "fonds" (transactions
    // existantes, primary + autres actifs) sont déplacés DANS la tx pour
    // éliminer la fenêtre TOCTOU (F2 Pass 1 code review).
    let existing = bank_accounts::find_by_id_for_company(&state.pool, current_user.company_id, id)
        .await?
        .ok_or(AppError::BankAccountNotFound)?;
    if existing.archived {
        // Idempotence v0.1 non-supportée — déjà archivé → 404 anti-énumération.
        return Err(AppError::BankAccountNotFound);
    }
    // Story 28-4a (#642, AC5) — un compte clos ne s'archive pas : il faut le
    // rouvrir d'abord. Lu sur ce pré-vol existant (aucun verrou ajouté),
    // AVANT la sentinelle et le décompte : un compte clos qui porte des imports
    // rend `409 BANK_ACCOUNT_CLOSED`, non le `412` dont le message propose la
    // clôture. Une clôture commitée après ce pré-vol est refusée par
    // `archive_for_company`, sous le verrou de la ligne (pris dès
    // `primary_flag_locked_in_tx`, Story 29-4b).
    if existing.is_closed() {
        return Err(AppError::BankAccountClosed {
            bank_account_id: id,
        });
    }

    let version = body.version;
    let updated = crate::retry::retry_app_on_deadlock("bank_accounts::archive", || {
        archive_once(&state.pool, &current_user, id, version)
    })
    .await?;
    Ok(Json(updated))
}

/// Une tentative de l'archivage : sentinelle `companies`, gardes « fonds »
/// (transactions ; principal, sous le `FOR UPDATE` de la ligne pris par
/// `primary_flag_locked_in_tx` ; autres comptes actifs), puis `UPDATE` du
/// compte (`archive_for_company`, qui redemande le verrou), trace — conclusion par
/// [`conclure`] (`rollback` explicite sur refus).
///
/// Story 29-4b (T6-bis, R-2, #651) — `is_primary` est **relu dans chaque
/// tentative**, sous le `FOR UPDATE` de la ligne
/// (`bank_accounts::primary_flag_locked_in_tx`), et non plus pris au pré-vol :
/// le pré-vol, hors transaction, et le rejeu, qui le réutilisait à chaque
/// tentative, laissaient la garde décider sur un état périmé (test
/// `archive_rereads_the_primary_flag_inside_its_transaction`).
async fn archive_once(
    pool: &sqlx::MySqlPool,
    current_user: &CurrentUser,
    id: i64,
    version: i32,
) -> Result<BankAccount, AppError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;
    let result = async {
        // Advisory lock sentinel — serialize les mutations CRUD bank_accounts d'un
        // même tenant ET protège contre une création concurrente de
        // bank_transactions via le même chemin (mais bank_imports n'acquiert pas
        // ce lock — L5 limitation v0.1 documentée) ; la course qu'il laissait est
        // fermée par la revérification de l'import et du lot (Story 29-4).
        bank_accounts::acquire_company_sentinel_lock(&mut tx, current_user.company_id).await?;

        // Guard transactions (AC#8) — DANS la tx, après sentinel lock.
        let tx_count = bank_accounts::count_transactions_for_bank_account(
            &mut tx,
            current_user.company_id,
            id,
        )
        .await?;
        if tx_count > 0 {
            return Err(AppError::BankAccountHasTransactions {
                transaction_count: tx_count,
            });
        }

        // Guard primary + autres comptes actifs (AC#9 / AC#10) — DANS la tx, sur
        // `is_primary` lu sous le `FOR UPDATE` de la ligne (Story 29-4b, R-2).
        // Compte absent ou archivé entre-temps : 404, le refus même
        // qu'`archive_for_company` aurait rendu sur la même ligne.
        let was_primary =
            bank_accounts::primary_flag_locked_in_tx(&mut tx, current_user.company_id, id)
                .await?
                .ok_or(AppError::BankAccountNotFound)?;
        if was_primary {
            let others =
                bank_accounts::count_other_active_for_company(&mut tx, current_user.company_id, id)
                    .await?;
            if others > 0 {
                return Err(AppError::BankAccountCannotArchivePrimary);
            }
            // others == 0 : primary unique → autorisé (AC#10).
        }

        let (updated, before) =
            match bank_accounts::archive_for_company(&mut tx, current_user.company_id, id, version)
                .await
            {
                Ok(pair) => pair,
                Err(DbError::NotFound) => return Err(AppError::BankAccountNotFound),
                Err(e) => return Err(AppError::Database(e)),
            };

        let details = serde_json::json!({
            "bank_account_id": id,
            "before": { "archived": before.archived, "version": before.version },
            "after": { "archived": updated.archived, "version": updated.version },
            "was_primary": before.is_primary,
        });
        audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::from_current_user(
                current_user,
                "bank_account.archived",
                "bank_account",
                id,
                Some(details),
            ),
        )
        .await?;
        Ok(updated)
    }
    .await;
    conclure(tx, result).await
}

/// Traduit le refus d'une clôture ou d'une réouverture (Story 28-4a, #642).
fn closure_refusal_error(refusal: bank_accounts::ClosureRefusal) -> AppError {
    use bank_accounts::ClosureRefusal;
    match refusal {
        ClosureRefusal::NotClosed => AppError::BankAccountNotClosed,
        ClosureRefusal::Primary => AppError::BankAccountCannotClosePrimary,
        ClosureRefusal::PendingTransactions { pending_count } => {
            AppError::BankAccountHasPendingTransactions { pending_count }
        }
        ClosureRefusal::OpenPaymentBatches { batch_count } => {
            AppError::BankAccountHasOpenPaymentBatches { batch_count }
        }
    }
}

/// Handler `POST /api/v1/bank-accounts/{id}/close` — **clôt** un compte
/// bancaire : il passe en lecture seule, son historique reste entier
/// (Comptable+, clés d'API `read-write` admises ; Story 28-4a, #642, AC3).
///
/// Corps `{ "version", "closedOn"? }` ; `closedOn` absent → date du jour
/// **UTC** (l'écran l'envoie toujours). Effet : `closed_on = <date>`,
/// `version + 1`, rien d'autre. Trace `bank_account.closed`.
///
/// **Ordre des refus** : forme (`400` : `version < 1`, date illisible, ou année
/// de `closedOn` hors de `1000..=9999`, le domaine de `DATE`) →
/// `404 BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND`
/// (inexistant, autre société, archivé) → sous verrou : `409
/// OPTIMISTIC_LOCK_CONFLICT` → `409 BANK_ACCOUNT_CLOSED` (déjà clos) → `412
/// BANK_ACCOUNT_CANNOT_CLOSE_PRIMARY` → `412 BANK_ACCOUNT_HAS_PENDING_TRANSACTIONS`
/// → `412 BANK_ACCOUNT_HAS_OPEN_PAYMENT_BATCHES`.
///
/// ⛔ La route ne lit **rien** dans la transaction avant
/// `bank_accounts::close_in_tx` : le `FOR UPDATE` de la ligne est sa première
/// instruction, et c'est ce qui fait voir aux décomptes tout écrivain commité
/// (voir le doc-comment de `close_in_tx`).
///
/// ⚠️ **Rejouée sur interblocage** par [`crate::retry::retry_app_on_deadlock`]
/// (opération `bank_accounts::close`, Story 29-4a, #651, AC4) ; la transaction
/// vit dans [`close_once`]. La route ne tient rien **comme détenteur**, mais sa
/// demande exclusive **en attente** fait attendre derrière elle toute demande
/// **neuve** de la ligne du compte — non la revérification de l'import et du
/// lot, redemande d'un partagé déjà tenu, qui passe (T29a-6, T29a-7). Cycle
/// atteignable en 29-4a, préexistant : un import ou un lot `I` tient la ligne
/// en partagé par le premier contrôle de clé étrangère de son `INSERT` et
/// demande `companies` (le second — ordre des contrôles supposé, non mesuré :
/// voir `recheck_open_locked_in_tx`) ; une route de gestion `M` (`PUT`,
/// archivage) tient la sentinelle `companies` et demande la ligne en exclusif,
/// en file derrière la clôture — `I → M → clôture → I` (à partir de la 29-4b,
/// la revérification d'une annulation ajoute ses propres cycles). La clôture y
/// est la victime désignée, la plus légère (aucune écriture, aucun verrou
/// d'enregistrement accordé) — raisonné pour un cycle à plus de deux, éprouvé
/// par T29a-4. Les contrôles de forme et l'onboarding restent devant
/// l'enveloppe.
pub async fn close_bank_account(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    CloseBankAccountBodyExtractor(body): CloseBankAccountBodyExtractor,
) -> Result<Json<BankAccount>, AppError> {
    assert_onboarding_complete(&state.pool).await?;
    if body.version < 1 {
        return Err(AppError::Validation("version doit être >= 1".to_string()));
    }
    let closed_on = body
        .closed_on
        .unwrap_or_else(|| chrono::Utc::now().date_naive());
    // Revue P1 (B-1) — la date est informative (C-28-4-3), non sans domaine :
    // `chrono` admet des années signées et au-delà de 9999, que la colonne `DATE`
    // de MariaDB refuserait en `500`. Hors de `1000..=9999` (le domaine de
    // `DATE`) : `400`, avant toute transaction.
    if !(1000..=9999).contains(&chrono::Datelike::year(&closed_on)) {
        return Err(AppError::Validation(
            "closedOn doit être une date entre l'an 1000 et l'an 9999".to_string(),
        ));
    }

    let version = body.version;
    let after = crate::retry::retry_app_on_deadlock("bank_accounts::close", || {
        close_once(&state.pool, &current_user, id, version, closed_on)
    })
    .await?;
    Ok(Json(after))
}

/// Une tentative de la clôture : ouvre la transaction, appelle
/// `bank_accounts::close_in_tx` **en première instruction**, trace
/// `bank_account.closed` dans la même transaction, conclut. Sur refus,
/// `rollback` explicite **avant** de rendre l'erreur : la transaction tient le
/// `FOR UPDATE` du compte, qu'un abandon implicite ne rendrait qu'au retour de
/// la connexion au pool — autant d'attente en plus pour les écrivains en file
/// derrière la clôture. Les erreurs de dépôt sortent en
/// `AppError::Database(_)`, visibles du prédicat de rejeu.
async fn close_once(
    pool: &sqlx::MySqlPool,
    current_user: &CurrentUser,
    id: i64,
    version: i32,
    closed_on: chrono::NaiveDate,
) -> Result<BankAccount, AppError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;
    let result = async {
        let (after, before) = match bank_accounts::close_in_tx(
            &mut tx,
            current_user.company_id,
            id,
            version,
            closed_on,
        )
        .await
        {
            Ok(bank_accounts::ClosureOutcome::Done { after, before }) => (after, before),
            Ok(bank_accounts::ClosureOutcome::Refused(refusal)) => {
                return Err(closure_refusal_error(refusal));
            }
            Err(DbError::NotFound) => return Err(AppError::BankAccountNotFound),
            Err(e) => return Err(AppError::Database(e)),
        };

        let details = serde_json::json!({
            "bank_account_id": id,
            "closed_on": after.closed_on,
            "before": { "closed_on": before.closed_on, "version": before.version },
            "after": { "closed_on": after.closed_on, "version": after.version },
        });
        audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::from_current_user(
                current_user,
                "bank_account.closed",
                "bank_account",
                id,
                Some(details),
            ),
        )
        .await?;
        Ok(*after)
    }
    .await;
    conclure(tx, result).await
}

/// Conclut une tentative de [`close_once`], [`reopen_once`], [`patch_once`]
/// et — revue de code P1, E-1 — de [`create_once`], [`update_once`],
/// [`archive_once`] (Story 29-4a) : commit sur succès, `rollback`
/// **explicite** sur erreur avant de la rendre — la transaction tient le
/// `FOR UPDATE` de la ligne du compte, ou la sentinelle `companies`.
async fn conclure<T>(
    tx: Transaction<'_, MySql>,
    result: Result<T, AppError>,
) -> Result<T, AppError> {
    match result {
        Ok(v) => {
            tx.commit()
                .await
                .map_err(|e| AppError::Internal(format!("commit tx: {e}")))?;
            Ok(v)
        }
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

/// Handler `POST /api/v1/bank-accounts/{id}/reopen` — **rouvre** un compte
/// bancaire clos (Comptable+, clés d'API `read-write` admises ; Story 28-4a,
/// #642, AC4). Corps `{ "version" }`. Effet : `closed_on = NULL`,
/// `version + 1` ; le compte ne redevient pas principal. Trace
/// `bank_account.reopened`, dont `before.closed_on` garde la date levée.
///
/// **Ordre des refus** : forme (`400`) → `404` (inexistant, autre société,
/// archivé) → sous verrou : `409 OPTIMISTIC_LOCK_CONFLICT` → `409
/// BANK_ACCOUNT_NOT_CLOSED`.
///
/// ⚠️ **Rejouée sur interblocage** (opération `bank_accounts::reopen`, Story
/// 29-4a, #651, AC4) ; la transaction vit dans [`reopen_once`]. Comme la
/// clôture, la réouverture ne tient rien comme détenteur, mais sa demande
/// exclusive en attente sur la ligne du compte clos fait attendre derrière
/// elle les demandes **neuves** de la ligne. Cycle atteignable en 29-4a : un
/// import ou un lot `I` dont la vue précède la clôture a posé son `INSERT` à
/// clé étrangère (partagé du compte) et demande `companies`, que tient une
/// route de gestion `M` en file derrière la réouverture — `I → M →
/// réouverture → I` ; sa revérification, qui le refusera, ne fait pas la
/// queue (T29a-6, T29a-7). À partir de la 29-4b, une annulation qui revérifie
/// ajoute le cycle à quatre de la fiche. La réouverture en est la victime —
/// d'où le rejeu.
pub async fn reopen_bank_account(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    ReopenBankAccountBodyExtractor(body): ReopenBankAccountBodyExtractor,
) -> Result<Json<BankAccount>, AppError> {
    assert_onboarding_complete(&state.pool).await?;
    if body.version < 1 {
        return Err(AppError::Validation("version doit être >= 1".to_string()));
    }
    let version = body.version;
    let after = crate::retry::retry_app_on_deadlock("bank_accounts::reopen", || {
        reopen_once(&state.pool, &current_user, id, version)
    })
    .await?;
    Ok(Json(after))
}

/// Une tentative de la réouverture : `bank_accounts::reopen_in_tx` en
/// première instruction, trace `bank_account.reopened`, conclusion par
/// [`conclure`] (`rollback` explicite sur refus).
async fn reopen_once(
    pool: &sqlx::MySqlPool,
    current_user: &CurrentUser,
    id: i64,
    version: i32,
) -> Result<BankAccount, AppError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;
    let result =
        async {
            let (after, before) =
                match bank_accounts::reopen_in_tx(&mut tx, current_user.company_id, id, version)
                    .await
                {
                    Ok(bank_accounts::ClosureOutcome::Done { after, before }) => (after, before),
                    Ok(bank_accounts::ClosureOutcome::Refused(refusal)) => {
                        return Err(closure_refusal_error(refusal));
                    }
                    Err(DbError::NotFound) => return Err(AppError::BankAccountNotFound),
                    Err(e) => return Err(AppError::Database(e)),
                };

            let details = serde_json::json!({
                "bank_account_id": id,
                "before": { "closed_on": before.closed_on, "version": before.version },
                "after": { "closed_on": after.closed_on, "version": after.version },
            });
            audit_log::insert_in_tx(
                &mut tx,
                NewAuditLogEntry::from_current_user(
                    current_user,
                    "bank_account.reopened",
                    "bank_account",
                    id,
                    Some(details),
                ),
            )
            .await?;
            Ok(*after)
        }
        .await;
    conclure(tx, result).await
}

/// Handler `PATCH /api/v1/bank-accounts/{id}` — met à jour le
/// `journalAccountId` du bank_account (legacy 8-5a-zero).
///
/// Story v014-1 (F7 Pass 3 Opus) — `details_json.trigger = "journal_account_link"`
/// ajouté pour cohérence audit log avec PUT (`trigger = "full_update"`).
///
/// **Ordre des erreurs** (Story 15-5b, AC12 ; Story 15-6c, AC6) : forme (400)
/// → 404 / 400 type du compte lié → 404 compte bancaire → 409 version → 400
/// `ACCOUNT_NOT_POSTABLE` (compte lié **changé** vers un compte non imputable ;
/// une valeur inchangée court-circuite sans contrôle) → 400
/// `BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT` (compte lié **changé** vers le compte
/// débiteurs ou créanciers désigné).
///
/// ⚠️ **Rejouée sur interblocage** (opération `bank_accounts::patch`, Story
/// 29-4a, #651, AC4) ; la transaction vit dans [`patch_once`]. Sa demande
/// exclusive de la ligne du compte (`set_journal_account_id_for_company`), en
/// attente derrière un détenteur partagé, fait attendre les demandes neuves de
/// la ligne : en 29-4a, l'exclusif d'une route de gestion `M` qui tient la
/// sentinelle `companies` qu'attend un import ou un lot `I` (`I → M → PATCH →
/// I`) ; à partir de la 29-4b, aussi le cycle à trois par l'intervalle — un
/// import (partagé du compte) attend l'intervalle que tient une suppression
/// d'import, le `PATCH` attend derrière l'import, la suppression revérifie
/// derrière le `PATCH`. Le `PATCH`, qui n'a rien écrit, peut y être la victime.
/// La version et `validate_journal_account_id` (sur le pool) restent devant
/// l'enveloppe.
pub async fn patch_bank_account_journal_link(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    PatchJournalLinkBodyExtractor(body): PatchJournalLinkBodyExtractor,
) -> Result<Json<BankAccount>, AppError> {
    if body.version < 1 {
        return Err(AppError::Validation("version doit être >= 1".to_string()));
    }

    validate_journal_account_id(
        &state.pool,
        current_user.company_id,
        body.journal_account_id,
        // Postabilité contrôlée dans le dépôt, sous verrou, si changée (AC12).
        false,
    )
    .await?;

    let (journal_account_id, version) = (body.journal_account_id, body.version);
    let updated = crate::retry::retry_app_on_deadlock("bank_accounts::patch", || {
        patch_once(&state.pool, &current_user, id, journal_account_id, version)
    })
    .await?;
    Ok(Json(updated))
}

/// Une tentative du `PATCH` : partagés des réglages (`claims_for_target`, en
/// première instruction), `FOR UPDATE` du compte et `UPDATE`
/// (`set_journal_account_id_for_company`), trace `bank_account.updated` si la
/// version a bougé, conclusion par [`conclure`] (`rollback` explicite sur
/// refus).
async fn patch_once(
    pool: &sqlx::MySqlPool,
    current_user: &CurrentUser,
    id: i64,
    journal_account_id: Option<i64>,
    version: i32,
) -> Result<BankAccount, AppError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;
    let result = async {
        // Story 15-6c (AC2, AC4) — première instruction de la transaction (le
        // lien ne prend pas la sentinelle), avant le `FOR UPDATE` de la ligne.
        let claims =
            claims_for_target(&mut tx, current_user.company_id, journal_account_id).await?;

        let (updated, before) = match bank_accounts::set_journal_account_id_for_company(
            &mut tx,
            current_user.company_id,
            id,
            journal_account_id,
            version,
            &claims,
        )
        .await
        {
            Ok(pair) => pair,
            Err(DbError::NotFound) => return Err(AppError::BankAccountNotFound),
            Err(e) => return Err(AppError::Database(e)),
        };

        if updated.version != before.version {
            let details = serde_json::json!({
                "bank_account_id": id,
                "trigger": "journal_account_link",
                "before": {
                    "journal_account_id": before.journal_account_id,
                    "version": before.version,
                },
                "after": {
                    "journal_account_id": updated.journal_account_id,
                    "version": updated.version,
                },
            });
            audit_log::insert_in_tx(
                &mut tx,
                NewAuditLogEntry::from_current_user(
                    current_user,
                    "bank_account.updated",
                    "bank_account",
                    id,
                    Some(details),
                ),
            )
            .await?;
        }
        Ok(updated)
    }
    .await;
    conclure(tx, result).await
}

#[cfg(test)]
mod tests {
    use super::*;

    // Le test `ensemble_refuse_borne_par_borne` de la 26-2a a déménagé en
    // `kesh-core` (`text::tests::ancienne_liste_reste_refusee`), Story 32-8b0.

    /// Story 32-8b0 (AC-03) — un point **hors de l'ancienne liste fermée** pour
    /// chaque voie neuve du prédicat partagé : contrôle de format attribué
    /// (U+0600, Cf), usage privé (U+F8FF, le logo Apple), non attribué (U+0378),
    /// remplissage `Lo` (U+115F), sélecteur de variante supplémentaire `Mn`
    /// (U+E0100). Tous étaient admis avant la 32-8b0 (M08).
    #[test]
    fn description_refuse_par_categorie() {
        for c in ['\u{0600}', '\u{F8FF}', '\u{0378}', '\u{115F}', '\u{E0100}'] {
            assert!(is_forbidden_in_description(c), "{c:?} doit être refusé");
            let saisie = format!("Épargne {c}");
            assert!(
                normalize_description(Some(&saisie)).is_err(),
                "{saisie:?} doit être refusée"
            );
        }
    }

    /// Story 32-8b0 (AC-04), non-régression : un emoji simple est admis dans
    /// une description, un emoji composé (sélecteur U+FE0F, liant U+200D) y est
    /// refusé — l'ancienne liste le tenait déjà.
    #[test]
    fn emojis_simples_admis_composes_refuses() {
        for simple in ["Épargne \u{26A0}", "Pro \u{1F4BB}"] {
            assert_eq!(
                normalize_description(Some(simple)).unwrap().as_deref(),
                Some(simple),
                "{simple:?} (emoji simple) doit être admis"
            );
        }
        for compose in ["Épargne \u{26A0}\u{FE0F}", "Pro \u{1F468}\u{200D}\u{1F4BB}"] {
            assert!(
                normalize_description(Some(compose)).is_err(),
                "{compose:?} (emoji composé) doit être refusé"
            );
        }
    }

    #[test]
    fn controles_cc_refuses() {
        for c in [
            '\n', '\r', '\t', '\u{0000}', '\u{001F}', '\u{007F}', '\u{0085}', '\u{009F}',
        ] {
            assert!(is_forbidden_in_description(c), "{:?} doit être refusé", c);
        }
    }

    #[test]
    fn lettres_et_espaces_ordinaires_acceptes() {
        for c in ['a', 'É', 'é', ' ', '-', '\u{00A0}', '’', '«', '»', '—'] {
            assert!(!is_forbidden_in_description(c), "{:?} doit être accepté", c);
        }
        // Story 32-8b0 (revue P1, B-M1) : une saisie **décomposée** (`e` suivi
        // de l'accent aigu combinant U+0301, `Mn` visible) est admise telle
        // quelle — `normalize_description` ne recompose pas en NFC.
        let decomposee = "Re\u{0301}serve";
        assert_eq!(
            normalize_description(Some(decomposee)).unwrap().as_deref(),
            Some(decomposee),
            "{decomposee:?} (forme décomposée) doit être admise"
        );
    }

    #[test]
    fn normalisation_de_la_description() {
        assert_eq!(normalize_description(None).unwrap(), None);
        assert_eq!(normalize_description(Some("")).unwrap(), None);
        assert_eq!(normalize_description(Some("   ")).unwrap(), None);
        assert_eq!(
            normalize_description(Some("  Épargne  "))
                .unwrap()
                .as_deref(),
            Some("Épargne")
        );
        // Contrôle APRÈS trim (C-26-2-28) : `\n` de bord retiré, accepté.
        assert_eq!(
            normalize_description(Some("Épargne\n")).unwrap().as_deref(),
            Some("Épargne")
        );
        assert!(normalize_description(Some("Épar\ngne")).is_err());
        // Bornes écrites en littéraux (M4, M15) — jamais la constante.
        assert!(normalize_description(Some(&"a".repeat(60))).is_ok());
        assert!(normalize_description(Some(&"a".repeat(61))).is_err());
        // Caractères, pas octets : 60 « é » précomposés (120 octets) acceptés.
        assert!(normalize_description(Some(&"\u{e9}".repeat(60))).is_ok());
        assert!(normalize_description(Some(&"\u{e9}".repeat(61))).is_err());
    }
}
