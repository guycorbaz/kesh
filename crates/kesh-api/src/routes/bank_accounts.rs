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
use kesh_db::entities::bank_account::{BankAccount, NewBankAccount};
use kesh_db::errors::DbError;
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
    pub iban: String,
    #[serde(default)]
    pub qr_iban: Option<String>,
    #[serde(default)]
    pub is_primary: bool,
    #[serde(default)]
    pub journal_account_id: Option<i64>,
}

/// Body du PUT `/bank-accounts/{id}` — édition complète v014-1.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBankAccountBody {
    pub bank_name: String,
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

/// Query string du GET `/bank-accounts` — paramètre `includeArchived`.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListBankAccountsQuery {
    #[serde(default)]
    pub include_archived: bool,
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
    pub current_balance: Option<Decimal>,
    pub last_transaction_date: Option<chrono::NaiveDate>,
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
    "`bankName` str, `iban` str, `qrIban?` str, `isPrimary?` bool, `journalAccountId?` i64"
);
impl_validation_extractor!(
    UpdateBankAccountBodyExtractor,
    UpdateBankAccountBody,
    "`bankName` str, `iban` str, `qrIban?` str, `isPrimary?` bool, `journalAccountId?` i64, `version` i32"
);
impl_validation_extractor!(
    ArchiveBankAccountBodyExtractor,
    ArchiveBankAccountBody,
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

/// Valide et normalise les champs métier d'un payload create/update.
///
/// Retourne `(bank_name normalisé, iban normalisé, qr_iban normalisé)`.
fn validate_bank_input(
    bank_name: &str,
    iban: &str,
    qr_iban: Option<&str>,
) -> Result<(String, String, Option<String>), AppError> {
    let bank_name = bank_name.trim().to_string();
    if bank_name.is_empty() {
        return Err(AppError::Validation(
            "Le nom de la banque ne peut pas être vide".into(),
        ));
    }

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

    Ok((bank_name, iban_normalized, qr_iban_normalized))
}

/// Validation pré-flight du `journalAccountId` (compte existe + actif +
/// Asset|Liability) — cohérent pattern PATCH.
async fn validate_journal_account_id(
    pool: &sqlx::MySqlPool,
    company_id: i64,
    journal_account_id: Option<i64>,
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
        AccountType::Asset | AccountType::Liability => Ok(()),
        other => Err(AppError::InvalidAccountType {
            account_id,
            account_type: other.as_str().to_string(),
        }),
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
        query.include_archived,
    )
    .await?;

    let payload: Vec<BankAccountWithBalance> = rows
        .into_iter()
        .map(
            |(account, current_balance, last_transaction_date)| BankAccountWithBalance {
                account,
                current_balance,
                last_transaction_date,
            },
        )
        .collect();

    Ok(Json(payload))
}

/// Handler `POST /api/v1/bank-accounts` — création v014-1 (Comptable+).
///
/// Transition primary silencieuse symétrique au PUT (FINDING-3 Pass 3 Opus).
/// Guard onboarding step >= 7 (sauf is_demo).
pub async fn create_bank_account(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    CreateBankAccountBodyExtractor(body): CreateBankAccountBodyExtractor,
) -> Result<(StatusCode, Json<BankAccount>), AppError> {
    assert_onboarding_complete(&state.pool).await?;

    let (bank_name, iban, qr_iban) =
        validate_bank_input(&body.bank_name, &body.iban, body.qr_iban.as_deref())?;
    validate_journal_account_id(
        &state.pool,
        current_user.company_id,
        body.journal_account_id,
    )
    .await?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;

    // Advisory lock sentinel (L5 mitigation — FINDING-9 Pass 3 Opus).
    bank_accounts::acquire_company_sentinel_lock(&mut tx, current_user.company_id).await?;

    // Si is_primary=true et un autre primary existe → flip silencieux atomique.
    // L'INSERT n'a pas encore d'id, on passe -1 comme excluded_id sentinel
    // (les ids MariaDB AUTO_INCREMENT sont toujours > 0 — F8 Pass 1 code review,
    // plus défensif que 0 qui pourrait coïncider avec un id dans des fixtures
    // pathologiques avec `AUTO_INCREMENT=0`).
    let demoted_primary = if body.is_primary {
        bank_accounts::flip_primary_off_for_company(&mut tx, current_user.company_id, -1)
            .await
            .map_err(AppError::Database)?
    } else {
        None
    };

    // INSERT du nouveau compte.
    let new = NewBankAccount {
        company_id: current_user.company_id,
        bank_name,
        iban,
        qr_iban,
        is_primary: body.is_primary,
    };

    let result = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, qr_iban, is_primary, journal_account_id) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(new.company_id)
    .bind(&new.bank_name)
    .bind(&new.iban)
    .bind(&new.qr_iban)
    .bind(new.is_primary)
    .bind(body.journal_account_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Database(kesh_db::errors::map_db_error(e)))?;

    let new_id = i64::try_from(result.last_insert_id())
        .map_err(|_| AppError::Internal("last_insert_id overflow".into()))?;

    let inserted = sqlx::query_as::<_, BankAccount>(
        "SELECT id, company_id, bank_name, iban, qr_iban, is_primary, journal_account_id, version, archived, created_at, updated_at \
         FROM bank_accounts WHERE id = ?",
    )
    .bind(new_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| AppError::Database(kesh_db::errors::map_db_error(e)))?;

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
        "is_primary": inserted.is_primary,
        "iban_present": true,
        "qr_iban_present": inserted.qr_iban.is_some(),
        "journal_account_id": inserted.journal_account_id,
    });
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::from_current_user(
            &current_user,
            "bank_account.created",
            "bank_account",
            new_id,
            Some(details),
        ),
    )
    .await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit tx: {e}")))?;

    Ok((StatusCode::CREATED, Json(inserted)))
}

/// Handler `PUT /api/v1/bank-accounts/{id}` — édition complète v014-1
/// (Comptable+).
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

    let (bank_name, iban, qr_iban) =
        validate_bank_input(&body.bank_name, &body.iban, body.qr_iban.as_deref())?;
    validate_journal_account_id(
        &state.pool,
        current_user.company_id,
        body.journal_account_id,
    )
    .await?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;

    bank_accounts::acquire_company_sentinel_lock(&mut tx, current_user.company_id).await?;

    // Si l'utilisateur veut promouvoir ce compte primary, flip l'éventuel
    // ancien (avant l'UPDATE de notre row, dans la même tx).
    let demoted_primary = if body.is_primary {
        bank_accounts::flip_primary_off_for_company(&mut tx, current_user.company_id, id)
            .await
            .map_err(AppError::Database)?
    } else {
        None
    };

    let new = NewBankAccount {
        company_id: current_user.company_id,
        bank_name,
        iban,
        qr_iban,
        is_primary: body.is_primary,
    };

    let (updated, before) = match bank_accounts::update_for_company(
        &mut tx,
        current_user.company_id,
        id,
        &new,
        body.journal_account_id,
        body.version,
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
            "iban_present": true,
            "qr_iban_present": before.qr_iban.is_some(),
            "is_primary": before.is_primary,
            "journal_account_id": before.journal_account_id,
            "version": before.version,
        },
        "after": {
            "bank_name": updated.bank_name,
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
            &current_user,
            "bank_account.updated",
            "bank_account",
            id,
            Some(details),
        ),
    )
    .await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit tx: {e}")))?;

    Ok(Json(updated))
}

/// Handler `DELETE /api/v1/bank-accounts/{id}` — soft-delete (archive) v014-1
/// (Comptable+).
///
/// AC#8 : refus 412 BANK_ACCOUNT_HAS_TRANSACTIONS si transactions existent.
/// AC#9 : refus 412 BANK_ACCOUNT_CANNOT_ARCHIVE_PRIMARY si primary + autres.
/// AC#10 : autorisé pour primary unique (cas dégénéré).
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

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;

    // Advisory lock sentinel — serialize les mutations CRUD bank_accounts d'un
    // même tenant ET protège contre une création concurrente de
    // bank_transactions via le même chemin (mais bank_imports n'acquiert pas
    // ce lock — L5 limitation v0.1 documentée).
    bank_accounts::acquire_company_sentinel_lock(&mut tx, current_user.company_id).await?;

    // Guard transactions (AC#8) — DANS la tx, après sentinel lock.
    let tx_count =
        bank_accounts::count_transactions_for_bank_account(&mut tx, current_user.company_id, id)
            .await?;
    if tx_count > 0 {
        return Err(AppError::BankAccountHasTransactions {
            transaction_count: tx_count,
        });
    }

    // Guard primary + autres comptes actifs (AC#9 / AC#10) — DANS la tx.
    if existing.is_primary {
        let others =
            bank_accounts::count_other_active_for_company(&mut tx, current_user.company_id, id)
                .await?;
        if others > 0 {
            return Err(AppError::BankAccountCannotArchivePrimary);
        }
        // others == 0 : primary unique → autorisé (AC#10).
    }

    let (updated, before) = match bank_accounts::archive_for_company(
        &mut tx,
        current_user.company_id,
        id,
        body.version,
    )
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
            &current_user,
            "bank_account.archived",
            "bank_account",
            id,
            Some(details),
        ),
    )
    .await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit tx: {e}")))?;

    Ok(Json(updated))
}

/// Handler `PATCH /api/v1/bank-accounts/{id}` — met à jour le
/// `journalAccountId` du bank_account (legacy 8-5a-zero).
///
/// Story v014-1 (F7 Pass 3 Opus) — `details_json.trigger = "journal_account_link"`
/// ajouté pour cohérence audit log avec PUT (`trigger = "full_update"`).
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
    )
    .await?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;

    let (updated, before) = match bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        current_user.company_id,
        id,
        body.journal_account_id,
        body.version,
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
                &current_user,
                "bank_account.updated",
                "bank_account",
                id,
                Some(details),
            ),
        )
        .await?;
    }

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit tx: {e}")))?;

    Ok(Json(updated))
}
